use iced::widget::image;
use pdfium_render::prelude::*;
use std::path::PathBuf;

pub fn render_pdf_to_pages(path: &PathBuf) -> Result<Vec<image::Handle>, Box<dyn std::error::Error>> {
    let pdfium = Pdfium::default();
    let document = pdfium.load_pdf_from_file(path, None)?;
    let mut handles = Vec::new();

    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("preview");

    for (i, page) in document.pages().iter().enumerate() {
        let render_config = PdfRenderConfig::new().set_target_width(1200);
        let bitmap = page.render_with_config(&render_config)?;
        let dynamic_image = bitmap.as_image();
        
        // Save to /tmp
        let tmp_path = format!("/tmp/{}-{}.png", file_stem, i + 1);
        let _ = dynamic_image.save(&tmp_path);

        let rgba_image = dynamic_image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        handles.push(image::Handle::from_rgba(
            width,
            height,
            rgba_image.into_raw(),
        ));
    }
    
    Ok(handles)
}
