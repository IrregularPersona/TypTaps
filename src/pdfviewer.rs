use iced::widget::image;
use pdfium_render::prelude::*;
use std::path::PathBuf;

pub fn render_pdf_to_handle(path: &PathBuf) -> Result<image::Handle, Box<dyn std::error::Error>> {
    let pdfium = Pdfium::default();
    let document = pdfium.load_pdf_from_file(path, None)?;
    let page = document.pages().get(0)?;
    let render_config = PdfRenderConfig::new().set_target_width(1200);
    let bitmap = page.render_with_config(&render_config)?;
    let rgba_image = bitmap.as_image().to_rgba8();
    let (width, height) = rgba_image.dimensions();
    Ok(image::Handle::from_rgba(
        width,
        height,
        rgba_image.into_raw(),
    ))
}
