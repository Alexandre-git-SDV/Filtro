//! Étape 2 — **Constructor**.
//!
//! Reçoit l'analyse (Analyzer) et les exigences de la chaîne de filtres, et en
//! tire une image de travail cohérente : c'est le seul endroit où l'image
//! d'origine est normalisée avant traitement.
//!
//! Le Constructor ne connaît **aucun filtre concret** : il ne lit que
//! [`FilterRequirements`], donnée déclarative fournie par le contrat.

use crate::model::codec;
use crate::model::error::Result;
use crate::model::filter::contract::{FilterChain, FilterRequirements};
use crate::model::pipeline::analyzer::{ImageAnalysis, ImageMetadata};
use crate::model::pixel::{Dimensions, PixelBuffer, Rgba8};

/// Politique de préparation, indépendante des filtres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstructionPolicy {
    /// Plus grand côté de l'image de travail (`None` = taille d'origine).
    pub max_dimension: Option<u32>,
    /// Couleur de fond utilisée lorsqu'il faut supprimer la transparence.
    pub background: Rgba8,
}

impl Default for ConstructionPolicy {
    fn default() -> Self {
        Self {
            max_dimension: None,
            background: Rgba8::WHITE,
        }
    }
}

/// Image prête à être déclinée par le Formatter puis rendue par le Renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedImage {
    /// Description de la source (dimensions d'origine incluses).
    pub metadata: ImageMetadata,
    /// Image de travail en RGBA8.
    pub canvas: PixelBuffer,
    /// Exigences cumulées de la chaîne, transmises aux étapes suivantes.
    pub requirements: FilterRequirements,
    /// Couleur de fond retenue pour les formats sans alpha.
    pub background: Rgba8,
    /// Dimensions d'origine si l'image a été réduite, `None` sinon.
    pub resized_from: Option<Dimensions>,
}

impl PreparedImage {
    pub fn dimensions(&self) -> Dimensions {
        self.canvas.dimensions()
    }
}

/// Étape de préparation.
#[derive(Debug, Clone, Default)]
pub struct Constructor {
    policy: ConstructionPolicy,
}

impl Constructor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_policy(policy: ConstructionPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> ConstructionPolicy {
        self.policy
    }

    /// Prépare l'image de travail.
    ///
    /// La borne dimensionnelle retenue est la plus stricte entre la politique
    /// de l'application et les exigences de la chaîne de filtres.
    pub fn prepare(&self, analysis: ImageAnalysis, chain: &FilterChain) -> Result<PreparedImage> {
        let requirements = chain.requirements();
        let (metadata, mut canvas) = analysis.into_parts();

        // Borne la plus stricte entre la politique de l'application et les
        // exigences déclarées par les filtres.
        let bound = match (self.policy.max_dimension, requirements.max_dimension) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(v), None) | (None, Some(v)) => Some(v),
            (None, None) => None,
        };

        let mut resized_from = None;
        if let Some(max_side) = bound
            && let Some(target) = canvas.dimensions().scaled_to_fit(max_side)
        {
            resized_from = Some(canvas.dimensions());
            canvas = codec::resize(&canvas, target)?;
        }

        // Le tampon reste en RGBA8 à alpha non prémultiplié : c'est la forme
        // canonique du cœur. Une future exigence de conversion (espace
        // linéaire, etc.) se traiterait ici, à partir de `requirements` seul.

        Ok(PreparedImage {
            metadata,
            canvas,
            requirements,
            background: self.policy.background,
            resized_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{canvas, metadata};

    fn analyse(width: u32, height: u32) -> ImageAnalysis {
        let buffer = canvas(width, height, Rgba8::BLACK);
        ImageAnalysis {
            metadata: metadata(buffer.dimensions()),
            buffer,
        }
    }

    #[test]
    fn conserve_les_dimensions_sans_contrainte() {
        let prepared = Constructor::new()
            .prepare(analyse(120, 80), &FilterChain::empty())
            .unwrap();
        assert_eq!(prepared.dimensions().width, 120);
        assert!(prepared.resized_from.is_none());
    }

    #[test]
    fn applique_la_borne_de_la_politique() {
        let constructor = Constructor::with_policy(ConstructionPolicy {
            max_dimension: Some(60),
            ..ConstructionPolicy::default()
        });

        let prepared = constructor
            .prepare(analyse(120, 80), &FilterChain::empty())
            .unwrap();

        assert_eq!(prepared.dimensions().width, 60);
        assert_eq!(prepared.dimensions().height, 40);
        assert_eq!(
            prepared.resized_from,
            Some(Dimensions::new(120, 80).unwrap())
        );
        // Les dimensions d'origine restent lisibles dans la description.
        assert_eq!(prepared.metadata.dimensions.width, 120);
    }
}
