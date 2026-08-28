//! Couche **View** — tout ce qui s'écrit sur le terminal.
//!
//! Aucune logique métier ici : les fonctions reçoivent des types du cœur
//! (`filtro::…`) déjà calculés et se contentent de les mettre en forme.

mod catalog;
mod report;

pub use catalog::print_filters;
pub use report::print_summary;

use filtro::FiltroError;

/// Écrit une erreur sur la sortie d'erreur, en déroulant la chaîne de causes.
pub fn report_error(error: &FiltroError) {
    eprintln!("erreur : {error}");
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        eprintln!("  ↳ {cause}");
        source = cause.source();
    }
}

/// Taille lisible par un humain (o, Kio, Mio).
pub fn human_size(bytes: usize) -> String {
    const KIO: usize = 1024;
    const MIO: usize = KIO * 1024;
    match bytes {
        0..=1023 => format!("{bytes} o"),
        1024..=1_048_575 => format!("{:.1} Kio", bytes as f64 / KIO as f64),
        _ => format!("{:.1} Mio", bytes as f64 / MIO as f64),
    }
}
