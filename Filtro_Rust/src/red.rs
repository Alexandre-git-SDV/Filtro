use image::{Rgb, RgbImage};
use rfd::FileDialog;

fn main() {
    // Niveau blanc
    let niveau_blanc: u8 = 180;

    println!("Veuillez choisir votre image");

    // Ouverture de la boîte de dialogue
    let fichier_image = FileDialog::new()
        .set_title("Sélectionner un fichier")
        .pick_file();

    let fichier_image = match fichier_image {
        Some(path) => path,
        None => {
            println!("Aucun fichier sélectionné.");
            return;
        }
    };

    println!("Démarrage du traitement de : {:?}", fichier_image);
    println!("niveau_blanc {}", niveau_blanc);

    // Chargement de l'image
    let mut img: RgbImage = image::open(&fichier_image)
        .expect("Impossible d'ouvrir l'image")
        .to_rgb8();

    let (largeur_img, hauteur_img) = img.dimensions();

    filtre_rouge(&mut img, largeur_img, hauteur_img, niveau_blanc);

    img.save("Image Rouge.jpg")
        .expect("Impossible de sauvegarder l'image");

    println!("Traitement image rouge terminé");
}

fn filtre_rouge(
    img: &mut RgbImage,
    largeur_img: u32,
    hauteur_img: u32,
    niveau_blanc: u8,
) {
    println!("Vous avez choisi le filtre Rouge");

    for y in 0..hauteur_img {
        for x in 0..largeur_img {
            let pixel = img.get_pixel(x, y);

            let r = pixel[0];
            let v = pixel[1];
            let b = pixel[2];

            let nouveau_pixel = if r > niveau_blanc
                && v > niveau_blanc
                && b > niveau_blanc
            {
                // Blanc -> Rouge
                Rgb([254, 1, 1])
            } else {
                // Permutation des couleurs (b, r, v)
                Rgb([b, r, v])
            };

            img.put_pixel(x, y, nouveau_pixel);
        }
    }
}