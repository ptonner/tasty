use core::panic;
use std::process::exit;

use futures::executor::ThreadPool;
use miniquad::*;

mod toy;
mod watch;

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    uniforms: shader::Uniforms,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();
        window::show_mouse(false);

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -1.0, y: -1.0 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y: -1.0 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y:  1.0 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -1.0, y:  1.0 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => panic!("Metal not supported"),
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        Stage {
            pipeline,
            bindings,
            ctx,
            uniforms: shader::Uniforms {
                u_resolution: window::screen_size(),
                u_mouse: (0.0, 0.0),
            },
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let t = date::now();

        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&self.uniforms));
        self.ctx.draw(0, 6, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }

    fn resize_event(&mut self, _width: f32, _height: f32) {
        self.uniforms.u_resolution = (_width, _height);
    }

    fn mouse_motion_event(&mut self, _x: f32, _y: f32) {
        let h = self.uniforms.u_resolution.1;
        self.uniforms.u_mouse = (_x, h - _y);
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    toy::create_toy(&path);

    let pool = ThreadPool::new().unwrap();
    pool.spawn_ok(async {
        if let Err(e) = watch::async_watch(path).await {
            println!("error: {:?}", e)
        }
    });

    let mut conf = conf::Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 330
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    //uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        texcoord = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 330
    varying lowp vec2 texcoord;

    uniform vec2 u_resolution;
    uniform vec2 u_mouse;

    out vec4 outColor;

    void main() {
        // gl_FragColor = texture2D(tex, texcoord);
        // gl_FragColor = vec4(texcoord.x, texcoord.y, 1, 1);
        // gl_FragColor = vec4(texcoord, 0, 0);
        // outColor = vec4(fract(gl_FragCoord.xy / 50.0), 0, 1);
        // outColor = vec4(fract(gl_FragCoord.xy / u_resolution), 0, 1);
        outColor = vec4(fract((gl_FragCoord.xy - u_mouse) / u_resolution), 0, 1);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            // images: vec!["tex".to_string()],
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("u_resolution", UniformType::Float2),
                    UniformDesc::new("u_mouse", UniformType::Float2),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub u_resolution: (f32, f32),
        pub u_mouse: (f32, f32),
    }
}

// fn main() {
//     let path = std::env::args()
//         .nth(1)
//         .expect("Argument 1 needs to be a path");

//     toy::create_toy(&path);

//     futures::executor::block_on(async {
//         if let Err(e) = watch::async_watch(path).await {
//             println!("error: {:?}", e)
//         }
//     });
// }
