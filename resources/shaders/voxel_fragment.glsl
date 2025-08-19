#version 100

precision highp float;

varying vec2 uv;
varying vec3 fragNormal;
varying vec3 facePosition;

uniform sampler2D Texture;
uniform sampler2D heightMap;
uniform vec3 cameraPos;
uniform vec3 cameraTarget;
uniform float fogFar;
uniform float fogNear;
uniform float lightLevel;
uniform vec3 fogBaseColorLight;
uniform vec3 fogBaseColorDark;
uniform int lightsCount;
uniform vec3 lights[64];
uniform int hasDynamicShadows;

const vec3 lightDir = normalize(vec3(0.2, 0.8, -1.0));
const float reflectionIntensity = 0.05;
const float ambient = 0.4;
const float specularStrength = 0.25;
const float dropShadowRadius = 0.4;
const float dropShadowLight = 0.2;
const float playerLightStrength = 15.0;
const float lampStrength = 6.0;
const float dynamicShadowStrength = 0.6;

const float areaSize = 16.0;
const float valuesInByte = 256.0;
const float maxLoadedAreasPerAxis = 37.0;

float calculateDiffuseLight(vec3 normal, float shadowedLightLevel) {
    float diffuse = max(dot(normal, lightDir), 0.0);

    if (facePosition.z > 0.0 && 
        distance(vec2(0.0, 0.0), vec2(facePosition.x, facePosition.y)) < dropShadowRadius) {
        diffuse = dropShadowLight * shadowedLightLevel;
    }

    return diffuse;
}

vec3 addFog(vec3 preFogColor, float distanceToFace, float darkLevel) {
    vec3 fogBaseColor = fogBaseColorLight * lightLevel + fogBaseColorDark * darkLevel;
    float fogFactor = clamp((fogFar - distanceToFace) / (fogFar - fogNear), 0.0, 1.0);
    
    return fogBaseColor * (1.0 - fogFactor) + preFogColor * fogFactor;
}

float addPlayerLight(float baseLight, float distanceToFace, float darkLevel) {
    float playerLightProximity = 1.0 - min(distanceToFace, playerLightStrength)/playerLightStrength;
    
    return min(baseLight + darkLevel * playerLightProximity * playerLightProximity, 1.0);
}

float addLampLighting(float lighting) {
    for (int i = 0; i < lightsCount; i++) {
        float distanceToLight = length(facePosition - lights[i]);
        lighting += max(1.0 - distanceToLight/lampStrength, 0.0);
    }

    return min(lighting, 1.0);
}

float calculateAmountInShadow() {
    if (hasDynamicShadows == 0) {
        return 0.0;
    }

    const float fadeAmount = 0.005;

    vec2 samplePos = (facePosition.xy + fragNormal.xy + mod(cameraPos.xy + vec2(0.5), areaSize)) / (areaSize * maxLoadedAreasPerAxis) + vec2(0.5, 0.5);
    float sampledHeight = texture2D(heightMap, samplePos).r;
    
    float worldHeight = (cameraPos.z + facePosition.z) / valuesInByte;
    float edgeLow = worldHeight - fadeAmount;
    float edgeHigh = worldHeight + fadeAmount;
    
    return 1.0 - smoothstep(edgeLow, edgeHigh, sampledHeight); 
}

void main() {
    vec4 texColor = texture2D(Texture, uv);
    vec3 normal = normalize(fragNormal);
    float distanceToFace = length(facePosition);
    float amountInShadow = calculateAmountInShadow();
    
    float darkLevel = 1.0 - lightLevel;

    float diffuse = calculateDiffuseLight(normal, lightLevel);

    float sunLighting = min(lightLevel, 1.0) * (ambient + diffuse * (1.0 - ambient));
    sunLighting *= (1.0 - dynamicShadowStrength * amountInShadow);
    float lighting = addLampLighting(addPlayerLight(sunLighting, distanceToFace, darkLevel));

    vec3 viewDir = normalize(-facePosition);
    vec3 reflectDir = reflect(-lightDir, normal);
    float specular = pow(max(dot(reflectDir, viewDir), 0.0), 32.0) * specularStrength * lightLevel;
    
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), 3.0);
    float rim = fresnel * reflectionIntensity;
    
    vec3 preFogColor = texColor.rgb * lighting + vec3(specular) + vec3(rim);

    vec3 finalColor = addFog(preFogColor, distanceToFace, darkLevel);

    gl_FragColor = vec4(finalColor, texColor.a);
}