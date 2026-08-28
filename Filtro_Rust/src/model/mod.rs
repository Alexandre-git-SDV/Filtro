//! Le cœur métier de Filtro — la couche **Model** du MVC.
//!
//! Ce module ne connaît ni la ligne de commande ni le terminal : les couches
//! `controller` et `view` du binaire le consomment uniquement à travers la
//! façade publique réexportée par [`crate`].
//!
//! * [`error`] — le type d'erreur unique ;
//! * [`pixel`] — les types d'image partagés, indépendants de toute bibliothèque ;
//! * [`pipeline`] — les quatre étapes stables et leur orchestration ;
//! * [`filter`] — le *contrat* des filtres et leur annuaire d'instanciation.
//!
//! `codec` (pont vers le crate `image`) et `resolution` (lecture des ppp) restent
//! internes au crate.

pub mod error;
pub mod filter;
pub mod pipeline;
pub mod pixel;

pub(crate) mod codec;
pub(crate) mod resolution;
