use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use png::Decoder;
use winres::WindowsResource;

const SMALL_ICON_PATH: &str = "resources/icons/icon16.png";
const MEDIUM_ICON_PATH: &str = "resources/icons/icon32.png";
const LARGE_ICON_PATH: &str = "resources/icons/icon64.png";
const WINDOWS_ICON_PATH: &str = "resources/icons/windows_icon.ico";

fn read_icon(path: &str, size: usize) -> Vec<u8> {
    let file = File::open(path).expect("icon file not found");
    let decoder = Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().expect("error initialising reader");
    let mut buf = vec![0; reader.output_buffer_size().expect("error reading icon")];
    let info = reader.next_frame(&mut buf).unwrap();

    assert_eq!(info.height as usize, size);
    assert_eq!(info.width as usize, size);

    buf
}

fn set_windows_icon() {
    let mut res = WindowsResource::new();
    res.set_icon(WINDOWS_ICON_PATH);
    res.compile().unwrap()
}

fn main() {
    println!("cargo::rerun-if-changed={SMALL_ICON_PATH}");
    println!("cargo::rerun-if-changed={MEDIUM_ICON_PATH}");
    println!("cargo::rerun-if-changed={LARGE_ICON_PATH}");

    let small_icon = read_icon(SMALL_ICON_PATH, 16);
    let medium_icon = read_icon(MEDIUM_ICON_PATH, 32);
    let large_icon = read_icon(LARGE_ICON_PATH, 64);

    let generated_code = format!(
        r#"pub const SMALL_ICON: [u8; 16*16*4] = {small_icon:?};
        pub const MEDIUM_ICON: [u8; 32*32*4] = {medium_icon:?};
        pub const LARGE_ICON: [u8; 64*64*4] = {large_icon:?};"#
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("icons.rs");
    fs::write(&dest_path, generated_code).unwrap();

    #[cfg(windows)]
    set_windows_icon();
}
