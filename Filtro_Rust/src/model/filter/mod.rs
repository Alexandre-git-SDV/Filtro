//! Le **contrat** des filtres et leur annuaire — aucun filtre concret ici.
//!
//! * [`contract`] décrit ce qu'un filtre *est* du point de vue du cœur
//!   ([`Filter`](contract::Filter), [`FilterChain`](contract::FilterChain),
//!   [`FilterRequirements`](contract::FilterRequirements)) ;
//! * [`registry`] configure et instancie les filtres à partir d'une expression
//!   textuelle ([`FilterRegistry`](registry::FilterRegistry),
//!   [`FilterRequest`](registry::FilterRequest)).
//!
//! Les filtres livrés avec le projet vivent dans [`crate::filters`], hors du cœur.

pub mod contract;
pub mod registry;
