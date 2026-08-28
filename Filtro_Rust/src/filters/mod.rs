//! Les filtres — **hors du cœur**.
//!
//! Ce module n'utilise que l'API publique de la bibliothèque
//! ([`Filter`](crate::Filter), [`FilterFactory`](crate::FilterFactory)).
//! Aucune étape du pipeline ne le connaît : il pourrait être déplacé dans un
//! crate séparé sans modifier une seule ligne d'Analyzer, Constructor,
//! Formatter ou Renderer.
//!
//! Filtres livrés avec le projet :
//!
//! | Identifiant | Effet |
//! |-------------|-------|
//! | `jaune` | blanc → jaune, canaux permutés en `bvr` |
//! | `vert` | blanc → vert, canaux permutés en `rbv` |
//! | `rouge` | blanc → rouge, canaux permutés en `brv` |
//! | `noir-blanc` | seuillage binaire |

pub mod color;
pub mod threshold;

use crate::FilterRegistry;
use crate::Result;

/// Enregistre tous les filtres fournis avec le projet.
///
/// C'est l'unique point de couplage entre l'application et les filtres :
/// ajouter un filtre se résume à une ligne ici.
pub fn register_all(registry: &mut FilterRegistry) -> Result<()> {
    registry.register(color::yellow())?;
    registry.register(color::green())?;
    registry.register(color::red())?;
    registry.register(threshold::BlackAndWhiteFactory)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tous_les_filtres_s_enregistrent() {
        let mut registry = FilterRegistry::new();
        register_all(&mut registry).unwrap();
        assert_eq!(registry.len(), 4);
        for id in ["jaune", "vert", "rouge", "noir-blanc"] {
            assert!(registry.get(id).is_some(), "filtre « {id} » absent");
        }
    }

    #[test]
    fn chaque_filtre_se_construit_sans_parametre() {
        let mut registry = FilterRegistry::new();
        register_all(&mut registry).unwrap();
        for factory in registry.factories() {
            let params = crate::FilterParams::new(factory.id());
            assert!(
                factory.build(&params).is_ok(),
                "le filtre « {} » exige un paramètre",
                factory.id()
            );
        }
    }
}
