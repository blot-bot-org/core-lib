use pyo3::prelude::*;

/// 
/// An interfacing object, used in the Python code, to store drawing instructions
/// so they can be later iterated and performed internally on a drawing surface.
///
/// # Fields:
/// - `instructions`: A vector of `GenericInstruction` objects
///
#[pyclass]
pub struct SurfaceInterface {
    instructions: Vec<GenericInstruction>
}


#[pymethods]
impl SurfaceInterface {
    /// 
    /// # Returns:
    /// - A new instance of the SurfaceInterface with no instructions
    ///
    #[new]
    pub fn new() -> Self {
        SurfaceInterface { instructions: vec![] }
    }

    ///
    /// Pushes a raise_pen instruction to the instruction vector
    ///
    /// # Parameters:
    /// - `raise`: A boolean, true if you want the pen to be raised, else false
    ///
    pub fn raise_pen(&mut self, raise: bool) {
        self.instructions.push(GenericInstruction::raise_pen(raise));
    }

    ///
    /// Pushes a sample_xy instruction to the instruction vector
    ///
    /// # Parameters:
    /// - `x`: The new x position of the pen
    /// - `y`: The new y position of the pen
    ///
    pub fn goto(&mut self, x: f64, y: f64) {
        self.instructions.push(GenericInstruction::sample_xy(x, y));
    }

    ///
    /// # Returns:
    /// - The list of instructions on the object
    ///
    pub fn get_instructions(&self) -> Vec<GenericInstruction> {
        self.instructions.clone()
    }
}





/// 
/// A GenericInstruction represents an individual function which can be performed on a DrawSurface.
/// For example, sample_xy and raise_pen.
///
/// # Fields:
/// - `kind`: The type given to the generic instruction
/// - `raised`: If kind is raise pen, new raised state
/// - `x`: If kind is sample_xy, new x position of the pen
/// - `y`: If kind is sample_xy, new y position of the pen
///
#[derive(Clone)]
#[pyclass]
pub struct GenericInstruction {
    #[pyo3(get)]
    pub kind: String, // "raise_pen" or "sample_xy"
    #[pyo3(get)]
    pub raised: Option<bool>,
    #[pyo3(get)]
    pub x: Option<f64>,
    #[pyo3(get)]
    pub y: Option<f64>,
}

#[pymethods]
impl GenericInstruction {
    /// 
    /// Lifts the pen off or on the paper.
    ///
    /// # Parameters:
    /// - `raised`: True to lift the pen off the paper, else false
    ///
    #[staticmethod]
    pub fn raise_pen(raised: bool) -> Self {
        GenericInstruction {
            kind: "raise_pen".to_string(),
            raised: Some(raised),
            x: None,
            y: None,
        }
    }

    /// 
    /// Moves the pen to a new position on the page.
    ///
    /// # Parameters:
    /// - `x`: The new x position of the pen
    /// - `y`: The new y position of the pen
    ///
    #[staticmethod]
    pub fn sample_xy(x: f64, y: f64) -> Self {
        GenericInstruction {
            kind: "sample_xy".to_string(),
            raised: None,
            x: Some(x),
            y: Some(y),
        }
    }
}
