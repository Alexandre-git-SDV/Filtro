//! Étape 4 — **Drawer / Renderer**.
//!
//! Applique la chaîne de filtres à chaque variante produite par le Formatter,
//! puis encode le résultat. Le Renderer est le seul composant du cœur qui
//! *exécute* du code de filtre — et il ne le fait qu'à travers le trait
//! [`Filter`](crate::model::filter::contract::Filter) : il ignore totalement ce que ces filtres font.

use std::path::{Path, PathBuf};

use crate::model::codec;
use crate::model::error::{FiltroError, Result};
use crate::model::filter::contract::FilterChain;
use crate::model::pipeline::analyzer::ImageMetadata;
use crate::model::pipeline::formatter::{FormatSet, OutputTarget};

/// Une image finale encodée, prête à être écrite ou servie.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedArtifact {
    /// Variante à l'origine de ce rendu.
    pub target: OutputTarget,
    /// Nom de fichier suggéré.
    pub file_name: String,
    /// Octets encodés.
    pub bytes: Vec<u8>,
}

impl RenderedArtifact {
    pub fn byte_size(&self) -> usize {
        self.bytes.len()
    }
}

/// Résultat complet du pipeline, proposé à l'utilisateur.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderOutput {
    /// Description de l'image source.
    pub metadata: ImageMetadata,
    /// Identifiants des filtres appliqués, dans l'ordre.
    pub applied_filters: Vec<String>,
    /// Une entrée par variante.
    pub artifacts: Vec<RenderedArtifact>,
}

impl RenderOutput {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// Écrit toutes les variantes dans un répertoire, créé si nécessaire.
    pub fn write_to_dir(&self, dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).map_err(|source| FiltroError::io(dir, source))?;
        let mut written = Vec::with_capacity(self.artifacts.len());
        for artifact in &self.artifacts {
            let path = dir.join(&artifact.file_name);
            std::fs::write(&path, &artifact.bytes)
                .map_err(|source| FiltroError::io(&path, source))?;
            written.push(path);
        }
        Ok(written)
    }
}

/// Étape de rendu.
#[derive(Debug, Clone, Default)]
pub struct Renderer {
    stem: Option<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Force le radical des noms de fichiers (par défaut : celui de la source).
    pub fn with_stem(mut self, stem: impl Into<String>) -> Self {
        self.stem = Some(stem.into());
        self
    }

    /// Applique la chaîne à chaque variante et encode le résultat.
    ///
    /// Les filtres sont appliqués **par variante** : un filtre peut donc
    /// s'adapter au format ou aux dimensions cibles via
    /// [`FilterContext`](crate::model::filter::contract::FilterContext).
    pub fn render(
        &self,
        set: FormatSet,
        chain: &FilterChain,
        metadata: &ImageMetadata,
    ) -> Result<RenderOutput> {
        let stem = self.stem.clone().unwrap_or_else(|| metadata.origin.stem());

        let mut artifacts = Vec::with_capacity(set.variants.len());
        for mut variant in set.variants {
            chain.apply(&mut variant.canvas, metadata, &variant.target)?;

            // Un filtre a pu modifier les dimensions : la cible est alignée
            // sur le résultat réel avant encodage.
            variant.target.dimensions = variant.canvas.dimensions();

            // Second passage de sécurité : si un filtre a réintroduit de la
            // transparence sur un format qui ne la gère pas.
            if !variant.target.format.supports_alpha() {
                variant
                    .canvas
                    .flatten_onto(variant.target.options.background);
            }

            let bytes = codec::encode(
                &variant.canvas,
                variant.target.format,
                &variant.target.options,
            )?;

            artifacts.push(RenderedArtifact {
                file_name: variant.target.file_name(&stem),
                target: variant.target,
                bytes,
            });
        }

        Ok(RenderOutput {
            metadata: metadata.clone(),
            applied_filters: chain.ids().into_iter().map(str::to_owned).collect(),
            artifacts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::filter::contract::{Filter, FilterContext};
    use crate::model::pipeline::formatter::{Formatter, OutputFormat};
    use crate::model::pixel::{PixelBuffer, Rgba8};
    use crate::test_support::prepared;

    /// Filtre factice, défini ici pour vérifier que le trait suffit.
    struct Inverseur;

    impl Filter for Inverseur {
        fn id(&self) -> &str {
            "inverseur-de-test"
        }

        fn apply(&self, canvas: &mut PixelBuffer, _ctx: &FilterContext<'_>) -> Result<()> {
            canvas.map_pixels(|_, _, p| Rgba8::new(255 - p.r, 255 - p.g, 255 - p.b, p.a));
            Ok(())
        }
    }

    #[test]
    fn rend_toutes_les_variantes() {
        let image = prepared(8, 8, Rgba8::BLACK);
        let set = Formatter::new()
            .with_formats(vec![OutputFormat::Png, OutputFormat::Jpeg])
            .unwrap()
            .format(&image)
            .unwrap();

        let output = Renderer::new()
            .with_stem("photo")
            .render(set, &FilterChain::empty(), &image.metadata)
            .unwrap();

        assert_eq!(output.len(), 2);
        assert!(output.applied_filters.is_empty());
        assert_eq!(output.artifacts[0].file_name, "photo.png");
        assert!(output.artifacts.iter().all(|a| a.byte_size() > 0));
    }

    #[test]
    fn applique_la_chaine_a_chaque_variante() {
        let image = prepared(8, 8, Rgba8::BLACK);
        let set = Formatter::new().format(&image).unwrap();
        let chain = FilterChain::new(vec![Box::new(Inverseur)]);

        let output = Renderer::new()
            .render(set, &chain, &image.metadata)
            .unwrap();

        assert_eq!(output.applied_filters, vec!["inverseur-de-test"]);

        // L'image noire est devenue blanche : on relit le PNG produit.
        let (buffer, _) = codec::decode(&output.artifacts[0].bytes, "test").unwrap();
        assert_eq!(buffer.pixel(0, 0), Some(Rgba8::WHITE));
    }
}
