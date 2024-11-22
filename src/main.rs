use macroquad::prelude::*;

#[macroquad::main("toasty")]
async fn main() {
    let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
    let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

    let pipeline_params = PipelineParams {
        depth_write: true,
        depth_test: Comparison::LessOrEqual,
        ..Default::default()
    };

    let material = load_material(
        ShaderSource::Glsl {
            vertex: &vertex_shader,
            fragment: &fragment_shader,
        },
        MaterialParams {
            pipeline_params,
            ..Default::default()
        },
    )
    .unwrap();

    loop {
        clear_background(WHITE);

        gl_use_material(&material);
        draw_rectangle(-1.0, -1.0, 2.0, 2.0, BLACK);
        gl_use_default_material();

        next_frame().await
    }
}

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
