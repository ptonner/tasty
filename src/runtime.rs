use core::panic;
use miniquad::TextureWrap;
use std::time::SystemTime;

use image::ImageFormat;
use image::ImageReader;
use miniquad::*;
use std::io::Cursor;

use futures::channel::mpsc::Receiver;

use crate::toy::Channel;
use crate::toy::TextureFilter;
use crate::toy::TextureWrap as ToyTextureWrap;
use crate::toy::{shader, ChannelConfig, Toy};

/// The runtime interface for toy execution
pub trait IRuntime {
    /// Initialize the runtime with a toy definition and optional update channel
    fn start(config: Toy, receiver: Option<Receiver<Toy>>);

    /// Compile the runtime for a given toy definition
    fn compile(&mut self, config: &Toy) -> Result<(), Box<dyn std::error::Error + 'static>>;
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
    // iSampleRate
    // iDate
}

enum MouseState {
    Down { x: f32, y: f32 },
    Up,
}

pub struct Runtime {
    context: Box<dyn RenderingBackend>,
    pipeline: Option<Pipeline>,
    bindings: Bindings,
    uniforms: Uniforms,
    start: SystemTime,
    last_frame: SystemTime,
    mouse_state: MouseState,
    receiver: Option<Receiver<Toy>>,
}

impl Runtime {
    pub fn new(rx: Option<Receiver<Toy>>) -> Runtime {
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

        let (w, h) = window::screen_size();
        Runtime {
            pipeline: None,
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

    fn add_channel(&mut self, channel: &Channel) -> TextureId {
        let b = channel.get_bytes();
        match channel.config {
            ChannelConfig::Texture {
                vflip,
                filter,
                wrap,
            } => {
                let mut reader = ImageReader::new(Cursor::new(b));
                reader.set_format(ImageFormat::Png);
                let mut im = reader.decode().unwrap();
                if vflip {
                    im = im.flipv();
                }
                let image = im.into_rgba8();
                let tex_id = self.context.new_texture_from_rgba8(
                    image.width() as _,
                    image.height() as _,
                    image.into_raw().as_slice(),
                );
                match filter {
                    TextureFilter::Mipmap => self.context.texture_set_filter(
                        tex_id,
                        FilterMode::Linear,
                        MipmapFilterMode::Nearest,
                    ),
                    TextureFilter::Linear => self.context.texture_set_filter(
                        tex_id,
                        FilterMode::Linear,
                        MipmapFilterMode::None,
                    ),
                    TextureFilter::Nearest => self.context.texture_set_filter(
                        tex_id,
                        FilterMode::Nearest,
                        MipmapFilterMode::None,
                    ),
                }
                match wrap {
                    ToyTextureWrap::Repeat => self.context.texture_set_wrap(
                        tex_id,
                        TextureWrap::Repeat,
                        TextureWrap::Repeat,
                    ),
                    ToyTextureWrap::Clamp => self.context.texture_set_wrap(
                        tex_id,
                        TextureWrap::Clamp,
                        TextureWrap::Clamp,
                    ),
                }
                return tex_id;
            }
        }
    }
}

impl IRuntime for Runtime {
    fn start(toy: Toy, rx: Option<Receiver<Toy>>) {
        let mut conf = conf::Conf::default();
        conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

        miniquad::start(conf, move || {
            let mut runtime = Self::new(rx);
            let _ = runtime.compile(&toy);
            Box::new(runtime)
        });
    }

    fn compile(&mut self, toy: &Toy) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.bindings.images = toy
            .config
            .channels
            .iter()
            .map(|c| self.add_channel(c))
            .collect();
        log::debug!(
            "Image definitions: {:?}",
            self.bindings
                .images
                .iter()
                .map(|tid| self.context.texture_params(*tid))
                .collect::<Vec<_>>()
        );

        let fragment = toy.fragment_shader();

        let meta = ShaderMeta {
            images: (0..self.bindings.images.len())
                .map(|i| format!("iChannel{i}"))
                .collect(),
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
        };

        match self.context.new_shader(
            match self.context.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: fragment.as_str(),
                },
                Backend::Metal => panic!("Metal not supported"),
            },
            meta,
        ) {
            Ok(shader) => {
                self.pipeline = Some(self.context.new_pipeline(
                    &[BufferLayout::default()],
                    &[
                        VertexAttribute::new("in_pos", VertexFormat::Float2),
                        VertexAttribute::new("in_uv", VertexFormat::Float2),
                    ],
                    shader,
                    PipelineParams::default(),
                ));
                Ok(())
            }
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl EventHandler for Runtime {
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

        match &mut self.receiver {
            Some(rec) => match rec.try_next() {
                Ok(Some(cfg)) => match self.compile(&cfg) {
                    Ok(()) => log::debug!("Successfully recompiled shader"),
                    // TODO: add visual indicator of error
                    Err(e) => log::error!("Error compiling: {:}", e),
                },
                Ok(None) => log::info!("Channel closed"),
                Err(_e) => (),
            },
            None => (),
        }
    }

    fn draw(&mut self) {
        match self.pipeline {
            Some(pipeline) => {
                self.context.begin_default_pass(Default::default());

                self.context.apply_pipeline(&pipeline);
                self.context.apply_bindings(&self.bindings);
                self.context
                    .apply_uniforms(UniformsSource::table(&self.uniforms));
                self.context.draw(0, 6, 1);
                self.context.end_render_pass();

                self.context.commit_frame();
            }
            None => (),
        }
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
