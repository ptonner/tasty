pub const VERTEX: &str = r#"#version 330
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        texcoord = in_uv;
    }"#;

pub const MAIN_IMAGE: &str = r#"void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord/iResolution.xy;

    // Time varying pixel color
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    // Output to screen
    fragColor = vec4(col,1.0);
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

uniform sampler2D iChannel0;
uniform sampler2D iChannel1;
uniform sampler2D iChannel2;
uniform sampler2D iChannel3;


out vec4 outColor;

// mainImage
{main_image}

void main() {{
    mainImage(outColor, gl_FragCoord.xy);
}}"#,
        main_image = main_image
    )
}
