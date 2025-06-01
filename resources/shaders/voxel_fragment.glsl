#version 100

precision mediump float;

varying vec2 uv;
varying vec3 fragNormal;
varying vec3 facePosition;

uniform sampler2D Texture;
uniform vec3 cameraPos;
uniform vec3 cameraTarget;
uniform float fogFar;
uniform float fogNear;

const vec3 lightDir = normalize(vec3(0.2, 0.8, -1.0));
const float reflectionIntensity = 0.05;
const float ambient = 0.4;
const float specularStrength = 0.25;
const float dropShadowRadius = 0.4;
const float dropShadowLight = 0.2;
const vec3 fogBaseColor = vec3(0.83, 0.69, 0.51);


void main() {
    vec4 texColor = texture2D(Texture, uv);
    vec3 normal = normalize(fragNormal);
    
    float diffuse = max(dot(normal, lightDir), 0.0);
    if (facePosition.z > 0.0 && 
        distance(vec2(0.0, 0.0), vec2(facePosition.x, facePosition.y)) < dropShadowRadius) {
        diffuse = dropShadowLight;
    }

    float lighting = ambient + diffuse * (1.0 - ambient);
    
    vec3 viewDir = normalize(-facePosition);
    vec3 reflectDir = reflect(-lightDir, normal);
    float specular = pow(max(dot(reflectDir, viewDir), 0.0), 32.0) * specularStrength;
    
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), 3.0);
    float rim = fresnel * reflectionIntensity;
    
    vec3 preFogColor = texColor.rgb * lighting + vec3(specular) + vec3(rim);

    float distanceToFace = length(facePosition);
    float fogFactor = clamp((fogFar - distanceToFace) / (fogFar - fogNear), 0.0, 1.0);
    vec3 finalColor = fogBaseColor * (1.0 - fogFactor) + preFogColor * fogFactor;

    gl_FragColor = vec4(finalColor, texColor.a);
}