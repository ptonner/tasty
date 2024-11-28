pub mod shader;

/// The definition of a Shader Toy
#[derive(Debug)]
pub struct Toy {
    /// Main image definition
    pub main_image: String,
}

impl Default for Toy {
    fn default() -> Self {
        Toy {
            main_image: shader::MAIN_IMAGE.into(),
        }
    }
}
