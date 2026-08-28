//! Pont interne entre le cœur et la bibliothèque de codecs.
//!
//! Ce module est le **seul** endroit du crate qui connaît le type
//! `image::RgbaImage`. Les filtres et les étapes du pipeline ne manipulent que
//! [`PixelBuffer`] : changer de bibliothèque de décodage n'impacterait que ce
//! fichier.

use std::io::Cursor;

use image::codecs::bmp::BmpEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::codecs::tiff::TiffEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType as ResizeFilter;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage};

use crate::model::error::{FiltroError, Result};
use crate::model::pipeline::formatter::{FormatOptions, OutputFormat};
use crate::model::pixel::{Dimensions, PixelBuffer};

/// Décode des octets bruts vers un tampon RGBA8 et renvoie le format détecté.
pub(crate) fn decode(bytes: &[u8], origin: &str) -> Result<(PixelBuffer, Option<ImageFormat>)> {
    let format = image::guess_format(bytes).ok();
    let decoded = image::load_from_memory(bytes).map_err(|source| FiltroError::Decode {
        origin: origin.to_owned(),
        source,
    })?;
    let rgba = decoded.into_rgba8();
    let dimensions = Dimensions::new(rgba.width(), rgba.height())?;
    let buffer = PixelBuffer::from_raw(dimensions, rgba.into_raw())?;
    Ok((buffer, format))
}

/// Ré-échantillonne un tampon (Lanczos3).
///
/// Attention : il s'agit du ré-échantillonnage interne du pipeline, sans aucun
/// rapport avec les filtres applicatifs du projet.
pub(crate) fn resize(buffer: &PixelBuffer, target: Dimensions) -> Result<PixelBuffer> {
    if buffer.dimensions() == target {
        return Ok(buffer.clone());
    }
    let source = to_rgba_image(buffer)?;
    let resized =
        image::imageops::resize(&source, target.width, target.height, ResizeFilter::Lanczos3);
    PixelBuffer::from_raw(target, resized.into_raw())
}

/// Encode un tampon vers les octets d'un fichier image.
pub(crate) fn encode(
    buffer: &PixelBuffer,
    format: OutputFormat,
    options: &FormatOptions,
) -> Result<Vec<u8>> {
    let width = buffer.width();
    let height = buffer.height();
    let mut out: Vec<u8> = Vec::new();
    let wrap = |source: image::ImageError| FiltroError::Encode {
        format: format.to_string(),
        source,
    };

    match format {
        OutputFormat::Png => {
            let encoder = PngEncoder::new_with_quality(
                &mut out,
                CompressionType::Default,
                PngFilterType::Adaptive,
            );
            encoder
                .write_image(buffer.as_bytes(), width, height, ExtendedColorType::Rgba8)
                .map_err(wrap)?;
        }
        OutputFormat::Jpeg => {
            // JPEG ne porte pas d'alpha : le Formatter a déjà aplati l'image.
            let rgb = to_rgb_bytes(buffer);
            JpegEncoder::new_with_quality(&mut out, options.quality)
                .encode(&rgb, width, height, ExtendedColorType::Rgb8)
                .map_err(wrap)?;
        }
        OutputFormat::Bmp => {
            let rgb = to_rgb_bytes(buffer);
            let mut cursor = Cursor::new(&mut out);
            BmpEncoder::new(&mut cursor)
                .encode(&rgb, width, height, ExtendedColorType::Rgb8)
                .map_err(wrap)?;
        }
        OutputFormat::Tiff => {
            let cursor = Cursor::new(&mut out);
            TiffEncoder::new(cursor)
                .write_image(buffer.as_bytes(), width, height, ExtendedColorType::Rgba8)
                .map_err(wrap)?;
        }
        OutputFormat::WebP => {
            // Encodeur sans perte : le paramètre `quality` est ignoré ici.
            WebPEncoder::new_lossless(&mut out)
                .write_image(buffer.as_bytes(), width, height, ExtendedColorType::Rgba8)
                .map_err(wrap)?;
        }
    }

    Ok(out)
}

fn to_rgba_image(buffer: &PixelBuffer) -> Result<RgbaImage> {
    RgbaImage::from_raw(buffer.width(), buffer.height(), buffer.as_bytes().to_vec()).ok_or_else(
        || FiltroError::InvalidImage("tampon incompatible avec les dimensions".to_owned()),
    )
}

fn to_rgb_bytes(buffer: &PixelBuffer) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(buffer.as_bytes().len() / 4 * 3);
    for chunk in buffer.as_bytes().as_chunks::<4>().0 {
        rgb.extend_from_slice(&chunk[..3]);
    }
    rgb
}
