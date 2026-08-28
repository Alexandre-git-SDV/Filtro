//! Fabriques d'objets pour les tests — compilé uniquement avec `cargo test`.
//!
//! Évite de répéter la construction d'une `ImageMetadata` ou d'une
//! `OutputTarget` complète dans chaque test.

use crate::model::filter::contract::{Filter, FilterContext, FilterRequirements};
use crate::model::pipeline::analyzer::{ColorStats, ImageMetadata, Origin, SourceFormat};
use crate::model::pipeline::constructor::PreparedImage;
use crate::model::pipeline::formatter::{FormatOptions, OutputFormat, OutputTarget, SizeVariant};
use crate::model::pixel::{Dimensions, PixelBuffer, Rgba8};

/// Description d'image factice, cohérente avec les dimensions données.
pub(crate) fn metadata(dimensions: Dimensions) -> ImageMetadata {
    ImageMetadata {
        origin: Origin::Memory {
            label: "test".into(),
        },
        source_format: SourceFormat::Png,
        dimensions,
        resolution: None,
        byte_size: 0,
        stats: ColorStats {
            has_transparency: false,
            is_grayscale: false,
            mean_luminance: 0.0,
            luminance_range: (0, 255),
        },
    }
}

/// Image uniforme de la taille demandée.
pub(crate) fn canvas(width: u32, height: u32, pixel: Rgba8) -> PixelBuffer {
    let dimensions = Dimensions::new(width, height).expect("dimensions valides");
    let mut canvas = PixelBuffer::new(dimensions).expect("allocation");
    canvas.map_pixels(|_, _, _| pixel);
    canvas
}

/// Image préparée uniforme, prête pour le Formatter ou le Renderer.
pub(crate) fn prepared(width: u32, height: u32, pixel: Rgba8) -> PreparedImage {
    let canvas = canvas(width, height, pixel);
    PreparedImage {
        metadata: metadata(canvas.dimensions()),
        canvas,
        requirements: FilterRequirements::default(),
        background: Rgba8::WHITE,
        resized_from: None,
    }
}

/// Variante de sortie factice (PNG, taille d'origine).
pub(crate) fn target(dimensions: Dimensions) -> OutputTarget {
    OutputTarget {
        format: OutputFormat::Png,
        size: SizeVariant::Original,
        dimensions,
        options: FormatOptions::default(),
    }
}

/// Applique un filtre à une image de 1×1 pixel et renvoie le résultat.
pub(crate) fn apply_to_pixel(filter: &dyn Filter, pixel: Rgba8) -> Rgba8 {
    let mut buffer = canvas(1, 1, pixel);
    let dimensions = buffer.dimensions();
    let metadata = metadata(dimensions);
    let target = target(dimensions);
    let ctx = FilterContext {
        metadata: &metadata,
        target: &target,
        position: 0,
    };
    filter.apply(&mut buffer, &ctx).expect("filtre appliqué");
    buffer.pixel(0, 0).expect("pixel présent")
}
