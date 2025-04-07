use image::GrayImage;
use image::Luma;
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

///
/// A canvas image with appropriate handling methods, to generate previews of drawings.
///
/// # Fields:
/// - `width`: The width of the paper, in millimetres
/// - `height`: The height of the paper, in millimetres
/// - `scale`: The scale to adjust resolution of the preview
///
pub struct PreviewCanvas {
    pub width: u32,
    pub height: u32,
    pub scale: u32,

    pub buffer: GrayImage,
}

impl PreviewCanvas {
    ///
    /// Creates a new instance of the image canvas, with a black/white image buffer.
    ///
    /// # Parameters:
    /// - `paper_width`: The width of the paper in millimetres
    /// - `paper_height`: The height of the paper in millimetres
    /// - `scale`: An optional scale to adjust the preview by, defaults to 1 
    /// 
    /// # Returns:
    /// - A new `PreviewCanvas` instance
    ///
    pub fn new(paper_width: u32, paper_height: u32, scale: Option<u32>) -> PreviewCanvas {
        let scale = scale.unwrap_or(1);

        let width = paper_width * scale;
        let height = paper_height * scale;
        
        let mut img_buffer = GrayImage::new(width, height);
        for (_, _, pixel) in img_buffer.enumerate_pixels_mut() {
            *pixel = Luma([255]);
        }
        
        PreviewCanvas { width, height, scale, buffer: img_buffer }
    }

    /// 
    /// Saves the preview to an image file on the disk.
    ///
    /// # Parameters:
    /// - `path`: The path to save the image file to
    ///
    pub fn save(&self, path: &str) {
        let _ = self.buffer.save_with_format(path, image::ImageFormat::Png);
    }

    /// 
    /// Draws an antialiased line between two points on the canvas. This function respects `scale`.
    ///
    /// # Parameters:
    /// - `x1` and `y1`: The x/y of the first point on the line
    /// - `x2` and `y2`: The x/y of the second point on the line
    ///
    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        draw_antialiased_line_segment_mut(
            &mut self.buffer,
            scale_floor_coordinates(x1, y1, self.scale),
            scale_floor_coordinates(x2, y2, self.scale),
            image::Luma([0]), interpolate
        );
    }

}

///
/// Scales and floors an (f64, f64) pair of coordinates. This is to make the values ready to reference
/// pixels on the canvas.
///
/// # Parameters: 
/// - `x`: The unscaled x value
/// - `y`: The unscaled y value
/// - `scale`: The scalar value of the coordinates
///
fn scale_floor_coordinates(x: f64, y: f64, scale: u32) -> (i32, i32) {
    ((x * scale as f64).floor() as i32, (y * scale as f64).floor() as i32)
}
