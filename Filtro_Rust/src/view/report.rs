//! Résumé d'un traitement terminé : source, filtres, variantes écrites.

use std::path::PathBuf;

use filtro::RenderOutput;

use super::human_size;

/// Écrit sur la sortie standard le compte rendu d'un [`RenderOutput`].
///
/// `written` est la liste des chemins réellement créés, dans le même ordre que
/// `output.artifacts`.
pub fn print_summary(output: &RenderOutput, written: &[PathBuf]) {
    let source = &output.metadata;
    println!(
        "source : {} — {} {} ({} octets)",
        source.origin, source.source_format, source.dimensions, source.byte_size
    );
    if let Some(resolution) = source.resolution {
        println!(
            "résolution : {:.0}×{:.0} ppp",
            resolution.dpi_x, resolution.dpi_y
        );
    }
    if output.applied_filters.is_empty() {
        println!("filtres : aucun");
    } else {
        println!("filtres : {}", output.applied_filters.join(" → "));
    }
    println!("{} variante(s) écrite(s) :", written.len());
    for (path, artifact) in written.iter().zip(&output.artifacts) {
        println!(
            "  {} — {} {} ({})",
            path.display(),
            artifact.target.format,
            artifact.target.dimensions,
            human_size(artifact.byte_size())
        );
    }
}
