use std::fs;
use std::path::Path;

const ICON_SIZE: usize = 256;
const ICON_HEADER_SIZE: u32 = 22;

fn main() {
    generate_windows_icon();
    tauri_build::build()
}

fn generate_windows_icon() {
    let mut pixels = Vec::with_capacity(ICON_SIZE * ICON_SIZE * 4);

    // Windows ICO bitmap pixels are stored bottom-up in BGRA order.
    for y in (0..ICON_SIZE).rev() {
        for x in 0..ICON_SIZE {
            let dx = x as f64 - 127.5;
            let dy = y as f64 - 127.5;
            let radius = (dx * dx + dy * dy).sqrt();
            let mut rgba = [0_u8, 0_u8, 0_u8, 0_u8];

            if radius <= 112.0 {
                rgba = [12, 22, 38, 255];
                let angle = (-dy).atan2(dx).to_degrees().rem_euclid(360.0);

                if (64.0..=84.0).contains(&radius) && (25.0..315.0).contains(&angle) {
                    rgba = [255, 199, 0, 255];
                }

                if point_in_triangle(
                    x as f64,
                    y as f64,
                    (199.0, 74.0),
                    (226.0, 71.0),
                    (218.0, 100.0),
                ) {
                    rgba = [255, 199, 0, 255];
                }

                if radius <= 28.0 {
                    rgba = [243, 239, 228, 255];
                }
            }

            pixels.extend_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
        }
    }

    let mask_row_bytes = ICON_SIZE.div_ceil(32) * 4;
    let and_mask = vec![0_u8; mask_row_bytes * ICON_SIZE];
    let mut dib = Vec::with_capacity(40 + pixels.len() + and_mask.len());

    // BITMAPINFOHEADER. ICO stores twice the visible height to include the AND mask.
    dib.extend_from_slice(&40_u32.to_le_bytes());
    dib.extend_from_slice(&(ICON_SIZE as i32).to_le_bytes());
    dib.extend_from_slice(&((ICON_SIZE * 2) as i32).to_le_bytes());
    dib.extend_from_slice(&1_u16.to_le_bytes());
    dib.extend_from_slice(&32_u16.to_le_bytes());
    dib.extend_from_slice(&0_u32.to_le_bytes());
    dib.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    dib.extend_from_slice(&0_i32.to_le_bytes());
    dib.extend_from_slice(&0_i32.to_le_bytes());
    dib.extend_from_slice(&0_u32.to_le_bytes());
    dib.extend_from_slice(&0_u32.to_le_bytes());
    dib.extend_from_slice(&pixels);
    dib.extend_from_slice(&and_mask);

    let mut ico = Vec::with_capacity(ICON_HEADER_SIZE as usize + dib.len());

    // ICONDIR: reserved, image type (icon), image count.
    ico.extend_from_slice(&0_u16.to_le_bytes());
    ico.extend_from_slice(&1_u16.to_le_bytes());
    ico.extend_from_slice(&1_u16.to_le_bytes());

    // ICONDIRENTRY. Zero width and height represent 256 pixels in ICO files.
    ico.extend_from_slice(&[0, 0, 0, 0]);
    ico.extend_from_slice(&1_u16.to_le_bytes());
    ico.extend_from_slice(&32_u16.to_le_bytes());
    ico.extend_from_slice(&(dib.len() as u32).to_le_bytes());
    ico.extend_from_slice(&ICON_HEADER_SIZE.to_le_bytes());
    ico.extend_from_slice(&dib);

    fs::write(Path::new("icons/icon.ico"), ico).expect("failed to generate icons/icon.ico");
}

fn point_in_triangle(
    x: f64,
    y: f64,
    first: (f64, f64),
    second: (f64, f64),
    third: (f64, f64),
) -> bool {
    let sign = |point: (f64, f64), left: (f64, f64), right: (f64, f64)| {
        (point.0 - right.0) * (left.1 - right.1)
            - (left.0 - right.0) * (point.1 - right.1)
    };
    let point = (x, y);
    let first_sign = sign(point, first, second);
    let second_sign = sign(point, second, third);
    let third_sign = sign(point, third, first);
    let has_negative = first_sign < 0.0 || second_sign < 0.0 || third_sign < 0.0;
    let has_positive = first_sign > 0.0 || second_sign > 0.0 || third_sign > 0.0;

    !(has_negative && has_positive)
}
