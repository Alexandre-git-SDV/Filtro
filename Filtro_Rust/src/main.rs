//! Binaire `filtro` — assemble les couches **Controller** et **View** autour de
//! la bibliothèque [`filtro`] (la couche **Model**).

mod controller;
mod view;

fn main() -> std::process::ExitCode {
    controller::run()
}
