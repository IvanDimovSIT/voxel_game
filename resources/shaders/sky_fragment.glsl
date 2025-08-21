#version 100

precision mediump float;

varying vec2 uv;
varying vec3 facePosition;

uniform sampler2D Texture;

const float fadeFactor = 0.05;

void main() {
    vec4 sample = texture2D(Texture, uv);
    float alpha = min(clamp(-facePosition.z*fadeFactor, 0.0, 1.0), sample.a);
    gl_FragColor = vec4(sample.rgb, alpha);
}