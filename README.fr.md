# Filtro

Filtro est un outil de traitement d'image qui transforme des photos ordinaires en
œuvres visuelles inspirées du style **Pop Art d'Andy Warhol** : aplats de couleur
saturés et contraste franc.

Il se présente comme un **outil en ligne de commande écrit en Rust** (dans
[`Filtro_Rust/`](Filtro_Rust/)) : un pipeline stable en quatre étapes et un jeu
de filtres qui s'ajoutent sans jamais toucher au cœur.

## ✨ Fonctionnalités

* 🎨 Filtres de couleur Pop Art — `jaune`, `vert`, `rouge`, `noir-blanc`
* 🔗 Chaînage de filtres (`-f rouge -f noir-blanc`)
* 🖼️ Sortie multi-format en une passe — PNG, JPEG, WebP, BMP, TIFF
* 📐 Déclinaisons dimensionnelles (vignettes, tailles web) générées ensemble
* 🔄 Simple convertisseur de formats quand aucun filtre n'est donné
* ⚡ Traitement rapide en un seul passage ; `#![forbid(unsafe_code)]`

## 🛠️ Technologies utilisées

* **Rust** (édition 2024, `rust-version` 1.88) — pipeline et CLI
* [`image`](https://crates.io/crates/image) — décodage et encodage
* [`clap`](https://crates.io/crates/clap) — analyse des arguments
* [`thiserror`](https://crates.io/crates/thiserror) — types d'erreur

## 📦 Installation locale

### Prérequis

* Une chaîne d'outils Rust ≥ 1.88 ([rustup.rs](https://rustup.rs))
* Git

### Cloner et compiler

```bash
git clone https://github.com/Alexandre-git-SDV/Filtro.git
cd Filtro/Filtro_Rust
cargo build --release
```

Le binaire se trouve alors dans `Filtro_Rust/target/release/filtro`.

## 🚀 Utilisation

À lancer depuis `Filtro_Rust/` (ou en appelant directement le binaire compilé) :

```bash
cargo run --release -- photo.jpg -f jaune
cargo run --release -- photo.jpg -f noir-blanc:niveau=128,mode=luminance
cargo run --release -- photo.jpg -f rouge -f noir-blanc            # chaînage
cargo run --release -- photo.jpg --formats png,jpeg,webp \
    --size vignette:256 --size web:1200 --quality 90 -o sortie

cargo run -- --list-filters
```

| Filtre | Effet |
|--------|-------|
| `jaune` | blanc → jaune, canaux rouge et bleu échangés |
| `vert` | blanc → vert |
| `rouge` | blanc → rouge |
| `noir-blanc` | seuillage binaire (par canaux ou par luminance) |

Les filtres de couleur acceptent `niveau` (seuil de blanc, défaut 180), `couleur`
(couleur de remplacement en hexadécimal) et `permutation`. `noir-blanc` accepte
`niveau`, `mode` (`canaux` / `luminance`), `clair` et `sombre`. Les paramètres
sont séparés par des virgules : les couleurs s'écrivent donc en hexadécimal
(`fefe01`), jamais en `254,254,1`.

## 📂 Structure du projet

```text
Filtro/
└── Filtro_Rust/               # l'outil CLI — voir Filtro_Rust/README.md
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs             # bibliothèque « filtro » (le Model) + façade publique
    │   ├── main.rs            # point d'entrée du binaire
    │   ├── model/             # M — cœur : types d'image, pipeline en 4 étapes, contrat des filtres
    │   ├── filters/           # les filtres, hors du cœur
    │   ├── controller/        # C — analyse de la CLI et orchestration
    │   └── view/              # V — affichage terminal
    └── tests/pipeline.rs      # test d'intégration bout en bout
```

Le pipeline : `Analyzer → Constructor → Formatter → Renderer`. On ajoute un filtre
en implémentant `Filter` + `FilterFactory` dans `src/filters/` puis en
l'enregistrant en une ligne — le cœur ne change pas. Détails complets dans
[`Filtro_Rust/README.md`](Filtro_Rust/README.md).

## 🎯 Objectif du projet

Filtro explore la rencontre entre programmation et création artistique en
utilisant le traitement d'image pour transformer des photos ordinaires en
compositions graphiques originales.

## 🤝 Contribution

Les contributions sont les bienvenues :

1. Forkez le projet
2. Créez une branche : `git checkout -b feature/nouvelle-fonctionnalite`
3. Faites vos modifications (`cargo test` et `cargo clippy` doivent rester au vert)
4. Créez une Pull Request

## 📄 Licence

Ce projet est open source. Consultez le dépôt pour plus d'informations sur la
licence utilisée.
