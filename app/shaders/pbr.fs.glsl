#version 330 core
struct Material {
  sampler2D albedo;
  sampler2D metallicRoughnessAo;
};

struct PointLight {
  vec3 color;
  vec3 position;
};

struct BrdfResult {
  vec3 specular;
  vec3 diffuse;
};

#define PREFILTER_MIP_LEVELS 11.0
uniform sampler2D uIrradianceMap;
uniform sampler2D uPrefilterMap;
uniform sampler2D uBrdfLut;

uniform Material uMaterial;

#define MAX_POINT_LIGHTS 64
uniform int uPointLightsSize;
uniform PointLight uPointLights[MAX_POINT_LIGHTS];

in vec3 vFragPos;
in vec3 vNormal;
in vec2 vTexPos;
out vec4 fColor;

vec3 calculateAmbience(vec3 albedo, float metallic, float roughness, float ao);
vec3 fresnelIbl(float cosTheta, vec3 F0, float roughness);
vec2 cartesianToUv(vec3 cartesian);
vec3 calculateRadiance(vec3 albedo, float metallic, float roughness, float ao);
/// Cook-Torrance-BRDF
BrdfResult brdf(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness, vec3 albedo, float metallic);
/// Trowbridge-Reitz GGX
float normalDistribution(vec3 normal, vec3 halfway, float roughness);
/// Schlick-Beckmann Approximation
vec3 fresnel(float cosTheta, vec3 albedo, float metallic);
float geometry(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness);
/// Schlick-GGX
float geometryGgx(vec3 normal, vec3 direction, float roughness);
vec3 hdrToSdr(vec3 hdrColor);

const float PI = 3.14159265;

void main() {
  vec3 albedo = texture(uMaterial.albedo, vTexPos).rgb;
  float metallic = texture(uMaterial.metallicRoughnessAo, vTexPos).b;
  float roughness = texture(uMaterial.metallicRoughnessAo, vTexPos).g;
  float ao = texture(uMaterial.metallicRoughnessAo, vTexPos).r;
  vec3 ambient = calculateAmbience(albedo, metallic, roughness, ao); //ambient too bright?
  vec3 radiance = calculateRadiance(albedo, metallic, roughness, ao);
  vec3 color = ambient + radiance;
  color = hdrToSdr(color);
  fColor = vec4(color, 1.0);
}

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 cartesianToUv(vec3 cartesian) {
  vec2 uv = vec2(atan(cartesian.z, cartesian.x), asin(cartesian.y));
  uv *= invAtan;
  uv += 0.5;
  uv.y *= -1;
  return uv;
}
vec3 fresnelIbl(float cosTheta, vec3 F0, float roughness) {
  return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

vec3 sRgbToLinear(vec3 sRgb) {
  return pow(sRgb, vec3(2.2));
}

vec3 calculateAmbience(vec3 albedo, float metallic, float roughness, float ao) {
  vec3 n = normalize(vNormal);
  vec3 v = normalize(-vFragPos);
  vec3 r = reflect(-v, n);
  vec2 uv = cartesianToUv(r);
  float nv = max(dot(n, v), 0.0);
  vec3 f0 = mix(vec3(0.04), albedo, metallic);
  vec3 f = fresnelIbl(nv, f0, roughness);
  vec3 kS = f;
  vec3 kD = 1.0 - kS;
  kD *= 1.0 - metallic;
  vec3 irradiance = texture(uIrradianceMap, cartesianToUv(n)).rgb;
  vec3 diffuse = irradiance * albedo;
  vec3 prefilterColor = textureLod(uPrefilterMap, uv, roughness * PREFILTER_MIP_LEVELS).rgb;
  vec2 brdf = texture(uBrdfLut, vec2(nv, roughness)).rg;
  vec3 specular = prefilterColor * (f * brdf.x + brdf.y);
  return (kD * diffuse + specular) * ao;
}

vec3 calculateRadiance(vec3 albedo, float metallic, float roughness, float ao) {
  vec3 normal = normalize(vNormal);
  vec3 viewDirection = normalize(-vFragPos);
  vec3 radianceSum = vec3(0.0);
  for (int i = 0; i < min(uPointLightsSize, MAX_POINT_LIGHTS); i++) {
    PointLight light = uPointLights[i];
    vec3 lightDirection = light.position - vFragPos;
    float distance = length(lightDirection);
    lightDirection = normalize(lightDirection);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance = light.color * attenuation;
    BrdfResult brdf = brdf(normal, viewDirection, lightDirection, roughness, albedo, metallic);
    radianceSum += (brdf.diffuse * albedo / PI + brdf.specular) * radiance * max(dot(normal, lightDirection), 0.0);
  }
  return radianceSum;
}

BrdfResult brdf(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness, vec3 albedo, float metallic) {
  vec3 halfwayDirection = normalize(lightDirection + viewDirection);
  float normalDistribution = normalDistribution(normal, halfwayDirection, roughness);
  vec3 fresnel = fresnel(max(dot(halfwayDirection, viewDirection), 0.0), albedo, metallic); //maybe
  float geometry = geometry(normal, viewDirection, lightDirection, roughness);
  vec3 nfg = normalDistribution * fresnel * geometry;
  float reflectance = 4.0 * max(dot(normal, viewDirection), 0.0) * max(dot(normal, lightDirection), 0.0) + 0.0001;
  vec3 specular = nfg / reflectance;
  vec3 diffuse = vec3(1.0) - fresnel;
  diffuse *= 1.0 - metallic;
  return BrdfResult(specular, diffuse);
}

float normalDistribution(vec3 normal, vec3 halfway, float roughness) {
  roughness = pow(roughness, 4.0);
  float alignment = max(dot(normal, halfway), 0.0);
  alignment = pow(alignment, 2.0);
  alignment = (alignment * (roughness - 1.0) + 1.0);
  return roughness / (pow(alignment, 2.0) * PI);
}

vec3 fresnel(float cosTheta, vec3 albedo, float metallic) {
  vec3 reflectance = vec3(0.04);
  reflectance = mix(reflectance, albedo, metallic);
  return reflectance + (1.0 - reflectance) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float geometry(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness) {
  roughness = pow(roughness + 1.0, 2.0) / 8.0;
  return geometryGgx(normal, viewDirection, roughness) * geometryGgx(normal, lightDirection, roughness);
}

float geometryGgx(vec3 normal, vec3 direction, float roughness) {
  float alignment = max(dot(normal, direction), 0.0);
  return alignment / (alignment * (1.0 - roughness) + roughness);
}
