#version 100

attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;
varying vec3 facePosition;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    facePosition = position;
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}