#version 100

precision highp float;

varying vec2 uv;
varying vec3 fragNormal;
varying vec3 facePosition;

uniform sampler2D Texture;
uniform sampler2D heightMap;
uniform vec3 cameraPos;
uniform vec3 cameraTarget;
uniform float lightLevel;

uniform float fogFar;
uniform float fogNear;
uniform vec3 fogBaseColorLight;
uniform vec3 fogBaseColorDark;

uniform int lightsCount;
uniform vec3 lights[64];

uniform int hasDynamicShadows;
uniform int showDropShadow;

// static world lighting
const vec3 lightDir = normalize(vec3(0.2, 0.8, -1.0));
const float reflectionIntensity = 0.05;
const float ambient = 0.4;
const float specularStrength = 0.25;

// player light and shadow
const float dropShadowRadius = 0.4;
const float dropShadowLight = 0.2;
const float playerLightStrength = 15.0;

// placed lamps
const float lampStrength = 6.0;

// dynamic shadows
const float dynamicShadowStrength = 0.6;
const float halfVoxelSize = 0.5;
const float areaSize = 16.0;
const float valuesInByte = 256.0;
const float maxAreasInShadowPerAxis = 32.0;

// calculates static lighting and drop shadow based on the time of day
float calculateDiffuseLight(vec3 normal, float shadowedLightLevel) {
    float diffuse = max(dot(normal, lightDir), 0.0);

    bool shouldDrawDropShadow = showDropShadow == 1 && 
        facePosition.z > 0.0 && 
        distance(vec2(0.0, 0.0), vec2(facePosition.x, facePosition.y)) < dropShadowRadius; 
    if (shouldDrawDropShadow) {
        diffuse = dropShadowLight * shadowedLightLevel;
    }

    return diffuse;
}

// draws the fog based on the fog uniforms and the time of day
vec3 addFog(vec3 preFogColor, float distanceToFace, float darkLevel) {
    vec3 fogBaseColor = fogBaseColorLight * lightLevel + fogBaseColorDark * darkLevel;
    float fogFactor = clamp((fogFar - distanceToFace) / (fogFar - fogNear), 0.0, 1.0);
    
    return fogBaseColor * (1.0 - fogFactor) + preFogColor * fogFactor;
}

// draws the player light at night
float addPlayerLight(float baseLight, float distanceToFace, float darkLevel) {
    float playerLightProximity = 1.0 - min(distanceToFace, playerLightStrength)/playerLightStrength;
    
    return min(baseLight + darkLevel * playerLightProximity * playerLightProximity, 1.0);
}

// draws placed lamps
float addLampLighting(float lighting) {
    for (int i = 0; i < lightsCount; i++) {
        vec3 directionToLight = lights[i] - facePosition;
        float distanceToLight = length(directionToLight);
        float brightness = clamp(1.0 - distanceToLight/lampStrength, 0.0, 1.0); 
        float facingLightCoef = (1.0 + dot(normalize(directionToLight), fragNormal))/2.0;    
        float blend = smoothstep(0.5, 1.5, distanceToLight);
        brightness *= mix(1.0, facingLightCoef, blend);
        lighting += brightness;
    }

    return min(lighting, 1.0);
}

// calculates dynamic shadows from the height map
float calculateAmountInShadow() {
    if (hasDynamicShadows == 0) {
        return 0.0;
    }

    const float fadeAmount = 0.005;
    const float offsetByNormal = 0.9;

    vec2 faceOffset = facePosition.xy + fragNormal.xy*offsetByNormal;
    vec2 cameraOffset = mod(cameraPos.xy + vec2(halfVoxelSize), areaSize);
    vec2 samplePos = (faceOffset + cameraOffset) / (areaSize * maxAreasInShadowPerAxis) + vec2(halfVoxelSize);
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
    float specular = pow(max(dot(reflectDir, viewDir), 0.0), 32.0) * specularStrength * lightLevel * (1.0 - amountInShadow);
    
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), 3.0);
    float rim = fresnel * reflectionIntensity;
    
    vec3 preFogColor = texColor.rgb * lighting + vec3(specular) + vec3(rim);

    vec3 finalColor = addFog(preFogColor, distanceToFace, darkLevel);

    gl_FragColor = vec4(finalColor, texColor.a);
}