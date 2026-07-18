use std::fs;
use std::path::Path;

fn main() {
    generate_windows_icon();
    tauri_build::build()
}

fn generate_windows_icon() {
    const ICON_HEADER_SIZE: u32 = 22;
    let png = include_bytes!("icons/128x128@2x.png");
    let mut ico = Vec::with_capacity(ICON_HEADER_SIZE as usize + png.len());

    // ICONDIR: reserved, image type (icon), image count.
    ico.extend_from_slice(&0_u16.to_le_bytes());
    ico.extend_from_slice(&1_u16.to_le_bytes());
    ico.extend_from_slice(&1_u16.to_le_bytes());

    // ICONDIRENTRY for a 256x256, 32-bit PNG-backed icon.
    ico.extend_from_slice(&[0, 0, 0, 0]);
    ico.extend_from_slice(&1_u16.to_le_bytes());
    ico.extend_from_slice(&32_u16.to_le_bytes());
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
    ico.extend_from_slice(&ICON_HEADER_SIZE.to_le_bytes());
    ico.extend_from_slice(png);

    let icon_path = Path::new("icons/icon.ico");
    fs::write(icon_path, ico).expect("failed to generate icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/128x128@2x.png");
}
