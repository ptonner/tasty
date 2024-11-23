use std::fs;
use std::path::PathBuf;

pub fn create_toy(path: &String) {
    fs::create_dir_all(path).expect("directory accessible");
    let path = PathBuf::from(path);
    fs::write(path.join("toy.glsl"), DEFAULT_TOY_SHADER).expect("toy writeable");
    fs::write(path.join("vertex.glsl"), DEFAULT_VERTEX_SHADER).expect("toy writeable");
    fs::write(path.join("fragment.glsl"), DEFAULT_FRAGMENT_SHADER).expect("toy writeable");
}
const DEFAULT_TOY_SHADER: &str = "void mainImage(out vec4 fragColor, in vec2 fragCoord)
{    
}";

const DEFAULT_FRAGMENT_SHADER: &str = "#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = vec4(uv, 0, 0);
}
";

const DEFAULT_VERTEX_SHADER: &str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

void main() {
    gl_Position = vec4(position, 1);
    uv = texcoord;
}
";
