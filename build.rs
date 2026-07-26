#[path = "src/icon_pixels.rs"]
mod icon_pixels;

fn main() {
    println!("cargo:rerun-if-changed=src/icon_pixels.rs");

    #[cfg(windows)]
    embed_windows_icon();
}

#[cfg(windows)]
fn embed_windows_icon() {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let icon_path = output.join("vibekeys.ico");
    let resource_path = output.join("vibekeys.rc");
    let mut icon = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16, 24, 32, 48, 64, 128, 256] {
        let image = ico::IconImage::from_rgba_data(size, size, icon_pixels::vk_icon_rgba(size));
        icon.add_entry(ico::IconDirEntry::encode(&image).expect("encode application icon"));
    }
    icon.write(File::create(&icon_path).expect("create application icon"))
        .expect("write application icon");

    let portable_icon_path = icon_path.to_string_lossy().replace('\\', "/");
    let mut resource = File::create(&resource_path).expect("create resource script");
    writeln!(resource, "1 ICON \"{portable_icon_path}\"").expect("write resource script");
    embed_resource::compile(&resource_path, embed_resource::NONE)
        .manifest_optional()
        .expect("embed application icon");
}
