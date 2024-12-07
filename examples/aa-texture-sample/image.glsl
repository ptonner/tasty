//modified from https://www.shadertoy.com/view/ltfXWS, tested a few different versions of this same function and this one seemed to have the nicest results
vec4 texture2DAA(sampler2D tex, vec2 uv) {
    vec2 texsize = vec2(textureSize(tex,0));
    vec2 uv_texspace = uv*texsize;
    vec2 seam = floor(uv_texspace+.5);
    uv_texspace = (uv_texspace-seam)/fwidth(uv_texspace)+seam;
    uv_texspace = clamp(uv_texspace, seam-.5, seam+.5);
    return texture(tex, uv_texspace/texsize);
}


void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord/iResolution.xx*(sin(iTime*.2)+1.5)-vec2(.5, .5);
    uv *= mat2(sin(iTime * 0.1), cos(iTime * 0.1), -cos(iTime * 0.1), sin(iTime * 0.1));


    if(fragCoord.x/iResolution.x < .5){
        fragColor = texture2DAA(iChannel0, uv); //anti aliased scaling (iChannel0 is set to "linear" scaling)
    } else {
        fragColor = texture(iChannel1, uv); //nearest neighbor scaling (iChannel1 is set to "nearest" scaling)
    }
}
