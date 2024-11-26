#![allow(non_snake_case)]
use core::panic;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use miniquad::*;

pub fn create_toy(path: &String) {
    fs::create_dir_all(path).expect("directory accessible");
    let path = PathBuf::from(path);
    fs::write(path.join("toy.glsl"), DEFAULT_TOY_SHADER).expect("toy writeable");
    fs::write(path.join("vertex.glsl"), DEFAULT_VERTEX_SHADER).expect("toy writeable");
    fs::write(path.join("fragment.glsl"), DEFAULT_FRAGMENT_SHADER).expect("toy writeable");
}

pub const VERTEX: &str = r#"#version 330
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    //uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        texcoord = in_uv;
    }"#;

pub const MAIN_IMAGE: &str = r#"void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    fragColor = vec4(1, 1, 1, 1);
}"#;

pub fn build_fragment_shader(main_image: &str) -> String {
    format!(
        r#"#version 330
varying lowp vec2 texcoord;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform float iTimeDelta;
uniform int iFrame;
uniform float iFrameRate;

out vec4 outColor;

// mainImage
{main_image}

void main() {{
    mainImage(outColor, gl_FragCoord.xy);
}}"#,
        main_image = main_image
    )
}

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

const DEFAULT_TOY_SHADER: &str = "void mainImage(out vec4 fragColor, in vec2 fragCoord)
{    
}";

const TEST_TOY: &str = "
// CC0: Starry planes
//  A simple result that I think turned out pretty nice

#define TIME        iTime
#define RESOLUTION  iResolution

#define ROT(a)      mat2(cos(a), sin(a), -sin(a), cos(a))

const float
  pi        = acos(-1.)
, tau       = 2.*pi
, planeDist = .5
, furthest  = 16.
, fadeFrom  = 8.
;

const vec2 
  pathA = vec2(.31, .41)
, pathB = vec2(1.0,sqrt(0.5))
;

const vec4 
  U = vec4(0, 1, 2, 3)
  ;
  
// License: Unknown, author: Matt Taylor (https://github.com/64), found: https://64.github.io/tonemapping/
vec3 aces_approx(vec3 v) {
  v = max(v, 0.0);
  v *= 0.6;
  float a = 2.51;
  float b = 0.03;
  float c = 2.43;
  float d = 0.59;
  float e = 0.14;
  return clamp((v*(a*v+b))/(v*(c*v+d)+e), 0.0, 1.0);
}

vec3 offset(float z) {
  return vec3(pathB*sin(pathA*z), z);
}

vec3 doffset(float z) {
  return vec3(pathA*pathB*cos(pathA*z), 1.0);
}

vec3 ddoffset(float z) {
  return vec3(-pathA*pathA*pathB*sin(pathA*z), 0.0);
}

vec4 alphaBlend(vec4 back, vec4 front) {
  // Based on: https://en.wikipedia.org/wiki/Alpha_compositing
  float w = front.w + back.w*(1.0-front.w);
  vec3 xyz = (front.xyz*front.w + back.xyz*back.w*(1.0-front.w))/w;
  return w > 0.0 ? vec4(xyz, w) : vec4(0.0);
}

// License: MIT, author: Inigo Quilez, found: https://www.iquilezles.org/www/articles/smin/smin.htm
float pmin(float a, float b, float k) {
  float h = clamp(0.5+0.5*(b-a)/k, 0.0, 1.0);
  return mix(b, a, h) - k*h*(1.0-h);
}

float pmax(float a, float b, float k) {
  return -pmin(-a, -b, k);
}

float pabs(float a, float k) {
  return -pmin(a, -a, k);
}

// License: MIT, author: Inigo Quilez, found: https://iquilezles.org/articles/distfunctions2d/
//   Slightly tweaked to round the inner corners
float star5(vec2 p, float r, float rf, float sm) {
  p = -p;
  const vec2 k1 = vec2(0.809016994375, -0.587785252292);
  const vec2 k2 = vec2(-k1.x,k1.y);
  p.x = abs(p.x);
  p -= 2.0*max(dot(k1,p),0.0)*k1;
  p -= 2.0*max(dot(k2,p),0.0)*k2;
  p.x = pabs(p.x, sm);
  p.y -= r;
  vec2 ba = rf*vec2(-k1.y,k1.x) - vec2(0,1);
  float h = clamp( dot(p,ba)/dot(ba,ba), 0.0, r );
  return length(p-ba*h) * sign(p.y*ba.x-p.x*ba.y);
}

vec3 palette(float n) {
  return 0.5+0.5*sin(vec3(0.,1.,2.)+n);
}

vec4 plane(vec3 ro, vec3 rd, vec3 pp, vec3 npp, float pd, vec3 cp, vec3 off, float n) {

  float aa = 3.*pd*distance(pp.xy, npp.xy);
  vec4 col = vec4(0.);
  vec2 p2 = pp.xy;
  p2 -= offset(pp.z).xy;
  vec2 doff   = ddoffset(pp.z).xz;
  vec2 ddoff  = doffset(pp.z).xz;
  float dd = dot(doff, ddoff);
  p2 *= ROT(dd*pi*5.);

  float d0 = star5(p2, 0.45, 1.6,0.2)-0.02;
  float d1 = d0-0.01;
  float d2 = length(p2);
  const float colp = pi*100.;
  float colaa = aa*200.;
  
  col.xyz = palette(0.5*n+2.*d2)*mix(0.5/(d2*d2), 1., smoothstep(-0.5+colaa, 0.5+colaa, sin(d2*colp)))/max(3.*d2*d2, 1E-1);
  col.xyz = mix(col.xyz, vec3(2.), smoothstep(aa, -aa, d1)); 
  col.w = smoothstep(aa, -aa, -d0);
  return col;

}

vec3 color(vec3 ww, vec3 uu, vec3 vv, vec3 ro, vec2 p) {
  float lp = length(p);
  vec2 np = p + 1./RESOLUTION.xy;
  float rdd = 2.0-0.25;
  
  vec3 rd = normalize(p.x*uu + p.y*vv + rdd*ww);
  vec3 nrd = normalize(np.x*uu + np.y*vv + rdd*ww);

  float nz = floor(ro.z / planeDist);

  vec4 acol = vec4(0.0);

  vec3 aro = ro;
  float apd = 0.0;

  for (float i = 1.; i <= furthest; ++i) {
    if ( acol.w > 0.95) {
      // Debug col to see when exiting
      // acol.xyz = palette(i); 
      break;
    }
    float pz = planeDist*nz + planeDist*i;

    float lpd = (pz - aro.z)/rd.z;
    float npd = (pz - aro.z)/nrd.z;
    float cpd = (pz - aro.z)/ww.z;

    {
      vec3 pp = aro + rd*lpd;
      vec3 npp= aro + nrd*npd;
      vec3 cp = aro+ww*cpd;

      apd += lpd;

      vec3 off = offset(pp.z);

      float dz = pp.z-ro.z;
      float fadeIn = smoothstep(planeDist*furthest, planeDist*fadeFrom, dz);
      float fadeOut = smoothstep(0., planeDist*.1, dz);
      float fadeOutRI = smoothstep(0., planeDist*1.0, dz);

      float ri = mix(1.0, 0.9, fadeOutRI*fadeIn);

      vec4 pcol = plane(ro, rd, pp, npp, apd, cp, off, nz+i);

      pcol.w *= fadeOut*fadeIn;
      acol = alphaBlend(pcol, acol);
      aro = pp;
    }
    
  }

  return acol.xyz*acol.w;

}

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
  vec2 r = RESOLUTION.xy, q = fragCoord/r, pp = -1.0+2.0*q, p = pp;
  p.x *= r.x/r.y;

  float tm  = planeDist*TIME;

  vec3 ro   = offset(tm);
  vec3 dro  = doffset(tm);
  vec3 ddro = ddoffset(tm);

  vec3 ww = normalize(dro);
  vec3 uu = normalize(cross(U.xyx+ddro, ww));
  vec3 vv = cross(ww, uu);
  
  vec3 col = color(ww, uu, vv, ro, p);
  col = aces_approx(col);
  col = sqrt(col);
  fragColor = vec4(col, 1);
}
";

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
pub struct Uniforms {
    pub iResolution: (f32, f32, f32),
    pub iMouse: (f32, f32, f32, f32),
    pub iTime: f32,
    pub iTimeDelta: f32,
    pub iFrame: i32,
    pub iFrameRate: f32,
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

        // let fragment = build_fragment_shader(MAIN_IMAGE);
        let fragment = build_fragment_shader(TEST_TOY);
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: VERTEX,
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
