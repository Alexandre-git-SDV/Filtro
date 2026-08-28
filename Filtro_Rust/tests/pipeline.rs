//! Test d'intégration bout-en-bout.
//!
//! Il n'utilise **que l'API publique** `filtro::…` — y compris pour définir un
//! filtre `luminosite` hors du cœur : la preuve que le contrat exposé suffit à
//! écrire un filtre sans toucher au pipeline.

use std::io::Cursor;

use filtro::{
    Analyzer, Filter, FilterChain, FilterContext, Origin, OutputFormat, Pipeline, PixelBuffer,
    Result, Rgba8, SizeVariant,
};

// --- Un filtre défini entièrement hors du crate ---------------------------

struct Luminosite {
    delta: i16,
}

impl Filter for Luminosite {
    fn id(&self) -> &str {
        "luminosite"
    }

    fn apply(&self, canvas: &mut PixelBuffer, _ctx: &FilterContext<'_>) -> Result<()> {
        canvas.map_pixels(|_, _, pixel| {
            let adjust = |channel: u8| (i16::from(channel) + self.delta).clamp(0, 255) as u8;
            Rgba8::new(adjust(pixel.r), adjust(pixel.g), adjust(pixel.b), pixel.a)
        });
        Ok(())
    }
}

// --- Fabrique d'une image source en mémoire ------------------------------

/// Encode un PNG uniforme de `width`×`height` de la couleur donnée.
fn source_png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(width, height, image::Rgba(color));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("encodage du PNG de test");
    bytes
}

fn analyze(bytes: &[u8]) -> filtro::ImageAnalysis {
    Analyzer::new()
        .analyze_bytes(
            bytes,
            Origin::Memory {
                label: "relecture".into(),
            },
        )
        .expect("relecture d'une variante produite")
}

// --- Les tests ----------------------------------------------------------

#[test]
fn bout_en_bout_avec_un_filtre_externe() {
    let png = source_png(64, 48, [100, 100, 100, 255]);

    let pipeline = Pipeline::builder()
        .formats(vec![OutputFormat::Png, OutputFormat::Jpeg])
        .sizes(vec![
            SizeVariant::Original,
            SizeVariant::parse("vignette:32").unwrap(),
        ])
        .quality(90)
        .build()
        .expect("pipeline valide");

    let chain = FilterChain::new(vec![Box::new(Luminosite { delta: 40 })]);
    let output = pipeline
        .run_bytes(
            &png,
            Origin::Memory {
                label: "source".into(),
            },
            &chain,
        )
        .expect("traitement");

    // 2 formats × 2 tailles.
    assert_eq!(output.artifacts.len(), 4);
    assert_eq!(output.applied_filters, vec!["luminosite"]);

    for artifact in &output.artifacts {
        assert!(!artifact.bytes.is_empty(), "variante vide");
        assert!(
            artifact.file_name.starts_with("relecture") || artifact.file_name.starts_with("source"),
            "nom inattendu : {}",
            artifact.file_name
        );
    }

    // La vignette PNG : plus grand côté ramené à 32, effet du filtre visible.
    let vignette = output
        .artifacts
        .iter()
        .find(|a| a.target.format == OutputFormat::Png && a.target.size.label() == "vignette")
        .expect("vignette png présente");
    assert_eq!(vignette.target.dimensions.longest_side(), 32);

    // Le PNG pleine taille : 100 + 40 = 140 sur chaque canal.
    let plein = output
        .artifacts
        .iter()
        .find(|a| a.target.format == OutputFormat::Png && a.target.size.label() == "original")
        .expect("png original présent");
    let relu = analyze(&plein.bytes);
    assert_eq!(
        relu.buffer.pixel(0, 0),
        Some(Rgba8::new(140, 140, 140, 255))
    );
}

#[test]
fn sans_filtre_le_pipeline_convertit_les_formats() {
    let png = source_png(20, 10, [10, 120, 200, 255]);

    let pipeline = Pipeline::builder()
        .formats(vec![OutputFormat::Png, OutputFormat::WebP])
        .build()
        .unwrap();

    let output = pipeline
        .run_bytes(
            &png,
            Origin::Memory {
                label: "src".into(),
            },
            &FilterChain::empty(),
        )
        .unwrap();

    assert!(output.applied_filters.is_empty());
    assert_eq!(output.artifacts.len(), 2);
    for artifact in &output.artifacts {
        assert_eq!(artifact.target.dimensions.width, 20);
        assert_eq!(artifact.target.dimensions.height, 10);
    }
}

#[test]
fn jpeg_ne_conserve_pas_la_transparence() {
    let png = source_png(16, 16, [200, 50, 50, 0]); // entièrement transparent

    let pipeline = Pipeline::builder()
        .formats(vec![OutputFormat::Jpeg])
        .build()
        .unwrap();

    let output = pipeline
        .run_bytes(
            &png,
            Origin::Memory {
                label: "src".into(),
            },
            &FilterChain::empty(),
        )
        .unwrap();

    let jpeg = &output.artifacts[0];
    assert!(!analyze(&jpeg.bytes).metadata.stats.has_transparency);
}

#[test]
fn un_flux_vide_est_rejete() {
    let pipeline = Pipeline::new();
    let result = pipeline.run_bytes(
        &[],
        Origin::Memory {
            label: "vide".into(),
        },
        &FilterChain::empty(),
    );
    assert!(result.is_err());
}

#[test]
fn la_borne_dimensionnelle_reduit_l_image_de_travail() {
    let png = source_png(200, 120, [255, 255, 255, 255]);

    let pipeline = Pipeline::builder()
        .formats(vec![OutputFormat::Png])
        .max_dimension(Some(50))
        .build()
        .unwrap();

    let output = pipeline
        .run_bytes(
            &png,
            Origin::Memory {
                label: "src".into(),
            },
            &FilterChain::empty(),
        )
        .unwrap();

    assert_eq!(output.artifacts[0].target.dimensions.longest_side(), 50);
    // La description conserve les dimensions d'origine.
    assert_eq!(output.metadata.dimensions.width, 200);
}
