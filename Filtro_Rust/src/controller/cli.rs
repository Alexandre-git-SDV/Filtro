//! Définition des arguments de la ligne de commande et analyse des valeurs
//! composées (formats, déclinaisons dimensionnelles).

use std::path::PathBuf;

use clap::Parser;
use filtro::{FiltroError, OutputFormat, Result, SizeVariant};

#[derive(Parser, Debug)]
#[command(
    name = "filtro",
    version,
    about = "Outil de filtrage d'images en ligne de commande",
    long_about = None
)]
pub struct Cli {
    /// Image à traiter.
    #[arg(value_name = "IMAGE", required_unless_present = "list_filters")]
    pub input: Option<PathBuf>,

    /// Filtre à appliquer, répétable : `-f id` ou `-f id:clé=valeur,clé=valeur`.
    #[arg(short = 'f', long = "filter", value_name = "EXPRESSION")]
    pub filters: Vec<String>,

    /// Formats de sortie, séparés par des virgules (png, jpeg, webp, bmp, tiff).
    #[arg(long, value_name = "FORMATS", default_value = "png")]
    pub formats: String,

    /// Déclinaison dimensionnelle, répétable : `--size vignette:256`.
    #[arg(long = "size", value_name = "ÉTIQUETTE:PIXELS")]
    pub sizes: Vec<String>,

    /// Répertoire de sortie.
    #[arg(
        short = 'o',
        long = "out",
        value_name = "RÉPERTOIRE",
        default_value = "sortie"
    )]
    pub out: PathBuf,

    /// Qualité des formats avec perte (1–100).
    #[arg(long, value_name = "1-100", default_value_t = 85)]
    pub quality: u8,

    /// Borne le plus grand côté de l'image de travail.
    #[arg(long, value_name = "PIXELS")]
    pub max_dimension: Option<u32>,

    /// Liste les filtres disponibles puis quitte.
    #[arg(long)]
    pub list_filters: bool,
}

/// Analyse la liste de formats séparés par des virgules.
pub fn parse_formats(raw: &str) -> Result<Vec<OutputFormat>> {
    let formats: Vec<OutputFormat> = raw
        .split(',')
        .filter(|token| !token.trim().is_empty())
        .map(OutputFormat::parse)
        .collect::<Result<_>>()?;
    if formats.is_empty() {
        return Err(FiltroError::Config(
            "la liste de formats est vide".to_owned(),
        ));
    }
    Ok(formats)
}

/// Analyse les déclinaisons dimensionnelles ; `Original` est toujours en tête.
pub fn parse_sizes(raw: &[String]) -> Result<Vec<SizeVariant>> {
    let mut sizes = vec![SizeVariant::Original];
    for expression in raw {
        sizes.push(SizeVariant::parse(expression)?);
    }
    Ok(sizes)
}
