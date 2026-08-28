//! Représentation mémoire des pixels, indépendante de toute bibliothèque tierce.
//!
//! C'est le seul type de données que les filtres manipulent. Il ne dépend
//! volontairement pas du crate `image` : la bibliothèque de décodage peut être
//! remplacée sans casser le contrat public offert aux filtres.

use crate::model::error::{FiltroError, Result};

/// Nombre d'octets par pixel (RGBA 8 bits, alpha non prémultiplié).
pub const CHANNELS: usize = 4;

/// Un pixel RGBA 8 bits, alpha non prémultiplié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    /// Pixel entièrement transparent.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    /// Blanc opaque (fond par défaut lors de l'aplatissement de l'alpha).
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    /// Noir opaque.
    pub const BLACK: Self = Self::new(0, 0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Analyse une couleur hexadécimale : `fefe01`, `#fefe01`, `fefe01ff`.
    ///
    /// Renvoie `None` si la chaîne n'est pas une couleur valide.
    pub fn from_hex(raw: &str) -> Option<Self> {
        let hex = raw.trim().trim_start_matches('#');
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        match hex.len() {
            6 => Some(Self::new(byte(0)?, byte(2)?, byte(4)?, 255)),
            8 => Some(Self::new(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        }
    }

    /// Luminance perçue (Rec. 601), utile aux filtres de seuillage.
    pub fn luminance(self) -> u8 {
        let y = 0.299 * f32::from(self.r) + 0.587 * f32::from(self.g) + 0.114 * f32::from(self.b);
        y.round().clamp(0.0, 255.0) as u8
    }

    /// Composite ce pixel sur un fond opaque et renvoie le résultat opaque.
    pub fn over(self, background: Rgba8) -> Rgba8 {
        if self.a == 255 {
            return self;
        }
        let alpha = f32::from(self.a) / 255.0;
        let mix = |fg: u8, bg: u8| -> u8 {
            let value = f32::from(fg) * alpha + f32::from(bg) * (1.0 - alpha);
            value.round().clamp(0.0, 255.0) as u8
        };
        Rgba8::new(
            mix(self.r, background.r),
            mix(self.g, background.g),
            mix(self.b, background.b),
            255,
        )
    }
}

/// Dimensions d'une image, toujours strictement positives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(FiltroError::InvalidImage(format!(
                "dimensions nulles ({width}x{height})"
            )));
        }
        Ok(Self { width, height })
    }

    /// Nombre total de pixels, calculé en `u64` pour éviter tout débordement.
    pub fn pixel_count(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// Plus grand côté, base des variantes dimensionnelles du Formatter.
    pub fn longest_side(self) -> u32 {
        self.width.max(self.height)
    }

    /// Dimensions réduites pour tenir dans une boîte de `max_side` pixels.
    /// Renvoie `None` si l'image tient déjà dans la boîte.
    pub fn scaled_to_fit(self, max_side: u32) -> Option<Dimensions> {
        if max_side == 0 || self.longest_side() <= max_side {
            return None;
        }
        let ratio = f64::from(max_side) / f64::from(self.longest_side());
        let width = ((f64::from(self.width) * ratio).round() as u32).max(1);
        let height = ((f64::from(self.height) * ratio).round() as u32).max(1);
        Some(Dimensions { width, height })
    }
}

impl std::fmt::Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Tampon de pixels RGBA8 contigu, ligne par ligne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    dimensions: Dimensions,
    data: Vec<u8>,
}

impl PixelBuffer {
    /// Crée un tampon entièrement transparent.
    pub fn new(dimensions: Dimensions) -> Result<Self> {
        let len = usize::try_from(dimensions.pixel_count())
            .ok()
            .and_then(|count| count.checked_mul(CHANNELS))
            .ok_or_else(|| {
                FiltroError::InvalidImage(format!("image trop grande : {dimensions}"))
            })?;
        Ok(Self {
            dimensions,
            data: vec![0; len],
        })
    }

    /// Construit un tampon à partir d'octets bruts RGBA8.
    ///
    /// # Erreurs
    /// Renvoie [`FiltroError::InvalidImage`] si la taille du tampon ne
    /// correspond pas aux dimensions annoncées.
    pub fn from_raw(dimensions: Dimensions, data: Vec<u8>) -> Result<Self> {
        let expected = usize::try_from(dimensions.pixel_count())
            .ok()
            .and_then(|count| count.checked_mul(CHANNELS))
            .ok_or_else(|| {
                FiltroError::InvalidImage(format!("image trop grande : {dimensions}"))
            })?;
        if data.len() != expected {
            return Err(FiltroError::InvalidImage(format!(
                "tampon de {} octets pour {dimensions} (attendu : {expected})",
                data.len()
            )));
        }
        Ok(Self { dimensions, data })
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    pub fn width(&self) -> u32 {
        self.dimensions.width
    }

    pub fn height(&self) -> u32 {
        self.dimensions.height
    }

    /// Accès en lecture aux octets bruts (RGBA entrelacés).
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Accès en écriture aux octets bruts, pour les filtres vectorisés.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consomme le tampon et rend les octets bruts.
    pub fn into_raw(self) -> Vec<u8> {
        self.data
    }

    fn offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.dimensions.width || y >= self.dimensions.height {
            return None;
        }
        let index = u64::from(y) * u64::from(self.dimensions.width) + u64::from(x);
        usize::try_from(index).ok()?.checked_mul(CHANNELS)
    }

    /// Lit un pixel, ou `None` hors limites.
    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgba8> {
        let offset = self.offset(x, y)?;
        Some(Rgba8::new(
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ))
    }

    /// Lit un pixel en repliant les coordonnées sur le bord le plus proche.
    /// Pratique pour les noyaux de convolution.
    pub fn pixel_clamped(&self, x: i64, y: i64) -> Rgba8 {
        let cx = x.clamp(0, i64::from(self.dimensions.width - 1)) as u32;
        let cy = y.clamp(0, i64::from(self.dimensions.height - 1)) as u32;
        self.pixel(cx, cy).unwrap_or(Rgba8::TRANSPARENT)
    }

    /// Écrit un pixel ; sans effet si les coordonnées sortent de l'image.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: Rgba8) {
        if let Some(offset) = self.offset(x, y) {
            self.data[offset] = pixel.r;
            self.data[offset + 1] = pixel.g;
            self.data[offset + 2] = pixel.b;
            self.data[offset + 3] = pixel.a;
        }
    }

    /// Itère sur les pixels dans l'ordre de lecture.
    pub fn pixels(&self) -> impl Iterator<Item = Rgba8> + '_ {
        self.data
            .as_chunks::<CHANNELS>()
            .0
            .iter()
            .map(|c| Rgba8::new(c[0], c[1], c[2], c[3]))
    }

    /// Applique une transformation pixel à pixel, en place.
    pub fn map_pixels<F>(&mut self, mut f: F)
    where
        F: FnMut(u32, u32, Rgba8) -> Rgba8,
    {
        let width = self.dimensions.width;
        for (index, chunk) in self
            .data
            .as_chunks_mut::<CHANNELS>()
            .0
            .iter_mut()
            .enumerate()
        {
            let index = index as u32;
            let x = index % width;
            let y = index / width;
            let out = f(x, y, Rgba8::new(chunk[0], chunk[1], chunk[2], chunk[3]));
            chunk[0] = out.r;
            chunk[1] = out.g;
            chunk[2] = out.b;
            chunk[3] = out.a;
        }
    }

    /// Vrai si au moins un pixel n'est pas totalement opaque.
    pub fn has_transparency(&self) -> bool {
        self.data
            .as_chunks::<CHANNELS>()
            .0
            .iter()
            .any(|c| c[3] != 255)
    }

    /// Aplatit la transparence sur un fond opaque (formats sans canal alpha).
    pub fn flatten_onto(&mut self, background: Rgba8) {
        if !self.has_transparency() {
            return;
        }
        self.map_pixels(|_, _, pixel| pixel.over(background));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(width: u32, height: u32) -> PixelBuffer {
        PixelBuffer::new(Dimensions::new(width, height).expect("dimensions valides"))
            .expect("allocation")
    }

    #[test]
    fn dimensions_refuse_zero() {
        assert!(Dimensions::new(0, 10).is_err());
        assert!(Dimensions::new(10, 0).is_err());
    }

    #[test]
    fn from_raw_verifie_la_taille() {
        let dims = Dimensions::new(2, 2).unwrap();
        assert!(PixelBuffer::from_raw(dims, vec![0; 16]).is_ok());
        assert!(PixelBuffer::from_raw(dims, vec![0; 15]).is_err());
    }

    #[test]
    fn lecture_ecriture_pixel() {
        let mut buf = buffer(3, 2);
        buf.set_pixel(2, 1, Rgba8::new(10, 20, 30, 40));
        assert_eq!(buf.pixel(2, 1), Some(Rgba8::new(10, 20, 30, 40)));
        assert_eq!(buf.pixel(3, 1), None);
    }

    #[test]
    fn map_pixels_fournit_les_coordonnees() {
        let mut buf = buffer(2, 2);
        buf.map_pixels(|x, y, _| Rgba8::new(x as u8, y as u8, 0, 255));
        assert_eq!(buf.pixel(1, 0), Some(Rgba8::new(1, 0, 0, 255)));
        assert_eq!(buf.pixel(0, 1), Some(Rgba8::new(0, 1, 0, 255)));
    }

    #[test]
    fn aplatissement_alpha() {
        let mut buf = buffer(1, 1);
        buf.set_pixel(0, 0, Rgba8::new(0, 0, 0, 0));
        buf.flatten_onto(Rgba8::WHITE);
        assert_eq!(buf.pixel(0, 0), Some(Rgba8::WHITE));
    }

    #[test]
    fn lecture_couleur_hexadecimale() {
        assert_eq!(
            Rgba8::from_hex("#fefe01"),
            Some(Rgba8::new(254, 254, 1, 255))
        );
        assert_eq!(Rgba8::from_hex("000000ff"), Some(Rgba8::BLACK));
        assert_eq!(Rgba8::from_hex("fff"), None);
        assert_eq!(Rgba8::from_hex("zzzzzz"), None);
    }

    #[test]
    fn reduction_proportionnelle() {
        let dims = Dimensions::new(1000, 500).unwrap();
        assert_eq!(dims.scaled_to_fit(2000), None);
        let reduced = dims.scaled_to_fit(100).unwrap();
        assert_eq!((reduced.width, reduced.height), (100, 50));
    }
}
