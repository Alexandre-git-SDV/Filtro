# Filtro (Rust)

Outil de filtrage d'images en ligne de commande, écrit en Rust (édition 2024,
`rust-version` 1.88) : un cœur stable, et des filtres qui s'ajoutent sans jamais
toucher au cœur.

## Architecture — dossiers et MVC

Le crate expose **une bibliothèque** (`filtro`, le *Model*) et **un binaire**
(`filtro`, qui porte le *Controller* et la *View*).

```
Filtro_Rust/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              racine de la bibliothèque : modules + façade `pub use`
│   ├── main.rs             entrée du binaire : assemble controller + view
│   │
│   ├── model/              ── M — le cœur métier (aucune dépendance CLI) ──
│   │   ├── error.rs        type d'erreur unique (FiltroError)
│   │   ├── pixel.rs        types d'image partagés (Rgba8, Dimensions, PixelBuffer)
│   │   ├── resolution.rs   (privé) lecture des ppp dans les métadonnées
│   │   ├── codec.rs        (privé) pont vers le crate `image`
│   │   ├── pipeline/
│   │   │   ├── analyzer.rs      étape 1 — décode et décrit l'image
│   │   │   ├── constructor.rs   étape 2 — prépare l'image de travail
│   │   │   ├── formatter.rs     étape 3 — décline formats et tailles
│   │   │   ├── renderer.rs      étape 4 — applique les filtres et encode
│   │   │   └── orchestrator.rs  enchaîne les 4 étapes (Pipeline, PipelineBuilder)
│   │   └── filter/
│   │       ├── contract.rs      CONTRAT des filtres (Filter, FilterChain, …)
│   │       └── registry.rs      configuration et instanciation des filtres
│   │
│   ├── filters/            ── LES FILTRES, hors du cœur ──
│   │   ├── mod.rs          register_all()
│   │   ├── color.rs        jaune, vert, rouge
│   │   └── threshold.rs    noir-blanc
│   │
│   ├── controller/         ── C — traduit la CLI en configuration du cœur ──
│   │   ├── mod.rs          run() : parse → pipeline → view ; register_filters()
│   │   └── cli.rs          arguments clap + analyse des formats / tailles
│   │
│   ├── view/               ── V — tout ce qui s'écrit sur le terminal ──
│   │   ├── report.rs       résumé d'un traitement terminé
│   │   ├── catalog.rs      affichage de `--list-filters`
│   │   └── mod.rs          rapport d'erreur + tailles lisibles
│   │
│   └── test_support.rs     (cfg(test)) fabriques d'objets pour les tests
└── tests/
    └── pipeline.rs         test d'intégration bout en bout
```

`controller/` et `view/` appartiennent **au binaire uniquement** : la
bibliothèque reste un domaine pur, réutilisable depuis un autre programme.

## Le pipeline

```
Utilisateur ── image ──▶ Analyzer ──┐
                                    ├──▶ Constructor ──▶ Formatter ──▶ Renderer ──▶ sorties
Utilisateur ── filtres ─▶ Registre ─┘                                  (applique la chaîne)
```

| Étape | Fichier | Entrée | Sortie |
|-------|---------|--------|--------|
| Analyzer | `src/model/pipeline/analyzer.rs` | octets | `ImageAnalysis` (description + pixels RGBA8) |
| Constructor | `src/model/pipeline/constructor.rs` | analyse + `FilterRequirements` | `PreparedImage` |
| Formatter | `src/model/pipeline/formatter.rs` | image préparée | `FormatSet` (format × taille) |
| Renderer | `src/model/pipeline/renderer.rs` | variantes + `FilterChain` | `RenderOutput` (octets encodés) |

Autour : `src/model/pixel.rs` (types d'image), `src/model/error.rs`,
`src/model/pipeline/orchestrator.rs` (orchestration), `src/model/codec.rs` (seul
fichier qui connaît le crate `image`), `src/model/resolution.rs` (lecture des ppp).

## Cœur et filtres

Le cœur ne contient aucun filtre. Il expose un contrat :

* `src/model/filter/contract.rs` — `Filter` (transforme des pixels),
  `FilterRequirements` (ce que le filtre attend de l'image), `FilterChain`
  (composition) ;
* `src/model/filter/registry.rs` — `FilterParams`, `FilterFactory`,
  `FilterRegistry`, `FilterRequest` (analyse de `id:clé=valeur`).

Les filtres vivent dans `src/filters/`, module que le cœur ne référence jamais.
Ajouter un filtre :

1. implémenter `Filter` et `FilterFactory` dans un fichier de `src/filters/` ;
2. ajouter une ligne à `filters::register_all`.

Rien d'autre : ni le pipeline, ni le contrôleur ne changent.

## Les filtres livrés

| Identifiant | Effet |
|-------------|-------|
| `jaune` | blanc → jaune, canaux permutés en `bvr` |
| `vert` | blanc → vert, canaux permutés en `rbv` |
| `rouge` | blanc → rouge, canaux permutés en `brv` |
| `noir-blanc` | seuillage binaire |

Paramètres communs aux filtres de couleur : `niveau` (seuil de blanc, défaut
180), `couleur` (remplacement du blanc, hexadécimal), `permutation` (mot de
trois lettres parmi `r`, `v`, `b`). `noir-blanc` accepte `niveau`, `mode`
(`canaux` ou `luminance`), `clair` et `sombre`.

La virgule sépare les paramètres : les couleurs s'écrivent donc en hexadécimal
(`fefe01`), jamais en `254,254,1`.

## Utilisation

```bash
cargo run --release -- photo.jpg -f jaune
cargo run --release -- photo.jpg -f noir-blanc:niveau=128,mode=luminance
cargo run --release -- photo.jpg -f rouge -f noir-blanc        # chaînage
cargo run --release -- photo.jpg --formats png,jpeg,webp \
    --size vignette:256 --size web:1200 --quality 90 -o sortie

cargo run -- --list-filters
```

Sans filtre, `filtro` se comporte en convertisseur de formats.

## Tests

```bash
cargo test        # unitaires par module + intégration bout en bout
```

`tests/pipeline.rs` définit un filtre `luminosite` **en dehors du cœur** : il
vérifie que l'API publique (`filtro::…`) suffit à écrire un filtre et à traiter
une image de bout en bout.

## Notes

* `#![forbid(unsafe_code)]`, aucun `unwrap()` hors des tests, `cargo clippy` propre.
* Garde-fou contre les bombes de décompression (`Analyzer::with_max_pixels`).
* Résolution lue dans le chunk `pHYs` (PNG) et le segment `JFIF` (JPEG).
* Dépendances : `image` 0.25, `thiserror` 2, `clap` 4.6.
