#version 100

precision mediump float;

varying vec2 uv;
varying vec3 fragNormal;
varying vec3 facePosition;

uniform sampler2D Texture;
uniform vec3 cameraPos;
uniform vec3 cameraTarget;

const vec3 lightDir = normalize(vec3(0.2, 0.8, -1.0));
const float reflectionIntensity = 0.05;
const float ambient = 0.4;
const float specularStrength = 0.25;
const float dropShadowRadius = 0.4;
const float dropShadowLight = 0.2;


void main() {
    vec4 texColor = texture2D(Texture, uv);
    vec3 normal = normalize(fragNormal);
    
    float diffuse = max(dot(normal, lightDir), 0.0);
    if (facePosition.z > cameraPos.z && 
        distance(vec2(cameraPos.x, cameraPos.y), vec2(facePosition.x, facePosition.y)) < dropShadowRadius) {
        diffuse = dropShadowLight;
    }

    float lighting = ambient + diffuse * (1.0 - ambient);
    
    vec3 viewDir = normalize(cameraPos - facePosition);
    vec3 reflectDir = reflect(-lightDir, normal);
    float specular = pow(max(dot(reflectDir, viewDir), 0.0), 32.0) * specularStrength;
    
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), 3.0);
    float rim = fresnel * reflectionIntensity;
    
    vec3 finalColor = texColor.rgb * lighting + vec3(specular) + vec3(rim);
    
    gl_FragColor = vec4(finalColor, texColor.a);
}