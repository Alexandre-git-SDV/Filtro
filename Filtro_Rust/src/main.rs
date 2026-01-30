use std::io::Cursor;
use image::ImageReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger une image depuis un fichier
    let img = ImageReader::open("/workspaces/Filtro/Filtro_Rust/src/Image.webp")?.decode()?;

    // Sauvegarde directe
    img.save("empty.jpg")?;

    // Écrire l'image dans un buffer mémoire
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

    // Relire l'image depuis le buffer
    let img2 = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    // Sauvegarde pour vérifier
    img2.save("from_bytes.png")?;

    Ok(())
}