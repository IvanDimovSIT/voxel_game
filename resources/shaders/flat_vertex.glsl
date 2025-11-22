#version 100

attribute vec3 position;
attribute vec2 texcoord;
attribute vec3 normal;

varying lowp vec2 uv;
varying lowp vec3 fragNormal;
varying vec3 facePosition;

uniform mat4 Model;
uniform mat4 Projection;
uniform	vec3 cameraPos;

void main() {
    facePosition = position - cameraPos;
    gl_Position = Projection * Model * vec4(facePosition, 1.0);
    uv = texcoord;
    fragNormal = normalize(mat3(Model) * normal);
}
