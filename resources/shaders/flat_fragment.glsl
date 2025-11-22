#version 100

precision mediump float;

varying vec2 uv;
varying vec3 facePosition;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv);
}
