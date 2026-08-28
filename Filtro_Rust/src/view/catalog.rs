//! Affichage du catalogue de filtres (`filtro --list-filters`).

use filtro::FilterRegistry;

/// Liste les filtres enregistrés, leurs paramètres et quelques exemples.
pub fn print_filters(registry: &FilterRegistry) {
    if registry.is_empty() {
        println!("Aucun filtre enregistré.");
        println!(
            "Les filtres s'ajoutent en implémentant `Filter` + `FilterFactory`, \
             puis en les enregistrant dans `register_filters` — sans modifier le cœur."
        );
        return;
    }
    println!("Filtres disponibles ({}) :", registry.len());
    for factory in registry.factories() {
        println!("\n  {:<12} {}", factory.id(), factory.description());
        for param in factory.parameters() {
            println!(
                "      {:<12} défaut {:<8} {}",
                param.name, param.default, param.help
            );
        }
    }
    println!("\nExemples :");
    println!("  filtro photo.jpg -f jaune");
    println!("  filtro photo.jpg -f noir-blanc:niveau=128,mode=luminance");
    println!("  filtro photo.jpg -f rouge -f noir-blanc      (chaînage)");
}
