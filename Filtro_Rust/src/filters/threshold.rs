//! Filtre `noir-blanc` — seuillage binaire.
//!
//! ```text
//! si r > niveau_blanc et v > niveau_blanc et b > niveau_blanc :
//!     pixel <- (255, 255, 255)
//! sinon :
//!     pixel <- (0, 0, 0)
//! ```
//!
//! Le mode par défaut compare les trois composantes séparément. Le paramètre
//! `mode=luminance` propose la variante habituelle en traitement d'image :
//! comparer la luminance perçue du pixel.

use crate::{Filter, FilterContext};
use crate::{FilterFactory, FilterParams, ParamSpec};
use crate::{FiltroError, Result};
use crate::{PixelBuffer, Rgba8};

/// Manière de décider si un pixel est clair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdMode {
    /// Les trois composantes doivent dépasser le seuil (mode par défaut).
    Channels,
    /// La luminance perçue doit dépasser le seuil.
    Luminance,
}

impl ThresholdMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "canaux" | "channels" => Some(Self::Channels),
            "luminance" => Some(Self::Luminance),
            _ => None,
        }
    }

    fn is_light(self, pixel: Rgba8, level: u8) -> bool {
        match self {
            Self::Channels => pixel.r > level && pixel.g > level && pixel.b > level,
            Self::Luminance => pixel.luminance() > level,
        }
    }
}

/// Seuillage binaire : chaque pixel devient clair ou sombre.
#[derive(Debug, Clone)]
pub struct BlackAndWhiteFilter {
    level: u8,
    mode: ThresholdMode,
    light: Rgba8,
    dark: Rgba8,
}

impl Filter for BlackAndWhiteFilter {
    fn id(&self) -> &str {
        "noir-blanc"
    }

    fn apply(&self, canvas: &mut PixelBuffer, _ctx: &FilterContext<'_>) -> Result<()> {
        canvas.map_pixels(|_, _, pixel| {
            let chosen = if self.mode.is_light(pixel, self.level) {
                self.light
            } else {
                self.dark
            };
            Rgba8 {
                a: pixel.a,
                ..chosen
            }
        });
        Ok(())
    }
}

/// Fabrique du filtre `noir-blanc`.
#[derive(Debug, Clone, Copy)]
pub struct BlackAndWhiteFactory;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "niveau",
        default: "180",
        help: "seuil (0-255) séparant le clair du sombre",
    },
    ParamSpec {
        name: "mode",
        default: "canaux",
        help: "canaux (comparaison composante par composante) ou luminance",
    },
    ParamSpec {
        name: "clair",
        default: "ffffff",
        help: "couleur hexadécimale des pixels au-dessus du seuil",
    },
    ParamSpec {
        name: "sombre",
        default: "000000",
        help: "couleur hexadécimale des pixels en dessous du seuil",
    },
];

impl FilterFactory for BlackAndWhiteFactory {
    fn id(&self) -> &'static str {
        "noir-blanc"
    }

    fn description(&self) -> &'static str {
        "Seuillage binaire : chaque pixel devient clair ou sombre"
    }

    fn parameters(&self) -> &'static [ParamSpec] {
        PARAMS
    }

    fn build(&self, params: &FilterParams) -> Result<Box<dyn Filter>> {
        let raw_mode = params.text("mode", "canaux");
        let mode = ThresholdMode::parse(raw_mode).ok_or_else(|| FiltroError::InvalidParameter {
            filter: self.id().to_owned(),
            name: "mode".to_owned(),
            reason: format!("« {raw_mode} » inconnu (attendu : canaux ou luminance)"),
        })?;

        Ok(Box::new(BlackAndWhiteFilter {
            level: params.u8("niveau", 180)?,
            mode,
            light: params.color("clair", Rgba8::WHITE)?,
            dark: params.color("sombre", Rgba8::BLACK)?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::apply_to_pixel;
    use crate::{FilterRegistry, FilterRequest};

    fn filtre(expression: &str) -> Box<dyn Filter> {
        let mut registry = FilterRegistry::new();
        registry.register(BlackAndWhiteFactory).unwrap();
        registry
            .build(&FilterRequest::parse(expression).unwrap())
            .unwrap()
    }

    #[test]
    fn seuillage_par_canaux() {
        let f = filtre("noir-blanc");
        assert_eq!(
            apply_to_pixel(&*f, Rgba8::new(200, 200, 200, 255)),
            Rgba8::WHITE
        );
        assert_eq!(
            apply_to_pixel(&*f, Rgba8::new(200, 100, 200, 255)),
            Rgba8::BLACK
        );
    }

    #[test]
    fn seuillage_par_luminance() {
        // Vert vif : luminance ≈ 191, au-dessus du seuil de 180…
        let vert = Rgba8::new(100, 255, 100, 255);
        assert_eq!(
            apply_to_pixel(&*filtre("noir-blanc:mode=luminance"), vert),
            Rgba8::WHITE
        );
        // …alors qu'en mode « canaux » le rouge à 100 le fait basculer au noir.
        assert_eq!(apply_to_pixel(&*filtre("noir-blanc"), vert), Rgba8::BLACK);
    }

    #[test]
    fn couleurs_personnalisees() {
        let f = filtre("noir-blanc:clair=ffcc00,sombre=001133");
        assert_eq!(
            apply_to_pixel(&*f, Rgba8::WHITE),
            Rgba8::new(255, 204, 0, 255)
        );
        assert_eq!(
            apply_to_pixel(&*f, Rgba8::BLACK),
            Rgba8::new(0, 17, 51, 255)
        );
    }

    #[test]
    fn mode_inconnu_rejete() {
        let mut registry = FilterRegistry::new();
        registry.register(BlackAndWhiteFactory).unwrap();
        let request = FilterRequest::parse("noir-blanc:mode=sepia").unwrap();
        assert!(registry.build(&request).is_err());
    }
}
