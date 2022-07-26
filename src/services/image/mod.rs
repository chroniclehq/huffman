use libvips::ops;
use libvips::ops::webpsave_buffer_with_opts;
use libvips::Result;
use libvips::VipsImage;

pub fn process_image(buffer: &Vec<u8>) -> Result<Vec<u8>> {
    let source = VipsImage::new_from_buffer(buffer, "").expect("Error during VipsImage init");
    let options = ops::WebpsaveBufferOptions {
        q: 50,
        strip: true,
        reduction_effort: 2,
        ..ops::WebpsaveBufferOptions::default()
    };
    webpsave_buffer_with_opts(&source, &options)
}
