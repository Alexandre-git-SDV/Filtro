//! Couche **Controller** — traduit la ligne de commande en configuration du
//! cœur, lance le pipeline, puis confie l'affichage à la couche [`view`](crate::view).

mod cli;

use std::process::ExitCode;

use clap::Parser;
use filtro::{FilterChain, FilterRegistry, FilterRequest, FiltroError, Pipeline, Result};

use crate::view;
use cli::{Cli, parse_formats, parse_sizes};

/// **Unique point de branchement des filtres.**
///
/// Les filtres vivent hors du cœur, dans le module `filtro::filters`.
fn register_filters(registry: &mut FilterRegistry) -> Result<()> {
    filtro::filters::register_all(registry)
}

/// Point d'entrée de l'outil : renvoie le code de sortie du processus.
pub fn run() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            view::report_error(&error);
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<()> {
    let cli = Cli::parse();

    let mut registry = FilterRegistry::new();
    register_filters(&mut registry)?;

    if cli.list_filters {
        view::print_filters(&registry);
        return Ok(());
    }

    let input = cli
        .input
        .ok_or_else(|| FiltroError::Config("aucune image fournie".to_owned()))?;

    let requests: Vec<FilterRequest> = cli
        .filters
        .iter()
        .map(|expression| FilterRequest::parse(expression))
        .collect::<Result<_>>()?;

    let chain: FilterChain = if requests.is_empty() {
        FilterChain::empty()
    } else {
        registry.build_chain(&requests)?
    };

    let formats = parse_formats(&cli.formats)?;
    let sizes = parse_sizes(&cli.sizes)?;

    let pipeline = Pipeline::builder()
        .formats(formats)
        .sizes(sizes)
        .quality(cli.quality)
        .max_dimension(cli.max_dimension)
        .build()?;

    let output = pipeline.run_path(&input, &chain)?;
    let written = output.write_to_dir(&cli.out)?;

    view::print_summary(&output, &written);
    Ok(())
}
