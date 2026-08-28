//! **Le contrat des filtres** — et rien d'autre.
//!
//! Ce fichier ne contient aucun filtre. Il décrit ce qu'un filtre *est*, du
//! point de vue du cœur :
//!
//! * il porte un identifiant ([`Filter::id`]) ;
//! * il peut déclarer ce qu'il attend de l'image ([`FilterRequirements`]) ;
//! * il transforme des pixels ([`Filter::apply`]).
//!
//! L'instanciation depuis la ligne de commande est décrite dans
//! [`crate::model::filter::registry`].

use std::fmt;

use crate::model::error::Result;
use crate::model::pipeline::analyzer::ImageMetadata;
use crate::model::pipeline::formatter::OutputTarget;
use crate::model::pixel::PixelBuffer;

/// Ce qu'un filtre exige de l'image qu'on lui remet.
///
/// C'est le **seul** canal par lequel un filtre influence la préparation :
/// le Constructor lit ces données, jamais le filtre lui-même.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilterRequirements {
    /// Le filtre a besoin d'un canal alpha significatif.
    pub needs_alpha: bool,
    /// Plus grand côté toléré ; le Constructor réduit l'image si nécessaire.
    pub max_dimension: Option<u32>,
}

impl FilterRequirements {
    /// Combine deux jeux d'exigences en retenant la contrainte la plus stricte.
    pub fn merge(self, other: Self) -> Self {
        Self {
            needs_alpha: self.needs_alpha || other.needs_alpha,
            max_dimension: match (self.max_dimension, other.max_dimension) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(v), None) | (None, Some(v)) => Some(v),
                (None, None) => None,
            },
        }
    }
}

/// Ce que le filtre sait du traitement en cours.
///
/// Le Renderer appelant les filtres une fois par variante, un filtre peut
/// s'adapter au format ou aux dimensions de sortie.
#[derive(Debug, Clone, Copy)]
pub struct FilterContext<'a> {
    /// Description de l'image source.
    pub metadata: &'a ImageMetadata,
    /// Variante en cours de rendu.
    pub target: &'a OutputTarget,
    /// Rang du filtre dans la chaîne (0 = premier).
    pub position: usize,
}

/// Un filtre configuré, prêt à traiter des pixels.
///
/// `Send + Sync` pour autoriser un rendu parallèle des variantes.
pub trait Filter: Send + Sync {
    /// Identifiant du filtre, tel qu'écrit sur la ligne de commande.
    fn id(&self) -> &str;

    /// Exigences sur l'image d'entrée. Par défaut : aucune.
    fn requirements(&self) -> FilterRequirements {
        FilterRequirements::default()
    }

    /// Transforme les pixels sur place.
    ///
    /// Un filtre qui change les dimensions (recadrage, rotation) remplace le
    /// contenu de `canvas` par un nouveau [`PixelBuffer`].
    fn apply(&self, canvas: &mut PixelBuffer, ctx: &FilterContext<'_>) -> Result<()>;
}

/// Suite ordonnée de filtres, appliquée telle quelle par le Renderer.
#[derive(Default)]
pub struct FilterChain {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    /// Chaîne vide : le pipeline se comporte alors en convertisseur de formats.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new(filters: Vec<Box<dyn Filter>>) -> Self {
        Self { filters }
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Identifiants des filtres, dans l'ordre d'application.
    pub fn ids(&self) -> Vec<&str> {
        self.filters.iter().map(|f| f.id()).collect()
    }

    /// Exigences cumulées, lues par le Constructor.
    pub fn requirements(&self) -> FilterRequirements {
        self.filters
            .iter()
            .fold(FilterRequirements::default(), |acc, f| {
                acc.merge(f.requirements())
            })
    }

    /// Applique tous les filtres, dans l'ordre.
    pub fn apply(
        &self,
        canvas: &mut PixelBuffer,
        metadata: &ImageMetadata,
        target: &OutputTarget,
    ) -> Result<()> {
        for (position, filter) in self.filters.iter().enumerate() {
            let ctx = FilterContext {
                metadata,
                target,
                position,
            };
            filter.apply(canvas, &ctx)?;
        }
        Ok(())
    }
}

impl fmt::Debug for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterChain")
            .field("ids", &self.ids())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fusion_des_exigences() {
        let a = FilterRequirements {
            needs_alpha: true,
            max_dimension: Some(4096),
        };
        let b = FilterRequirements {
            needs_alpha: false,
            max_dimension: Some(2048),
        };
        assert_eq!(
            a.merge(b),
            FilterRequirements {
                needs_alpha: true,
                max_dimension: Some(2048),
            }
        );
    }

    #[test]
    fn chaine_vide() {
        let chain = FilterChain::empty();
        assert!(chain.is_empty());
        assert_eq!(chain.requirements(), FilterRequirements::default());
    }
}
