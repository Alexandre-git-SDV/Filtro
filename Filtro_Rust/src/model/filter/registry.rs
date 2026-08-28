//! Déclaration, configuration et instanciation des filtres.
//!
//! Un filtre se demande en une chaîne : `jaune`, `jaune:niveau=200`,
//! `noir-blanc:niveau=128,clair=ffcc00`.
//!
//! * [`FilterRequest`] découpe cette expression ;
//! * [`FilterParams`] convertit les valeurs (toujours du texte au départ) ;
//! * [`FilterFactory`] valide et construit le filtre ;
//! * [`FilterRegistry`] fait l'annuaire des fabriques connues.
//!
//! Les paramètres sont volontairement textuels : une fabrique décide seule du
//! type attendu, ce qui évite au cœur de connaître la nature des filtres.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::model::error::{FiltroError, Result};
use crate::model::filter::contract::{Filter, FilterChain};
use crate::model::pixel::Rgba8;

// ---------------------------------------------------------------------------
// Paramètres
// ---------------------------------------------------------------------------

/// Paramètres nommés destinés à une fabrique.
///
/// Les accesseurs prennent la valeur par défaut en argument : un filtre
/// s'écrit donc sans code de configuration répétitif.
///
/// ```
/// # use filtro::{FilterRequest, Rgba8};
/// let request = FilterRequest::parse("jaune:niveau=200")?;
/// assert_eq!(request.params.u8("niveau", 180)?, 200);
/// assert_eq!(request.params.color("couleur", Rgba8::WHITE)?, Rgba8::WHITE);
/// # Ok::<(), filtro::FiltroError>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterParams {
    filter: String,
    values: BTreeMap<String, String>,
}

impl FilterParams {
    /// Crée un jeu de paramètres vide destiné au filtre `filter`.
    pub fn new(filter: impl Into<String>) -> Self {
        Self {
            filter: filter.into(),
            values: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.values.insert(name.into(), value.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Valeur brute, sans conversion.
    pub fn raw(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    /// Texte, ou la valeur par défaut.
    pub fn text<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.raw(name).unwrap_or(default)
    }

    /// Entier 0–255 (seuils, composantes).
    pub fn u8(&self, name: &str, default: u8) -> Result<u8> {
        self.parse(name, default, "un entier entre 0 et 255")
    }

    /// Entier positif.
    pub fn u32(&self, name: &str, default: u32) -> Result<u32> {
        self.parse(name, default, "un entier positif")
    }

    /// Nombre réel.
    pub fn f64(&self, name: &str, default: f64) -> Result<f64> {
        self.parse(name, default, "un nombre")
    }

    /// Nombre réel contraint à un intervalle.
    pub fn f64_in(
        &self,
        name: &str,
        default: f64,
        range: std::ops::RangeInclusive<f64>,
    ) -> Result<f64> {
        let value = self.f64(name, default)?;
        if !range.contains(&value) {
            return Err(self.invalid(
                name,
                format!(
                    "{value} est hors de l'intervalle [{}, {}]",
                    range.start(),
                    range.end()
                ),
            ));
        }
        Ok(value)
    }

    /// Booléen : `true`/`vrai`/`oui`/`1` ou `false`/`faux`/`non`/`0`.
    pub fn bool(&self, name: &str, default: bool) -> Result<bool> {
        match self.raw(name) {
            None => Ok(default),
            Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "true" | "vrai" | "oui" | "1" => Ok(true),
                "false" | "faux" | "non" | "0" => Ok(false),
                other => Err(self.invalid(name, format!("« {other} » n'est pas un booléen"))),
            },
        }
    }

    /// Couleur hexadécimale : `fefe01`, `#fefe01` ou `fefe01ff` (avec alpha).
    ///
    /// La virgule séparant les paramètres, la notation `r,v,b` n'est pas
    /// utilisable ici.
    pub fn color(&self, name: &str, default: Rgba8) -> Result<Rgba8> {
        match self.raw(name) {
            None => Ok(default),
            Some(raw) => Rgba8::from_hex(raw).ok_or_else(|| {
                self.invalid(
                    name,
                    format!("« {raw} » n'est pas une couleur hexadécimale (ex. fefe01)"),
                )
            }),
        }
    }

    /// Rejette les paramètres absents de la déclaration de la fabrique.
    /// Rend les fautes de frappe visibles immédiatement.
    pub fn check(&self, specs: &[ParamSpec]) -> Result<()> {
        let unknown = self
            .values
            .keys()
            .find(|name| !specs.iter().any(|spec| spec.name == name.as_str()));
        match unknown {
            None => Ok(()),
            Some(name) => {
                let known: Vec<&str> = specs.iter().map(|spec| spec.name).collect();
                Err(self.invalid(
                    name,
                    if known.is_empty() {
                        "ce filtre n'accepte aucun paramètre".to_owned()
                    } else {
                        format!("paramètres acceptés : {}", known.join(", "))
                    },
                ))
            }
        }
    }

    fn parse<T: FromStr>(&self, name: &str, default: T, expected: &str) -> Result<T> {
        match self.raw(name) {
            None => Ok(default),
            Some(raw) => raw
                .trim()
                .parse()
                .map_err(|_| self.invalid(name, format!("« {raw} » n'est pas {expected}"))),
        }
    }

    fn invalid(&self, name: &str, reason: impl Into<String>) -> FiltroError {
        FiltroError::InvalidParameter {
            filter: self.filter.clone(),
            name: name.to_owned(),
            reason: reason.into(),
        }
    }
}

/// Description d'un paramètre, pour l'aide en ligne et la détection des fautes.
///
/// `default` est purement documentaire : la valeur réellement appliquée est
/// celle passée aux accesseurs de [`FilterParams`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: &'static str,
    pub help: &'static str,
}

// ---------------------------------------------------------------------------
// Fabriques et registre
// ---------------------------------------------------------------------------

/// Sait décrire un filtre et l'instancier à partir de paramètres.
pub trait FilterFactory: Send + Sync {
    /// Identifiant unique, en minuscules, sans espace ni deux-points.
    fn id(&self) -> &'static str;

    /// Résumé d'une ligne affiché par `filtro --list-filters`.
    fn description(&self) -> &'static str;

    /// Paramètres acceptés.
    fn parameters(&self) -> &'static [ParamSpec] {
        &[]
    }

    /// Valide les paramètres et construit le filtre.
    fn build(&self, params: &FilterParams) -> Result<Box<dyn Filter>>;
}

/// Filtre demandé par l'utilisateur, avant résolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRequest {
    pub id: String,
    pub params: FilterParams,
}

impl FilterRequest {
    /// Analyse une expression `id` ou `id:clé=valeur,clé=valeur`.
    pub fn parse(expression: &str) -> Result<Self> {
        let expression = expression.trim();
        let (id, rest) = match expression.split_once(':') {
            Some((id, rest)) => (id.trim(), rest),
            None => (expression, ""),
        };
        if id.is_empty() {
            return Err(FiltroError::Config(format!(
                "expression de filtre invalide : « {expression} »"
            )));
        }

        let mut params = FilterParams::new(id);
        for pair in rest.split(',').filter(|p| !p.trim().is_empty()) {
            let (key, value) = pair.split_once('=').ok_or_else(|| {
                FiltroError::Config(format!(
                    "paramètre « {} » mal formé (attendu clé=valeur)",
                    pair.trim()
                ))
            })?;
            params.set(key.trim(), value.trim());
        }

        Ok(Self {
            id: id.to_owned(),
            params,
        })
    }
}

/// Annuaire des filtres disponibles à l'exécution.
///
/// Le cœur crée un registre vide ; les filtres s'y ajoutent au démarrage.
#[derive(Default)]
pub struct FilterRegistry {
    factories: BTreeMap<&'static str, Box<dyn FilterFactory>>,
}

impl FilterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute une fabrique.
    ///
    /// # Erreurs
    /// Identifiant mal formé ou déjà pris.
    pub fn register(&mut self, factory: impl FilterFactory + 'static) -> Result<()> {
        let id = factory.id();
        if id.is_empty() || id.contains(char::is_whitespace) || id.contains(':') {
            return Err(FiltroError::Config(format!(
                "identifiant de filtre invalide : « {id} »"
            )));
        }
        if self.factories.contains_key(id) {
            return Err(FiltroError::Config(format!(
                "deux filtres portent l'identifiant « {id} »"
            )));
        }
        self.factories.insert(id, Box::new(factory));
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// Fabriques disponibles, triées par identifiant.
    pub fn factories(&self) -> impl Iterator<Item = &dyn FilterFactory> + '_ {
        self.factories.values().map(AsRef::as_ref)
    }

    pub fn get(&self, id: &str) -> Option<&dyn FilterFactory> {
        self.factories.get(id).map(AsRef::as_ref)
    }

    /// Construit un filtre unique.
    pub fn build(&self, request: &FilterRequest) -> Result<Box<dyn Filter>> {
        let factory = self
            .get(&request.id)
            .ok_or_else(|| FiltroError::UnknownFilter {
                name: request.id.clone(),
                available: self
                    .factories
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", "),
            })?;
        request.params.check(factory.parameters())?;
        factory.build(&request.params)
    }

    /// Construit une chaîne complète, dans l'ordre fourni.
    pub fn build_chain(&self, requests: &[FilterRequest]) -> Result<FilterChain> {
        let mut filters = Vec::with_capacity(requests.len());
        for request in requests {
            filters.push(self.build(request)?);
        }
        Ok(FilterChain::new(filters))
    }
}

impl fmt::Debug for FilterRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterRegistry")
            .field("filtres", &self.factories.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifiant_seul() {
        let request = FilterRequest::parse("jaune").unwrap();
        assert_eq!(request.id, "jaune");
        assert!(request.params.is_empty());
    }

    #[test]
    fn parse_parametres() {
        let request = FilterRequest::parse("jaune:niveau=200,couleur=fefe01").unwrap();
        assert_eq!(request.params.u8("niveau", 180).unwrap(), 200);
        assert_eq!(
            request.params.color("couleur", Rgba8::WHITE).unwrap(),
            Rgba8::new(254, 254, 1, 255)
        );
    }

    #[test]
    fn valeurs_par_defaut() {
        let params = FilterParams::new("test");
        assert_eq!(params.u8("niveau", 180).unwrap(), 180);
        assert!(!params.bool("inverse", false).unwrap());
        assert_eq!(params.text("mode", "canaux"), "canaux");
    }

    #[test]
    fn conversions_invalides() {
        let request = FilterRequest::parse("test:niveau=300,actif=peut-etre").unwrap();
        assert!(request.params.u8("niveau", 0).is_err());
        assert!(request.params.bool("actif", false).is_err());
    }

    #[test]
    fn parse_refuse_les_expressions_malformees() {
        assert!(FilterRequest::parse("").is_err());
        assert!(FilterRequest::parse("jaune:niveau").is_err());
    }

    #[test]
    fn parametre_inconnu_signale() {
        let request = FilterRequest::parse("jaune:nivo=200").unwrap();
        let specs = [ParamSpec {
            name: "niveau",
            default: "180",
            help: "seuil",
        }];
        assert!(request.params.check(&specs).is_err());
    }
}
