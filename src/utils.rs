pub fn get_icons(ext: &str) -> (&'static str, &'static str) {
    let type_icon = match ext {
        "png" | "jpg" | "jpeg" => "🖼️",
        "typ" | "pdf" => "📄",
        "rs" => "🦀",
        "md" => "📝",
        _ => "🔹",
    };

    ("📄", type_icon)
}
