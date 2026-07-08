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

uniform Material uMaterial;
#define MAX_POINT_LIGHTS 64
uniform int uPointLightsSize;
uniform PointLight uPointLights[MAX_POINT_LIGHTS];

in vec3 vFragPos;
in vec3 vNormal;
in vec2 vTexPos;
out vec4 fColor;

vec3 calculateRadiance(vec3 albedo, float metallic, float roughness, float ao);
/// Cook-Torrance-BRDF
BrdfResult brdf(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness, vec3 albedo, float metallic);
/// Trowbridge-Reitz GGX
float normalDistribution(vec3 normal, vec3 halfway, float roughness);
/// Schlick-Beckmann Approximation
vec3 fresnel(float cosTheta, vec3 albedo, float metallic);
float geometry(vec3 normal, vec3 viewDirection, vec3 lightDirection, float roughness);
/// Schlick-GGX
float geometryGGX(vec3 normal, vec3 direction, float roughness);

const float PI = 3.14159265;

void main() {
  vec3 albedo_raw = texture(uMaterial.albedo, vTexPos).rgb;
  vec3 albedo = pow(albedo_raw, vec3(2.2));
  float metallic = texture(uMaterial.metallicRoughnessAo, vTexPos).b;
  float roughness = texture(uMaterial.metallicRoughnessAo, vTexPos).g;
  float ao = texture(uMaterial.metallicRoughnessAo, vTexPos).r;
  vec3 radiance = calculateRadiance(albedo, metallic, roughness, ao);
  vec3 ambient = vec3(0.03) * albedo * ao;
  vec3 color = ambient + radiance;
  color = color / (color + vec3(1.0));
  color = pow(color, vec3(1.0 / 2.2));
  fColor = vec4(color, 1.0);
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
  return geometryGGX(normal, viewDirection, roughness) * geometryGGX(normal, lightDirection, roughness);
}

float geometryGGX(vec3 normal, vec3 direction, float roughness) {
  float alignment = max(dot(normal, direction), 0.0);
  return alignment / (alignment * (1.0 - roughness) + roughness);
}
