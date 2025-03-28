#version 100

precision mediump float;

varying vec2 uv;
varying vec3 fragNormal;

uniform sampler2D Texture;

const vec3 lightDir = normalize(vec3(0.2, 0.8, -1.0));

void main() {
    vec4 texColor = texture2D(Texture, uv);

    float lighting = max(dot(normalize(fragNormal), lightDir), 0.5);

    gl_FragColor = vec4(texColor.rgb * lighting, texColor.a);
}
