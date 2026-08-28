//! Le pipeline : quatre étapes stables, plus leur orchestration.
//!
//! | Étape | Module | Entrée | Sortie |
//! |-------|--------|--------|--------|
//! | [`Analyzer`](analyzer::Analyzer) | [`analyzer`] | octets | [`ImageAnalysis`](analyzer::ImageAnalysis) |
//! | [`Constructor`](constructor::Constructor) | [`constructor`] | analyse + exigences | [`PreparedImage`](constructor::PreparedImage) |
//! | [`Formatter`](formatter::Formatter) | [`formatter`] | image préparée | [`FormatSet`](formatter::FormatSet) |
//! | [`Renderer`](renderer::Renderer) | [`renderer`] | variantes + chaîne | [`RenderOutput`](renderer::RenderOutput) |
//!
//! [`orchestrator`] assemble ces étapes en un [`Pipeline`](orchestrator::Pipeline)
//! immuable et réutilisable.

pub mod analyzer;
pub mod constructor;
pub mod formatter;
pub mod orchestrator;
pub mod renderer;
