pub mod shader;

#[derive(Debug)]
pub struct Toy {
    pub main_image: String,
}

impl Default for Toy {
    fn default() -> Self {
        Toy {
            main_image: shader::MAIN_IMAGE.into(),
        }
    }
}
