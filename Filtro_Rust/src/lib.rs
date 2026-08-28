//! # Filtro
//!
//! Traitement d'images organisé en quatre étapes stables, plus un jeu de
//! filtres qui vit à l'extérieur de ces étapes.
//!
//! ## Architecture (MVC)
//!
//! | Couche | Emplacement | Rôle |
//! |--------|-------------|------|
//! | **Model** | [`model`] (cette bibliothèque) | le cœur métier : pipeline, contrat des filtres, types d'image |
//! | **View** | `src/view/` (binaire) | rendu terminal des résultats et du catalogue de filtres |
//! | **Controller** | `src/controller/` (binaire) | traduction des arguments CLI en configuration du cœur |
//!
//! Les filtres livrés avec le projet sont regroupés dans [`filters`] : ce module
//! n'utilise que l'API publique ci-dessous et aucune étape du pipeline ne le
//! référence.
//!
//! ## Le pipeline
//!
//! | Étape | Rôle | Entrée | Sortie |
//! |-------|------|--------|--------|
//! | [`Analyzer`] | décode et décrit l'image | octets | [`ImageAnalysis`] |
//! | [`Constructor`] | prépare l'image de travail | analyse + exigences des filtres | [`PreparedImage`] |
//! | [`Formatter`] | décline formats et tailles | image préparée | [`FormatSet`] |
//! | [`Renderer`] | applique les filtres et encode | variantes + chaîne | [`RenderOutput`] |
//!
//! ```no_run
//! use filtro::{FilterRegistry, FilterRequest, OutputFormat, Pipeline};
//!
//! # fn main() -> filtro::Result<()> {
//! let mut registry = FilterRegistry::new();
//! filtro::filters::register_all(&mut registry)?;
//!
//! let requests = [FilterRequest::parse("jaune:niveau=200")?];
//! let chain = registry.build_chain(&requests)?;
//!
//! let pipeline = Pipeline::builder()
//!     .formats(vec![OutputFormat::Png, OutputFormat::Jpeg])
//!     .quality(90)
//!     .build()?;
//!
//! pipeline.run_path("photo.jpg", &chain)?.write_to_dir("sortie")?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

// --- Le cœur (Model) ------------------------------------------------------
pub mod model;

// --- Les filtres, hors du cœur ------------------------------------------
pub mod filters;

#[cfg(test)]
mod test_support;

// --- Façade publique ----------------------------------------------------
// L'API exposée est stable : seuls les chemins internes ont bougé.
pub use crate::model::error::{FiltroError, Result};
pub use crate::model::filter::contract::{Filter, FilterChain, FilterContext, FilterRequirements};
pub use crate::model::filter::registry::{
    FilterFactory, FilterParams, FilterRegistry, FilterRequest, ParamSpec,
};
pub use crate::model::pipeline::analyzer::{
    Analyzer, ColorStats, ImageAnalysis, ImageMetadata, Origin, SourceFormat,
};
pub use crate::model::pipeline::constructor::{ConstructionPolicy, Constructor, PreparedImage};
pub use crate::model::pipeline::formatter::{
    FormatOptions, FormatSet, FormattedImage, Formatter, OutputFormat, OutputTarget, SizeVariant,
};
pub use crate::model::pipeline::orchestrator::{Pipeline, PipelineBuilder};
pub use crate::model::pipeline::renderer::{RenderOutput, RenderedArtifact, Renderer};
pub use crate::model::pixel::{Dimensions, PixelBuffer, Rgba8};
pub use crate::model::resolution::Resolution;
