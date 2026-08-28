//! Orchestration des quatre étapes du cœur.
//!
//! ```text
//! Utilisateur ─ image ─▶ Analyzer ─┐
//!                                  ├─▶ Constructor ─▶ Formatter ─▶ Renderer ─▶ sorties
//! Utilisateur ─ filtres ─▶ Registre ┘                                 (applique les filtres)
//! ```
//!
//! Le [`Pipeline`] est immuable et réutilisable pour plusieurs images.

use std::path::Path;

use crate::model::error::Result;
use crate::model::filter::contract::FilterChain;
use crate::model::pipeline::analyzer::{Analyzer, ImageAnalysis, Origin};
use crate::model::pipeline::constructor::{ConstructionPolicy, Constructor};
use crate::model::pipeline::formatter::{Formatter, OutputFormat, SizeVariant};
use crate::model::pipeline::renderer::{RenderOutput, Renderer};

/// Chaîne de traitement complète.
#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    pub analyzer: Analyzer,
    pub constructor: Constructor,
    pub formatter: Formatter,
    pub renderer: Renderer,
}

impl Pipeline {
    /// Pipeline par défaut : PNG, dimensions d'origine, aucune limite.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }

    /// Traite un fichier de bout en bout.
    pub fn run_path(&self, input: impl AsRef<Path>, chain: &FilterChain) -> Result<RenderOutput> {
        let analysis = self.analyzer.analyze_path(input)?;
        self.run_analysis(analysis, chain)
    }

    /// Traite un flux déjà en mémoire.
    pub fn run_bytes(
        &self,
        bytes: &[u8],
        origin: Origin,
        chain: &FilterChain,
    ) -> Result<RenderOutput> {
        let analysis = self.analyzer.analyze_bytes(bytes, origin)?;
        self.run_analysis(analysis, chain)
    }

    /// Reprend le pipeline après l'analyse (utile pour les tests et les caches).
    pub fn run_analysis(
        &self,
        analysis: ImageAnalysis,
        chain: &FilterChain,
    ) -> Result<RenderOutput> {
        let prepared = self.constructor.prepare(analysis, chain)?;
        let set = self.formatter.format(&prepared)?;
        self.renderer.render(set, chain, &prepared.metadata)
    }
}

/// Construction pas à pas d'un [`Pipeline`], avec validation des réglages.
#[derive(Debug, Clone, Default)]
pub struct PipelineBuilder {
    formats: Vec<OutputFormat>,
    sizes: Vec<SizeVariant>,
    quality: Option<u8>,
    max_dimension: Option<u32>,
    max_pixels: Option<Option<u64>>,
    stem: Option<String>,
}

impl PipelineBuilder {
    pub fn formats(mut self, formats: Vec<OutputFormat>) -> Self {
        self.formats = formats;
        self
    }

    pub fn sizes(mut self, sizes: Vec<SizeVariant>) -> Self {
        self.sizes = sizes;
        self
    }

    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Borne le plus grand côté de l'image de travail.
    pub fn max_dimension(mut self, max_dimension: Option<u32>) -> Self {
        self.max_dimension = max_dimension;
        self
    }

    /// Borne la taille décompressée acceptée par l'Analyzer.
    pub fn max_pixels(mut self, max_pixels: Option<u64>) -> Self {
        self.max_pixels = Some(max_pixels);
        self
    }

    pub fn stem(mut self, stem: impl Into<String>) -> Self {
        self.stem = Some(stem.into());
        self
    }

    /// Valide les réglages et assemble le pipeline.
    pub fn build(self) -> Result<Pipeline> {
        let mut analyzer = Analyzer::new();
        if let Some(max_pixels) = self.max_pixels {
            analyzer = analyzer.with_max_pixels(max_pixels);
        }

        let constructor = Constructor::with_policy(ConstructionPolicy {
            max_dimension: self.max_dimension,
            ..ConstructionPolicy::default()
        });

        let mut formatter = Formatter::new();
        if !self.formats.is_empty() {
            formatter = formatter.with_formats(self.formats)?;
        }
        formatter = formatter.with_sizes(self.sizes);
        if let Some(quality) = self.quality {
            formatter = formatter.with_quality(quality)?;
        }

        let renderer = match self.stem {
            Some(stem) => Renderer::new().with_stem(stem),
            None => Renderer::new(),
        };

        Ok(Pipeline {
            analyzer,
            constructor,
            formatter,
            renderer,
        })
    }
}
