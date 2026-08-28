//! Filtres de couleur : `jaune`, `vert`, `rouge`.
//!
//! Les trois filtres font la même chose à deux constantes près :
//!
//! ```text
//! si r > niveau_blanc et v > niveau_blanc et b > niveau_blanc :
//!     pixel <- (254, 254, 1)     // le blanc devient une couleur
//! sinon :
//!     pixel <- (b, v, r)         // les autres pixels sont permutés
//! ```
//!
//! Un seul type Rust couvre donc les trois filtres : seules la couleur de
//! remplacement et la permutation par défaut changent.

use crate::Result;
use crate::{Filter, FilterContext};
use crate::{FilterFactory, FilterParams, ParamSpec};
use crate::{PixelBuffer, Rgba8};

// ---------------------------------------------------------------------------
// Permutation des canaux
// ---------------------------------------------------------------------------

/// Un canal source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    R,
    V,
    B,
}

/// Réaffectation des trois canaux, décrite par un mot de trois lettres.
///
/// `bvr` signifie : « le rouge de sortie prend le bleu d'entrée, le vert garde
/// le vert, le bleu prend le rouge » — soit le pixel `(b, v, r)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelOrder([Channel; 3]);

impl ChannelOrder {
    /// Aucune permutation (`rvb`).
    pub const IDENTITY: Self = Self([Channel::R, Channel::V, Channel::B]);

    /// Analyse un mot de trois lettres parmi `r`, `v`/`g`, `b`.
    pub fn parse(raw: &str) -> Option<Self> {
        let letters: Vec<Channel> = raw
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|c| match c {
                'r' => Some(Channel::R),
                'v' | 'g' => Some(Channel::V),
                'b' => Some(Channel::B),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let channels: [Channel; 3] = letters.try_into().ok()?;
        Some(Self(channels))
    }

    /// Applique la permutation ; l'alpha est toujours préservé.
    pub fn apply(self, pixel: Rgba8) -> Rgba8 {
        let take = |channel: Channel| match channel {
            Channel::R => pixel.r,
            Channel::V => pixel.g,
            Channel::B => pixel.b,
        };
        Rgba8::new(take(self.0[0]), take(self.0[1]), take(self.0[2]), pixel.a)
    }
}

// ---------------------------------------------------------------------------
// Le filtre
// ---------------------------------------------------------------------------

/// Remplace les pixels clairs par une couleur, permute les autres.
#[derive(Debug, Clone)]
pub struct ColorFilter {
    id: &'static str,
    /// Seuil au-delà duquel un pixel est considéré comme blanc (`niveau_blanc`).
    level: u8,
    /// Couleur qui remplace le blanc.
    color: Rgba8,
    /// Permutation appliquée aux pixels non blancs.
    order: ChannelOrder,
}

impl ColorFilter {
    /// Vrai si les trois composantes dépassent le seuil.
    fn is_white(&self, pixel: Rgba8) -> bool {
        pixel.r > self.level && pixel.g > self.level && pixel.b > self.level
    }
}

impl Filter for ColorFilter {
    fn id(&self) -> &str {
        self.id
    }

    fn apply(&self, canvas: &mut PixelBuffer, _ctx: &FilterContext<'_>) -> Result<()> {
        canvas.map_pixels(|_, _, pixel| {
            if self.is_white(pixel) {
                Rgba8 {
                    a: pixel.a,
                    ..self.color
                }
            } else {
                self.order.apply(pixel)
            }
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Fabrique
// ---------------------------------------------------------------------------

/// Fabrique commune aux trois filtres de couleur.
///
/// Chaque filtre n'est qu'un jeu de valeurs par défaut.
#[derive(Debug, Clone, Copy)]
pub struct ColorFilterFactory {
    id: &'static str,
    description: &'static str,
    color: Rgba8,
    order: &'static str,
    params: &'static [ParamSpec],
}

impl FilterFactory for ColorFilterFactory {
    fn id(&self) -> &'static str {
        self.id
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn parameters(&self) -> &'static [ParamSpec] {
        self.params
    }

    fn build(&self, params: &FilterParams) -> Result<Box<dyn Filter>> {
        let level = params.u8("niveau", 180)?;
        let color = params.color("couleur", self.color)?;
        let raw_order = params.text("permutation", self.order);
        let order =
            ChannelOrder::parse(raw_order).ok_or_else(|| crate::FiltroError::InvalidParameter {
                filter: self.id.to_owned(),
                name: "permutation".to_owned(),
                reason: format!("« {raw_order} » n'est pas un mot de trois lettres parmi r, v, b"),
            })?;

        Ok(Box::new(ColorFilter {
            id: self.id,
            level,
            color,
            order,
        }))
    }
}

/// Construit la liste de paramètres d'un filtre de couleur.
macro_rules! color_params {
    ($couleur:expr, $permutation:expr) => {
        &[
            ParamSpec {
                name: "niveau",
                default: "180",
                help: "seuil (0-255) au-delà duquel un pixel est vu comme blanc",
            },
            ParamSpec {
                name: "couleur",
                default: $couleur,
                help: "couleur hexadécimale remplaçant le blanc",
            },
            ParamSpec {
                name: "permutation",
                default: $permutation,
                help: "réaffectation des canaux des autres pixels (ex. bvr)",
            },
        ]
    };
}

/// Filtre `jaune` — blanc → jaune, canaux permutés en `bvr`.
pub fn yellow() -> ColorFilterFactory {
    ColorFilterFactory {
        id: "jaune",
        description: "Remplace le blanc par du jaune et échange rouge et bleu",
        color: Rgba8::new(254, 254, 1, 255),
        order: "bvr",
        params: color_params!("fefe01", "bvr"),
    }
}

/// Filtre `vert` — blanc → vert, canaux permutés en `rbv`.
pub fn green() -> ColorFilterFactory {
    ColorFilterFactory {
        id: "vert",
        description: "Remplace le blanc par du vert et échange vert et bleu",
        color: Rgba8::new(145, 254, 1, 255),
        order: "rbv",
        params: color_params!("91fe01", "rbv"),
    }
}

/// Filtre `rouge` — blanc → rouge, canaux permutés en `brv`.
pub fn red() -> ColorFilterFactory {
    ColorFilterFactory {
        id: "rouge",
        description: "Remplace le blanc par du rouge et fait tourner les canaux",
        color: Rgba8::new(254, 1, 1, 255),
        order: "brv",
        params: color_params!("fe0101", "brv"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::apply_to_pixel;
    use crate::{FilterRegistry, FilterRequest};

    #[test]
    fn permutation_bvr_echange_rouge_et_bleu() {
        let order = ChannelOrder::parse("bvr").unwrap();
        let out = order.apply(Rgba8::new(10, 20, 30, 200));
        assert_eq!(out, Rgba8::new(30, 20, 10, 200));
    }

    #[test]
    fn permutation_invalide() {
        assert!(ChannelOrder::parse("bv").is_none());
        assert!(ChannelOrder::parse("xyz").is_none());
        assert_eq!(ChannelOrder::parse("rgb"), Some(ChannelOrder::IDENTITY));
    }

    fn filtre(expression: &str) -> Box<dyn Filter> {
        let mut registry = FilterRegistry::new();
        registry.register(yellow()).unwrap();
        registry.register(green()).unwrap();
        registry.register(red()).unwrap();
        registry
            .build(&FilterRequest::parse(expression).unwrap())
            .unwrap()
    }

    #[test]
    fn le_blanc_devient_jaune() {
        let sortie = apply_to_pixel(&*filtre("jaune"), Rgba8::new(200, 210, 220, 255));
        assert_eq!(sortie, Rgba8::new(254, 254, 1, 255));
    }

    #[test]
    fn les_autres_pixels_sont_permutes() {
        // 100 < 180 : le pixel n'est pas « blanc », il est donc permuté.
        let sortie = apply_to_pixel(&*filtre("jaune"), Rgba8::new(10, 100, 30, 255));
        assert_eq!(sortie, Rgba8::new(30, 100, 10, 255));
    }

    #[test]
    fn le_seuil_est_reglable() {
        let clair = Rgba8::new(200, 200, 200, 255);
        // Avec un seuil à 220, ce pixel n'est plus considéré comme blanc.
        let sortie = apply_to_pixel(&*filtre("jaune:niveau=220"), clair);
        assert_eq!(sortie, clair); // permutation bvr sur un gris = identique
    }

    #[test]
    fn la_couleur_est_reglable() {
        let sortie = apply_to_pixel(&*filtre("rouge:couleur=00ccff"), Rgba8::WHITE);
        assert_eq!(sortie, Rgba8::new(0, 204, 255, 255));
    }

    #[test]
    fn l_alpha_est_preserve() {
        let sortie = apply_to_pixel(&*filtre("vert"), Rgba8::new(250, 250, 250, 64));
        assert_eq!(sortie.a, 64);
    }

    #[test]
    fn permutation_invalide_rejetee_a_la_construction() {
        let mut registry = FilterRegistry::new();
        registry.register(yellow()).unwrap();
        let request = FilterRequest::parse("jaune:permutation=xx").unwrap();
        assert!(registry.build(&request).is_err());
    }
}
