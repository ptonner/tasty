use core::panic;
use std::fs;
use std::time::SystemTime;

use futures::channel::mpsc::Receiver;
use miniquad::*;
use notify::event::{DataChange, ModifyKind};
use notify::{Error, Event, EventKind};

pub mod shader;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float3),
                UniformDesc::new("iMouse", UniformType::Float4),
                UniformDesc::new("iTime", UniformType::Float1),
                UniformDesc::new("iTimeDelta", UniformType::Float1),
                UniformDesc::new("iFrame", UniformType::Int1),
                UniformDesc::new("iFrameRate", UniformType::Float1),
            ],
        },
    }
}

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

pub struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    uniforms: Uniforms,
    start: SystemTime,
    last_frame: SystemTime,
    mouse_state: MouseState,
    receiver: Receiver<Result<Event, Error>>,
}

impl Stage {
    pub fn new(rx: Receiver<Result<Event, Error>>) -> Stage {
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

        let fragment = shader::build_fragment_shader(shader::MAIN_IMAGE);
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: fragment.as_str(),
                    },
                    Backend::Metal => panic!("Metal not supported"),
                },
                meta(),
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

        let (w, h) = window::screen_size();
        Stage {
            pipeline,
            bindings,
            ctx,
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

    // TODO: move into watch?
    fn handle_event(&mut self, event: Event) {
        match event {
            Event {
                kind: EventKind::Modify(ModifyKind::Data(DataChange::Any)),
                paths,
                attrs: _,
            } => {
                let p = &paths[0];
                match p.file_name().unwrap().to_owned().to_str().unwrap() {
                    "toy.glsl" => match fs::read_to_string(p) {
                        Ok(toy) => self.recompile(&toy),
                        Err(err) => println!("Error reading {:?}: {:}", p, err),
                    },
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn recompile(&mut self, toy: &String) {
        let fragment = shader::build_fragment_shader(toy);
        match self.ctx.new_shader(
            match self.ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: fragment.as_str(),
                },
                Backend::Metal => panic!("Metal not supported"),
            },
            meta(),
        ) {
            Ok(shader) => {
                let pipeline = self.ctx.new_pipeline(
                    &[BufferLayout::default()],
                    &[
                        VertexAttribute::new("in_pos", VertexFormat::Float2),
                        VertexAttribute::new("in_uv", VertexFormat::Float2),
                    ],
                    shader,
                    PipelineParams::default(),
                );
                self.pipeline = pipeline;
            }
            // TODO: add visual indicator of failed compilation
            Err(err) => println!("Failed to compile shader: {:}", err),
        }
    }
}

impl EventHandler for Stage {
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
            Ok(Some(Ok(evt))) => self.handle_event(evt),
            Ok(Some(Err(err))) => println!("error: {:?}", err),
            Ok(None) => println!("closed"),
            Err(_e) => (),
        }
    }

    fn draw(&mut self) {
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
