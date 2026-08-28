//! Lecture de la résolution (points par pouce) dans les métadonnées.
//!
//! Le crate `image` n'expose pas la densité : on lit donc directement le chunk
//! `pHYs` d'un PNG ou le segment `JFIF` d'un JPEG. La lecture est défensive :
//! toute anomalie renvoie `None` plutôt qu'une erreur, car l'absence de
//! résolution n'empêche pas le traitement.

use crate::model::pipeline::analyzer::SourceFormat;

/// Résolution déclarée par le fichier source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub dpi_x: f32,
    pub dpi_y: f32,
}

const METERS_PER_INCH: f32 = 0.0254;

/// Extrait la résolution si le format la porte et qu'elle est exploitable.
pub(crate) fn read(bytes: &[u8], format: &SourceFormat) -> Option<Resolution> {
    match format {
        SourceFormat::Png => png_phys(bytes),
        SourceFormat::Jpeg => jpeg_jfif(bytes),
        _ => None,
    }
}

fn be_u32(slice: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(slice.get(..4)?.try_into().ok()?))
}

fn be_u16(slice: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(slice.get(..2)?.try_into().ok()?))
}

/// Parcourt les chunks PNG jusqu'à `pHYs` (toujours placé avant `IDAT`).
fn png_phys(bytes: &[u8]) -> Option<Resolution> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.get(..8)? != SIGNATURE {
        return None;
    }

    let mut cursor = 8usize;
    while cursor + 8 <= bytes.len() {
        let length = be_u32(&bytes[cursor..])? as usize;
        let kind = bytes.get(cursor + 4..cursor + 8)?;
        let data_start = cursor + 8;
        let data_end = data_start.checked_add(length)?;
        if data_end > bytes.len() {
            return None;
        }

        if kind == b"pHYs" && length >= 9 {
            let data = &bytes[data_start..data_end];
            // Unité 1 = pixels par mètre ; 0 = simple rapport d'aspect.
            if data[8] != 1 {
                return None;
            }
            return Some(Resolution {
                dpi_x: be_u32(data)? as f32 * METERS_PER_INCH,
                dpi_y: be_u32(&data[4..])? as f32 * METERS_PER_INCH,
            });
        }
        if kind == b"IDAT" || kind == b"IEND" {
            return None;
        }

        cursor = data_end.checked_add(4)?; // + CRC
    }
    None
}

/// Parcourt les segments JPEG jusqu'à l'en-tête APP0/JFIF.
fn jpeg_jfif(bytes: &[u8]) -> Option<Resolution> {
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }

    let mut cursor = 2usize;
    while cursor + 4 <= bytes.len() {
        if bytes[cursor] != 0xFF {
            return None;
        }
        let marker = bytes[cursor + 1];
        // Marqueurs sans charge utile.
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            cursor += 2;
            continue;
        }
        // Début des données compressées : plus rien à lire.
        if marker == 0xDA {
            return None;
        }

        let length = be_u16(&bytes[cursor + 2..])? as usize;
        if length < 2 {
            return None;
        }
        let segment = bytes.get(cursor + 4..cursor + 2 + length)?;

        if marker == 0xE0 && segment.len() >= 12 && &segment[..5] == b"JFIF\0" {
            let x = be_u16(&segment[8..])? as f32;
            let y = be_u16(&segment[10..])? as f32;
            if x == 0.0 || y == 0.0 {
                return None;
            }
            return match segment[7] {
                1 => Some(Resolution { dpi_x: x, dpi_y: y }), // points par pouce
                2 => Some(Resolution {
                    dpi_x: x * 2.54,
                    dpi_y: y * 2.54,
                }), // points par centimètre
                _ => None,
            };
        }

        cursor += 2 + length;
    }
    None
}
