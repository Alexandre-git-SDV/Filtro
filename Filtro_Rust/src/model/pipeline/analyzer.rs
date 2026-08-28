//! Étape 1 — **Analyzer**.
//!
//! Lit l'image d'entrée, la décode en RGBA8 et produit une description
//! exploitable par le reste de la chaîne : dimensions, résolution, format
//! source, caractéristiques colorimétriques.
//!
//! L'Analyzer ne connaît aucun filtre.

use std::path::{Path, PathBuf};

use crate::model::codec;
use crate::model::error::{FiltroError, Result};
use crate::model::pixel::{Dimensions, PixelBuffer};
use crate::model::resolution::{self, Resolution};

/// Format du fichier source tel que détecté à la lecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Tiff,
    /// Format reconnu par le décodeur mais hors de la liste ci-dessus.
    Other(String),
    /// Format indéterminé (flux brut, en-tête tronqué…).
    Unknown,
}

impl SourceFormat {
    fn from_image_format(format: Option<image::ImageFormat>) -> Self {
        match format {
            Some(image::ImageFormat::Png) => Self::Png,
            Some(image::ImageFormat::Jpeg) => Self::Jpeg,
            Some(image::ImageFormat::Gif) => Self::Gif,
            Some(image::ImageFormat::WebP) => Self::WebP,
            Some(image::ImageFormat::Bmp) => Self::Bmp,
            Some(image::ImageFormat::Tiff) => Self::Tiff,
            Some(other) => Self::Other(format!("{other:?}").to_lowercase()),
            None => Self::Unknown,
        }
    }
}

impl std::fmt::Display for SourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png => f.write_str("png"),
            Self::Jpeg => f.write_str("jpeg"),
            Self::Gif => f.write_str("gif"),
            Self::WebP => f.write_str("webp"),
            Self::Bmp => f.write_str("bmp"),
            Self::Tiff => f.write_str("tiff"),
            Self::Other(name) => f.write_str(name),
            Self::Unknown => f.write_str("inconnu"),
        }
    }
}

/// Provenance de l'image analysée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Fichier sur disque.
    Path(PathBuf),
    /// Flux mémoire (entrée standard, test, appel bibliothèque).
    Memory { label: String },
}

impl Origin {
    /// Radical utilisable pour nommer les fichiers de sortie.
    pub fn stem(&self) -> String {
        match self {
            Self::Path(path) => path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "image".to_owned()),
            Self::Memory { label } => label.clone(),
        }
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Memory { label } => write!(f, "<{label}>"),
        }
    }
}

/// Statistiques calculées en un seul passage sur les pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStats {
    /// Vrai si au moins un pixel n'est pas opaque.
    pub has_transparency: bool,
    /// Vrai si tous les pixels vérifient r == g == b.
    pub is_grayscale: bool,
    /// Luminance moyenne (0.0 – 255.0).
    pub mean_luminance: f32,
    /// Luminance minimale et maximale observées.
    pub luminance_range: (u8, u8),
}

/// Description de l'image, sans les pixels.
///
/// C'est cette structure — légère et clonable — qui circule dans les contextes
/// de filtre et dans les journaux.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageMetadata {
    pub origin: Origin,
    pub source_format: SourceFormat,
    pub dimensions: Dimensions,
    pub resolution: Option<Resolution>,
    pub byte_size: usize,
    pub stats: ColorStats,
}

/// Résultat complet de l'analyse : description + pixels décodés.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageAnalysis {
    pub metadata: ImageMetadata,
    pub buffer: PixelBuffer,
}

impl ImageAnalysis {
    /// Sépare la description des pixels ; consommé par le Constructor.
    pub fn into_parts(self) -> (ImageMetadata, PixelBuffer) {
        (self.metadata, self.buffer)
    }
}

/// Étape d'analyse.
#[derive(Debug, Clone)]
pub struct Analyzer {
    /// Garde-fou contre les images décompressées démesurées (bombes de décompression).
    max_pixels: Option<u64>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            // 100 Mpx ≈ 400 Mio en RGBA8.
            max_pixels: Some(100_000_000),
        }
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Modifie la limite de pixels acceptés (`None` = aucune limite).
    pub fn with_max_pixels(mut self, max_pixels: Option<u64>) -> Self {
        self.max_pixels = max_pixels;
        self
    }

    /// Analyse un fichier.
    pub fn analyze_path(&self, path: impl AsRef<Path>) -> Result<ImageAnalysis> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| FiltroError::io(path, source))?;
        self.analyze_bytes(&bytes, Origin::Path(path.to_path_buf()))
    }

    /// Analyse un flux déjà en mémoire.
    pub fn analyze_bytes(&self, bytes: &[u8], origin: Origin) -> Result<ImageAnalysis> {
        if bytes.is_empty() {
            return Err(FiltroError::InvalidImage(format!("flux vide : {origin}")));
        }

        let (buffer, detected) = codec::decode(bytes, &origin.to_string())?;
        let dimensions = buffer.dimensions();

        if let Some(limit) = self.max_pixels
            && dimensions.pixel_count() > limit
        {
            return Err(FiltroError::InvalidImage(format!(
                "image de {dimensions} ({} pixels) au-delà de la limite de {limit} pixels",
                dimensions.pixel_count()
            )));
        }

        let source_format = SourceFormat::from_image_format(detected);
        let resolution = resolution::read(bytes, &source_format);
        let stats = compute_stats(&buffer);

        Ok(ImageAnalysis {
            metadata: ImageMetadata {
                origin,
                source_format,
                dimensions,
                resolution,
                byte_size: bytes.len(),
                stats,
            },
            buffer,
        })
    }
}

fn compute_stats(buffer: &PixelBuffer) -> ColorStats {
    let mut has_transparency = false;
    let mut is_grayscale = true;
    let mut sum = 0.0f64;
    let mut min = u8::MAX;
    let mut max = u8::MIN;

    for pixel in buffer.pixels() {
        if pixel.a != 255 {
            has_transparency = true;
        }
        if pixel.r != pixel.g || pixel.g != pixel.b {
            is_grayscale = false;
        }
        let luma = pixel.luminance();
        sum += f64::from(luma);
        min = min.min(luma);
        max = max.max(luma);
    }

    let count = buffer.dimensions().pixel_count().max(1) as f64;
    ColorStats {
        has_transparency,
        is_grayscale,
        mean_luminance: (sum / count) as f32,
        luminance_range: (min, max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pixel::Rgba8;

    fn png_bytes(width: u32, height: u32, pixel: Rgba8) -> Vec<u8> {
        let dims = Dimensions::new(width, height).unwrap();
        let mut buffer = PixelBuffer::new(dims).unwrap();
        buffer.map_pixels(|_, _, _| pixel);
        crate::model::codec::encode(
            &buffer,
            crate::model::pipeline::formatter::OutputFormat::Png,
            &crate::model::pipeline::formatter::FormatOptions::default(),
        )
        .expect("encodage png")
    }

    #[test]
    fn analyse_une_image_png() {
        let bytes = png_bytes(4, 2, Rgba8::new(200, 200, 200, 255));
        let analyzer = Analyzer::new();
        let analysis = analyzer
            .analyze_bytes(
                &bytes,
                Origin::Memory {
                    label: "test".into(),
                },
            )
            .expect("analyse");

        assert_eq!(analysis.metadata.source_format, SourceFormat::Png);
        assert_eq!(analysis.metadata.dimensions.width, 4);
        assert_eq!(analysis.metadata.dimensions.height, 2);
        assert!(analysis.metadata.stats.is_grayscale);
        assert!(!analysis.metadata.stats.has_transparency);
    }

    #[test]
    fn refuse_un_flux_vide() {
        let analyzer = Analyzer::new();
        let err = analyzer
            .analyze_bytes(
                &[],
                Origin::Memory {
                    label: "vide".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, FiltroError::InvalidImage(_)));
    }

    #[test]
    fn respecte_la_limite_de_pixels() {
        let bytes = png_bytes(8, 8, Rgba8::BLACK);
        let analyzer = Analyzer::new().with_max_pixels(Some(16));
        assert!(
            analyzer
                .analyze_bytes(
                    &bytes,
                    Origin::Memory {
                        label: "gros".into()
                    }
                )
                .is_err()
        );
    }
}
