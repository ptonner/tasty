use core::panic;
use std::time::SystemTime;

use miniquad::*;

use futures::channel::mpsc::Receiver;

pub mod shader;

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

#[repr(C)]
#[allow(non_snake_case)]
pub struct Uniforms {
    pub iResolution: (f32, f32, f32),
    pub iMouse: (f32, f32, f32, f32),
    pub iTime: f32,
    pub iTimeDelta: f32,
    pub iFrame: i32,
    pub iFrameRate: f32,
    // TODO
    // iChannelTime
    // iChannelResolution
    // iChannel
    // iDate
    // iSampleRate
}

enum MouseState {
    Down { x: f32, y: f32 },
    Up,
}

pub struct Toy {
    context: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    uniforms: Uniforms,
    start: SystemTime,
    last_frame: SystemTime,
    mouse_state: MouseState,
    receiver: Receiver<ToyConfig>,
}

#[derive(Debug)]
pub struct ToyConfig {
    pub main_image: String,
}

impl Default for ToyConfig {
    fn default() -> Self {
        ToyConfig {
            main_image: shader::MAIN_IMAGE.into(),
        }
    }
}

impl ToyConfig {
    fn create_pipeline(
        &self,
        ctx: &mut Box<dyn RenderingBackend>,
    ) -> Result<Pipeline, ShaderError> {
        let fragment = shader::build_fragment_shader(self.main_image.as_str());
        match ctx.new_shader(
            match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: fragment.as_str(),
                },
                Backend::Metal => panic!("Metal not supported"),
            },
            shader::meta(),
        ) {
            Ok(shader) => Ok(ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("in_pos", VertexFormat::Float2),
                    VertexAttribute::new("in_uv", VertexFormat::Float2),
                ],
                shader,
                PipelineParams::default(),
            )),
            Err(err) => Err(err),
        }
    }
}

impl Toy {
    pub fn new(rx: Receiver<ToyConfig>) -> Toy {
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

        let cfg = ToyConfig::default();
        let pipeline = cfg.create_pipeline(&mut ctx).unwrap();
        let (w, h) = window::screen_size();
        Toy {
            pipeline,
            bindings,
            context: ctx,
            uniforms: Uniforms {
                iResolution: (w, h, 1.0),
                iMouse: (0.0, 0.0, 0.0, 0.0),
                iTime: 0.0,
                iTimeDelta: 0.0,
                iFrame: 0,
                iFrameRate: 0.0,
            },
            start: SystemTime::now(),
            last_frame: SystemTime::now(),
            mouse_state: MouseState::Up,
            receiver: rx,
        }
    }

    fn recompile(&mut self, config: ToyConfig) {
        match config.create_pipeline(&mut self.context) {
            Ok(pipeline) => self.pipeline = pipeline,
            // TODO: add visual indicator of failed compilation
            Err(err) => println!("Error compiling {:?}: {:}", config, err),
        }
    }
}

impl EventHandler for Toy {
    fn update(&mut self) {
        let now = (SystemTime::now()
            .duration_since(self.start)
            .expect("Linear time")
            .as_millis() as f32)
            / 1000.0;
        let dt = (SystemTime::now()
            .duration_since(self.last_frame)
            .expect("Linear time")
            .as_millis() as f32)
            / 1000.0;
        self.last_frame = SystemTime::now();
        self.uniforms.iTime = now;
        self.uniforms.iTimeDelta = dt;
        self.uniforms.iFrame += 1;
        self.uniforms.iFrameRate = 1.0 / dt;

        match self.receiver.try_next() {
            Ok(Some(cfg)) => self.recompile(cfg),
            Ok(None) => println!("closed"),
            Err(_e) => (),
        }
    }

    fn draw(&mut self) {
        self.context.begin_default_pass(Default::default());

        self.context.apply_pipeline(&self.pipeline);
        self.context.apply_bindings(&self.bindings);
        self.context
            .apply_uniforms(UniformsSource::table(&self.uniforms));
        self.context.draw(0, 6, 1);
        self.context.end_render_pass();

        self.context.commit_frame();
    }

    fn resize_event(&mut self, _width: f32, _height: f32) {
        self.uniforms.iResolution = (_width, _height, 1.0);
    }

    fn mouse_motion_event(&mut self, _x: f32, _y: f32) {
        match self.mouse_state {
            MouseState::Down { x, y } => {
                let h = self.uniforms.iResolution.1;
                self.uniforms.iMouse = (_x, h - _y, x, h - y);
            }
            _ => (),
        }
    }

    fn mouse_button_down_event(&mut self, _button: MouseButton, _x: f32, _y: f32) {
        match _button {
            MouseButton::Left => {
                self.mouse_state = MouseState::Down { x: _x, y: _y };
            }
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _button: MouseButton, _x: f32, _y: f32) {
        match _button {
            MouseButton::Left => self.mouse_state = MouseState::Up,
            _ => (),
        }
    }
}
