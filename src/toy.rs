use log;
use std::path::Path;
use std::{fs, io};

use serde::Deserialize;
use serde::Serialize;

pub mod shader;

/// Channel configuration (texture, video, etc)
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(untagged)]
pub enum ChannelConfig {
    Texture { vflip: bool },
}

impl ChannelConfig {
    fn from_empty() -> Self {
        Self::Texture { vflip: true }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinName {
    RgbaNoiseSmall,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum ChannelSource {
    /// A built-in channel
    Builtin { name: BuiltinName },
    /// Local data
    FromDisk { path: String },
}

/// Channel definition
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Channel {
    pub source: ChannelSource,

    /// Configuration for the channel
    #[serde(default = "ChannelConfig::from_empty")]
    pub config: ChannelConfig,
}

impl Channel {
    pub fn get_bytes(&self) -> Vec<u8> {
        match self.source {
            ChannelSource::Builtin { name } => match name {
                BuiltinName::RgbaNoiseSmall => {
                    include_bytes!("toy/res/rgba-noise-small.png").into()
                }
            },
            ChannelSource::FromDisk { path: _ } => todo!(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Channels defined for this toy
    pub channels: Vec<Channel>,
}

/// The definition of a Shader Toy
#[derive(Debug)]
pub struct Toy {
    /// Main image definition
    pub main_image: String,
    /// Configuration
    pub config: Config,
}

impl Default for Toy {
    fn default() -> Self {
        Toy {
            main_image: shader::MAIN_IMAGE.into(),
            config: Config::default(),
        }
    }
}

impl Toy {
    pub fn from_path<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut toy = Toy::default();

        // load image.glsl
        let image_path = path.as_ref().join("image.glsl");
        if image_path.exists() {
            match fs::read_to_string(image_path) {
                Ok(main_image) => toy.main_image = main_image,
                Err(e) => log::error!("Error reading main image: {}", e),
            }
        }

        // load config
        let config_path = path.as_ref().join("toy.toml");
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(conf) => match toml::from_str(conf.as_str()) {
                    Ok(conf) => toy.config = conf,
                    Err(e) => log::error!("Error parsing toy config: {}", e),
                },
                Err(e) => log::error!("Error reading toy config: {}", e),
            }
        }

        return toy;
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
            fs::write(image_path, &self.main_image)?;
        }
        let config_path = path.as_ref().join("toy.toml");
        if !config_path.exists() | overwrite {
            fs::write(
                config_path,
                toml::to_string(&self.config).expect("toy always serializable"),
            )?;
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
        let mut toy = Toy::default();

        // make channels non-default
        let chan = Channel::Builtin {
            name: "foo".into(),
            config: ChannelConfig::Texture,
        };
        toy.config.channels = vec![chan];

        let tmp_dir = TempDir::new("example").unwrap().into_path();
        let _ = toy.write(&tmp_dir, false);
        let data = fs::read_to_string(&tmp_dir.clone().join("image.glsl")).unwrap();
        assert_eq!(data, toy.main_image);

        let conf: Config = toml::from_str(
            fs::read_to_string(&tmp_dir.clone().join("toy.toml"))
                .unwrap()
                .as_str(),
        )
        .unwrap();
        assert_eq!(conf, toy.config);
    }

    #[test]
    fn toy_from_disk() {
        // use default from empty directory
        let tmp_dir = TempDir::new("example").unwrap().into_path();
        let mut toy = Toy::from_path(&tmp_dir);
        assert_eq!(shader::MAIN_IMAGE, toy.main_image);

        // make sure changes are loaded
        toy.main_image = "test".into();
        toy.write(&tmp_dir, true).unwrap();
        let toy = Toy::from_path(tmp_dir);
        assert_eq!(String::from_str("test").unwrap(), toy.main_image);
    }

    #[test]
    fn toy_partial_def_err() {
        let mut toy = Toy::default();
        toy.main_image = "test".into();
        let chan = Channel::Builtin {
            name: "foo".into(),
            config: ChannelConfig::Texture,
        };
        toy.config.channels = vec![chan];

        // Setup initial directory
        let tmp_dir = TempDir::new("toy_partial_def_err").unwrap().into_path();

        // successful partial read with broken config
        toy.write(&tmp_dir, true).unwrap();
        fs::write(&tmp_dir.join("toy.toml"), "foobarbaz").unwrap();
        let read = Toy::from_path(&tmp_dir);
        assert_eq!(String::from_str("test").unwrap(), read.main_image);

        // successful partial read with missing main image
        toy.write(&tmp_dir, true).unwrap();
        fs::remove_file(&tmp_dir.join("image.glsl")).unwrap();
        let read = Toy::from_path(tmp_dir);
        assert_eq!(toy.config, read.config);
    }

    #[test]
    fn create_frag_shader() {
        let toy = Toy::default();
        let frag = toy.fragment_shader();
        assert!(frag.contains(shader::MAIN_IMAGE));
    }
}
