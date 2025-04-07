
use crate::hardware::PhysicalDimensions;
use crate::instruction::InstructionSet;
use crate::instruction::error::InstructionError;

pub mod belts;
pub mod canvas;

///
/// Performs the provided motor instructions on a canvas, and saves the file.
///
/// # Parameters:
/// - `init_x`: The initial x value of the pen, relative to the top left motor shaft
/// - `init_y`: The initial y value of the pen, relative to the top left motor shaft
/// - `instruction_set`: The instruction set to preview
/// - `path`: The path to save the preview image to - *no checks are done to confirm the directory exists*
///
/// # Returns:
/// - `None` if the preview generated successfully, and the image was saved
/// - `InstructionError` to explain why the preview was unable to be generated
///
pub fn generate_preview(init_xy: (f64, f64), physical_dim: &PhysicalDimensions, instruction_set: &InstructionSet, path: &str) -> Option<InstructionError> {
    let mut preview_canvas = canvas::PreviewCanvas::new(210, 297, Some(4));
    let step_instructions: Vec<(i16, i16)> = match instruction_set.parse_to_numerical_steps() {
        Ok(value) => value,
        Err(err) => return Some(err)
    };

    let mut belts = belts::Belts::new_by_cartesian(physical_dim.page_horizontal_offset() + init_xy.0, physical_dim.page_vertical_offset() + init_xy.1, *physical_dim.motor_interspace());
    let mut last_xy = belts.get_as_cartesian();

    for (ld, rd) in step_instructions.iter() {
        belts.move_by_steps(*ld, -rd);
        let (x, y) = belts.get_as_cartesian();

        preview_canvas.line(last_xy.0 - *physical_dim.page_horizontal_offset(), last_xy.1 - *physical_dim.page_vertical_offset(), x - *physical_dim.page_horizontal_offset(), y - *physical_dim.page_vertical_offset());
        last_xy = (x, y);
    }

    preview_canvas.save(path);
    None
}
