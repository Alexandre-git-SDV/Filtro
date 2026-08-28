# Filtro

Filtro is an image-processing tool that turns ordinary photos into visual
artworks inspired by the **Pop Art style of Andy Warhol**: flat, saturated
colours and hard contrast.

It ships as a **command-line tool written in Rust** (under
[`Filtro_Rust/`](Filtro_Rust/)): a stable four-stage pipeline and a set of
filters that can be added without ever touching the core.

## ✨ Features

* 🎨 Pop Art colour filters — `jaune`, `vert`, `rouge`, `noir-blanc`
* 🔗 Filter chaining (`-f rouge -f noir-blanc`)
* 🖼️ Multi-format output in one run — PNG, JPEG, WebP, BMP, TIFF
* 📐 Size variants (thumbnails, web sizes) generated together
* 🔄 Runs as a plain format converter when no filter is given
* ⚡ Fast, single-pass processing; `#![forbid(unsafe_code)]`

## 🛠️ Technologies Used

* **Rust** (edition 2024, `rust-version` 1.88) — pipeline and CLI
* [`image`](https://crates.io/crates/image) — decoding and encoding
* [`clap`](https://crates.io/crates/clap) — argument parsing
* [`thiserror`](https://crates.io/crates/thiserror) — error types

## 📦 Local Installation

### Prerequisites

* A Rust toolchain ≥ 1.88 ([rustup.rs](https://rustup.rs))
* Git

### Clone and build

```bash
git clone https://github.com/Alexandre-git-SDV/Filtro.git
cd Filtro/Filtro_Rust
cargo build --release
```

The binary is then at `Filtro_Rust/target/release/filtro`.

## 🚀 Usage

Run from `Filtro_Rust/` (or call the built binary directly):

```bash
cargo run --release -- photo.jpg -f jaune
cargo run --release -- photo.jpg -f noir-blanc:niveau=128,mode=luminance
cargo run --release -- photo.jpg -f rouge -f noir-blanc            # chaining
cargo run --release -- photo.jpg --formats png,jpeg,webp \
    --size vignette:256 --size web:1200 --quality 90 -o out

cargo run -- --list-filters
```

| Filter | Effect |
|--------|--------|
| `jaune` | white → yellow, red/blue channels swapped |
| `vert` | white → green |
| `rouge` | white → red |
| `noir-blanc` | binary threshold (per-channel or luminance) |

Colour filters accept `niveau` (white threshold, default 180), `couleur` (hex
replacement colour) and `permutation`. `noir-blanc` accepts `niveau`, `mode`
(`canaux` / `luminance`), `clair` and `sombre`. Parameters are comma-separated,
so colours are written in hex (`fefe01`), never as `254,254,1`.

## 📂 Project Structure

```text
Filtro/
└── Filtro_Rust/               # the CLI tool — see Filtro_Rust/README.md
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs             # library "filtro" (the Model) + public façade
    │   ├── main.rs            # binary entry point
    │   ├── model/             # M — core: pixel types, 4-stage pipeline, filter contract
    │   ├── filters/           # the filters, outside the core
    │   ├── controller/        # C — CLI parsing and orchestration
    │   └── view/              # V — terminal output
    └── tests/pipeline.rs      # end-to-end integration test
```

The pipeline: `Analyzer → Constructor → Formatter → Renderer`. Filters are added
by implementing `Filter` + `FilterFactory` in `src/filters/` and registering them
in one line — the core never changes. Full details in
[`Filtro_Rust/README.md`](Filtro_Rust/README.md).

## 🎯 Project Purpose

Filtro explores the connection between programming and digital art by using
image-processing techniques to transform ordinary photos into original graphic
compositions.

## 🤝 Contributing

Contributions are welcome:

1. Fork the project
2. Create a new branch: `git checkout -b feature/new-feature`
3. Make your changes (`cargo test` and `cargo clippy` should stay green)
4. Submit a Pull Request

## 📄 License

This project is open source. Check the repository for more information about the
license.
