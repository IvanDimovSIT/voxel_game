#version 100

precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;

void main() {
    vec4 color = texture2D(Texture, uv);
    gl_FragColor = vec4(color.rgb * 0.8, color.a);
}