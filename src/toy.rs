use std::io::Error;
use std::path::Path;
use std::{fs, io};

use clap::error::Result;

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

impl Toy {
    pub fn from_path<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let image_path = path.as_ref().join("image.glsl");
        let mut toy = Toy::default();
        if image_path.exists() {
            let main_image = fs::read_to_string(image_path)?;
            toy.main_image = main_image;
        }
        return Ok(toy);
    }

    pub fn write<P>(&self, path: P, overwrite: bool) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        if !path.as_ref().exists() {
            fs::create_dir_all(&path).expect("directory accessible");
        }
        let image_path = path.as_ref().join("image.glsl");
        if !image_path.exists() | overwrite {
            return fs::write(image_path, &self.main_image);
        }
        return Ok(());
    }

    pub fn fragment_shader(&self) -> String {
        return shader::build_fragment_shader(self.main_image.as_str());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs, str::FromStr};
    use tempdir::TempDir;

    #[test]
    fn toy_writing() {
        let toy = Toy::default();
        let tmp_dir = TempDir::new("example").unwrap().into_path();
        let _ = toy.write(&tmp_dir, false);
        let data = fs::read_to_string(&tmp_dir.clone().join("image.glsl")).unwrap();
        assert_eq!(data, toy.main_image);
    }

    #[test]
    fn toy_from_disk() {
        // use default from empty directory
        let tmp_dir = TempDir::new("example").unwrap().into_path();
        let mut toy = Toy::from_path(&tmp_dir).unwrap();
        assert_eq!(shader::MAIN_IMAGE, toy.main_image);

        // make sure changes are loaded
        toy.main_image = "test".into();
        toy.write(&tmp_dir, true).unwrap();
        let toy = Toy::from_path(tmp_dir).unwrap();
        assert_eq!(String::from_str("test").unwrap(), toy.main_image);
    }

    #[test]
    fn create_frag_shader() {
        let toy = Toy::default();
        let frag = toy.fragment_shader();
        assert!(frag.contains(shader::MAIN_IMAGE));
    }
}
