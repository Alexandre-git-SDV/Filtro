//! Étape 3 — **Formatter**.
//!
//! Décline l'image préparée en variantes : combinaisons (format de sortie ×
//! déclinaison dimensionnelle). Chaque variante possède son propre tampon,
//! déjà adapté aux contraintes du format (aplatissement de l'alpha pour JPEG
//! et BMP, par exemple).
//!
//! Le Formatter ne connaît **aucun filtre** : il prépare des supports, le
//! Renderer y appliquera la chaîne.

use crate::model::codec;
use crate::model::error::{FiltroError, Result};
use crate::model::pipeline::constructor::PreparedImage;
use crate::model::pixel::{Dimensions, PixelBuffer, Rgba8};

/// Format de fichier proposé en sortie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    Png,
    Jpeg,
    WebP,
    Bmp,
    Tiff,
}

impl OutputFormat {
    /// Tous les formats gérés, dans l'ordre d'affichage.
    pub const ALL: [OutputFormat; 5] = [
        OutputFormat::Png,
        OutputFormat::Jpeg,
        OutputFormat::WebP,
        OutputFormat::Bmp,
        OutputFormat::Tiff,
    ];

    /// Extension de fichier, sans point.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    /// Vrai si le format transporte un canal alpha.
    pub fn supports_alpha(self) -> bool {
        matches!(self, Self::Png | Self::WebP | Self::Tiff)
    }

    /// Vrai si le format est compressé avec perte (le réglage qualité s'applique).
    pub fn is_lossy(self) -> bool {
        matches!(self, Self::Jpeg)
    }

    /// Analyse un nom de format (`png`, `jpg`, `jpeg`, `webp`, `bmp`, `tiff`…).
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "webp" => Ok(Self::WebP),
            "bmp" => Ok(Self::Bmp),
            "tif" | "tiff" => Ok(Self::Tiff),
            other => Err(FiltroError::UnknownFormat(other.to_owned())),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        })
    }
}

/// Réglages d'encodage communs à une variante.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormatOptions {
    /// Qualité 1–100 pour les formats avec perte.
    pub quality: u8,
    /// Fond appliqué lorsque le format ne gère pas la transparence.
    pub background: Rgba8,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            quality: 85,
            background: Rgba8::WHITE,
        }
    }
}

/// Déclinaison dimensionnelle demandée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeVariant {
    /// Dimensions de l'image de travail.
    Original,
    /// Réduction pour tenir dans une boîte carrée de `max_side` pixels.
    Bounded { label: String, max_side: u32 },
}

impl SizeVariant {
    pub fn label(&self) -> &str {
        match self {
            Self::Original => "original",
            Self::Bounded { label, .. } => label,
        }
    }

    /// Analyse une expression `étiquette:côté`, par exemple `vignette:256`.
    pub fn parse(expression: &str) -> Result<Self> {
        let (label, side) = expression.split_once(':').ok_or_else(|| {
            FiltroError::Config(format!(
                "déclinaison « {expression} » mal formée (attendu étiquette:pixels)"
            ))
        })?;
        let label = label.trim();
        if label.is_empty() {
            return Err(FiltroError::Config(
                "étiquette de déclinaison vide".to_owned(),
            ));
        }
        let max_side: u32 = side.trim().parse().map_err(|_| {
            FiltroError::Config(format!("« {} » n'est pas un nombre de pixels", side.trim()))
        })?;
        if max_side == 0 {
            return Err(FiltroError::Config(
                "la taille d'une déclinaison doit être supérieure à zéro".to_owned(),
            ));
        }
        Ok(Self::Bounded {
            label: label.to_owned(),
            max_side,
        })
    }
}

/// Identité d'une variante de sortie, transmise aux filtres via le contexte.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputTarget {
    pub format: OutputFormat,
    pub size: SizeVariant,
    pub dimensions: Dimensions,
    pub options: FormatOptions,
}

impl OutputTarget {
    /// Nom de fichier construit à partir d'un radical.
    pub fn file_name(&self, stem: &str) -> String {
        match &self.size {
            SizeVariant::Original => format!("{stem}.{}", self.format.extension()),
            SizeVariant::Bounded { label, .. } => {
                format!("{stem}-{label}.{}", self.format.extension())
            }
        }
    }
}

/// Une variante prête à être filtrée puis encodée.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedImage {
    pub target: OutputTarget,
    pub canvas: PixelBuffer,
}

/// Ensemble des variantes issues d'une même image préparée.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatSet {
    pub variants: Vec<FormattedImage>,
}

impl FormatSet {
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }
}

/// Étape de déclinaison.
#[derive(Debug, Clone)]
pub struct Formatter {
    formats: Vec<OutputFormat>,
    sizes: Vec<SizeVariant>,
    options: FormatOptions,
}

impl Default for Formatter {
    fn default() -> Self {
        Self {
            formats: vec![OutputFormat::Png],
            sizes: vec![SizeVariant::Original],
            options: FormatOptions::default(),
        }
    }
}

impl Formatter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Définit les formats produits (au moins un).
    pub fn with_formats(mut self, formats: Vec<OutputFormat>) -> Result<Self> {
        if formats.is_empty() {
            return Err(FiltroError::Config(
                "au moins un format de sortie est requis".to_owned(),
            ));
        }
        self.formats = formats;
        self.formats.dedup();
        Ok(self)
    }

    /// Définit les déclinaisons dimensionnelles (`Original` est ajouté si la liste est vide).
    pub fn with_sizes(mut self, sizes: Vec<SizeVariant>) -> Self {
        self.sizes = if sizes.is_empty() {
            vec![SizeVariant::Original]
        } else {
            sizes
        };
        self
    }

    /// Définit la qualité d'encodage des formats avec perte.
    pub fn with_quality(mut self, quality: u8) -> Result<Self> {
        if !(1..=100).contains(&quality) {
            return Err(FiltroError::Config(format!(
                "qualité {quality} hors de l'intervalle 1–100"
            )));
        }
        self.options.quality = quality;
        Ok(self)
    }

    pub fn formats(&self) -> &[OutputFormat] {
        &self.formats
    }

    pub fn sizes(&self) -> &[SizeVariant] {
        &self.sizes
    }

    /// Produit toutes les variantes (déclinaison × format).
    ///
    /// Le ré-échantillonnage n'est calculé qu'une fois par déclinaison, puis
    /// partagé entre les formats.
    pub fn format(&self, prepared: &PreparedImage) -> Result<FormatSet> {
        let mut variants = Vec::with_capacity(self.sizes.len() * self.formats.len());
        let mut options = self.options;
        options.background = prepared.background;

        for size in &self.sizes {
            let canvas = match size {
                SizeVariant::Original => prepared.canvas.clone(),
                SizeVariant::Bounded { max_side, .. } => {
                    match prepared.canvas.dimensions().scaled_to_fit(*max_side) {
                        Some(target) => codec::resize(&prepared.canvas, target)?,
                        None => prepared.canvas.clone(),
                    }
                }
            };
            let dimensions = canvas.dimensions();

            for format in &self.formats {
                let mut variant_canvas = canvas.clone();
                if !format.supports_alpha() {
                    variant_canvas.flatten_onto(options.background);
                }
                variants.push(FormattedImage {
                    target: OutputTarget {
                        format: *format,
                        size: size.clone(),
                        dimensions,
                        options,
                    },
                    canvas: variant_canvas,
                });
            }
        }

        Ok(FormatSet { variants })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::prepared;

    #[test]
    fn produit_le_produit_cartesien() {
        let formatter = Formatter::new()
            .with_formats(vec![OutputFormat::Png, OutputFormat::Jpeg])
            .unwrap()
            .with_sizes(vec![
                SizeVariant::Original,
                SizeVariant::Bounded {
                    label: "vignette".into(),
                    max_side: 50,
                },
            ]);

        let set = formatter
            .format(&prepared(200, 100, Rgba8::new(10, 20, 30, 255)))
            .unwrap();

        assert_eq!(set.len(), 4);
        let vignette = set
            .variants
            .iter()
            .find(|v| v.target.size.label() == "vignette")
            .unwrap();
        assert_eq!(vignette.target.dimensions.width, 50);
        assert_eq!(vignette.canvas.width(), 50);
    }

    #[test]
    fn aplatit_l_alpha_pour_les_formats_sans_transparence() {
        let formatter = Formatter::new()
            .with_formats(vec![OutputFormat::Png, OutputFormat::Jpeg])
            .unwrap();

        let set = formatter
            .format(&prepared(4, 4, Rgba8::new(10, 20, 30, 0)))
            .unwrap();

        for variant in &set.variants {
            assert_eq!(
                variant.canvas.has_transparency(),
                variant.target.format.supports_alpha()
            );
        }
    }

    #[test]
    fn nomme_les_fichiers() {
        let formatter = Formatter::new().with_sizes(vec![SizeVariant::Bounded {
            label: "web".into(),
            max_side: 800,
        }]);
        let set = formatter
            .format(&prepared(1600, 900, Rgba8::BLACK))
            .unwrap();
        assert_eq!(set.variants[0].target.file_name("photo"), "photo-web.png");
    }

    #[test]
    fn refuse_une_qualite_invalide() {
        assert!(Formatter::new().with_quality(0).is_err());
        assert!(Formatter::new().with_quality(101).is_err());
    }
}
