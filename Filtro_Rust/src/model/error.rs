//! Type d'erreur unique du cœur applicatif.
//!
//! Aucune fonction publique de ce crate ne panique sur un chemin critique :
//! toute défaillance remonte sous forme de [`FiltroError`].

use std::path::PathBuf;

use thiserror::Error;

/// Alias de `Result` utilisé partout dans le crate.
pub type Result<T> = std::result::Result<T, FiltroError>;

/// Erreurs pouvant survenir dans la chaîne Analyzer → Constructor → Formatter → Renderer.
#[derive(Debug, Error)]
pub enum FiltroError {
    /// Lecture ou écriture sur le système de fichiers impossible.
    #[error("erreur d'entrée/sortie sur « {path} »")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Le décodage de l'image source a échoué.
    #[error("décodage impossible pour « {origin} »")]
    Decode {
        origin: String,
        #[source]
        source: image::ImageError,
    },

    /// L'encodage vers un format de sortie a échoué.
    #[error("encodage impossible vers le format {format}")]
    Encode {
        format: String,
        #[source]
        source: image::ImageError,
    },

    /// Données de pixels incohérentes (dimensions nulles, tampon mal dimensionné…).
    #[error("image invalide : {0}")]
    InvalidImage(String),

    /// Aucun filtre enregistré ne porte cet identifiant.
    #[error("filtre inconnu : « {name} » (disponibles : {available})")]
    UnknownFilter { name: String, available: String },

    /// Paramètre présent mais inutilisable.
    #[error("paramètre « {name} » invalide pour le filtre « {filter} » : {reason}")]
    InvalidParameter {
        filter: String,
        name: String,
        reason: String,
    },

    /// Paramètre obligatoire absent.
    #[error("paramètre obligatoire « {name} » manquant pour le filtre « {filter} »")]
    MissingParameter { filter: String, name: String },

    /// Un filtre a refusé de traiter l'image.
    #[error("échec du filtre « {filter} » : {reason}")]
    FilterFailed { filter: String, reason: String },

    /// Format de sortie non reconnu.
    #[error("format de sortie inconnu : « {0} »")]
    UnknownFormat(String),

    /// Configuration incohérente fournie par l'appelant (CLI, API…).
    #[error("configuration invalide : {0}")]
    Config(String),
}

impl FiltroError {
    /// Raccourci pour construire une erreur d'entrée/sortie.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Raccourci pour signaler l'échec d'un filtre depuis une implémentation externe.
    pub fn filter_failed(filter: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::FilterFailed {
            filter: filter.into(),
            reason: reason.into(),
        }
    }
}
