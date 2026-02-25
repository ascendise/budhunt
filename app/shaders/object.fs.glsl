#version 330 core
struct Material {
  sampler2D diffuse;
  sampler2D specular;
  sampler2D emission;
  float shininess;
};

struct Light {
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

struct DirectionalLight {
  vec3 direction;
  Light light;
};

struct PointLight {
  vec3 position;
  float constant;
  float linear;
  float quadratic;
  Light light;
};

struct SpotLight {
  vec3 position;
  vec3 direction;
  float cutoff;
  float outerCutoff;
  Light light;
};

uniform Material uMaterial;
#define MAX_DIR_LIGHTS 16
uniform int uDirectionalLightsSize;
uniform DirectionalLight uDirectionalLights[MAX_DIR_LIGHTS];
#define MAX_POINT_LIGHTS 64
uniform int uPointLightsSize;
uniform PointLight uPointLights[MAX_POINT_LIGHTS];
#define MAX_SPOT_LIGHTS 16
uniform int uSpotLightsSize;
uniform SpotLight uSpotLights[MAX_SPOT_LIGHTS];

in vec3 vFragPos;
in vec3 vNormal;
in vec2 vTexPos;
out vec4 fColor;

Light calcDirectionalLight(DirectionalLight light, vec3 fragPos, vec3 normal);
Light calcLight(Light light, vec3 lightDirection, vec3 viewDirection, vec3 normal);
Light calcPointLight(PointLight light, vec3 fragPos, vec3 normal);
Light callSpotLight(SpotLight light, vec3 fragPos, vec3 normal);

void main() {
  vec3 diffuse = texture(uMaterial.diffuse, vTexPos).rgb;
  vec3 specular = texture(uMaterial.specular, vTexPos).rgb;
  vec3 emission = texture(uMaterial.emission, vTexPos).rgb;
  vec3 normal = normalize(vNormal);
  vec3 material = vec3(0.0);
  Light light;
  for (int i = 0; i < min(uDirectionalLightsSize, MAX_DIR_LIGHTS); i++) {
    light = calcDirectionalLight(uDirectionalLights[i], vFragPos, normal);
    material += (light.ambient * diffuse)
        + (light.diffuse * diffuse)
        + (light.specular * specular);
  }
  for (int i = 0; i < min(uPointLightsSize, MAX_POINT_LIGHTS); i++) {
    light = calcPointLight(uPointLights[i], vFragPos, normal);
    material += (light.ambient * diffuse)
        + (light.diffuse * diffuse)
        + (light.specular * specular);
  }
  for (int i = 0; i < min(uSpotLightsSize, MAX_SPOT_LIGHTS); i++) {
    light = callSpotLight(uSpotLights[i], vFragPos, normal);
    material += (light.ambient * diffuse)
        + (light.diffuse * diffuse)
        + (light.specular * specular);
  }
  material += emission;
  fColor = vec4(material, 1.0);
}

Light calcDirectionalLight(DirectionalLight dirLight, vec3 fragPos, vec3 normal) {
  vec3 lightDirection = normalize(-dirLight.direction);
  vec3 viewDirection = normalize(-fragPos);
  return calcLight(dirLight.light, lightDirection, viewDirection, normal);
}

Light calcLight(Light light, vec3 lightDirection, vec3 viewDirection, vec3 normal) {
  vec3 ambient = light.ambient;
  float diff = max(dot(normal, lightDirection), 0.0);
  vec3 diffuse = light.diffuse * diff;
  vec3 reflectDirection = reflect(-lightDirection, normal);
  float spec = pow(max(dot(viewDirection, reflectDirection), 0.0), uMaterial.shininess);
  vec3 specular = light.specular * spec;
  return Light(ambient, diffuse, specular);
}

Light calcPointLight(PointLight pointLight, vec3 fragPos, vec3 normal) {
  float dist = length(pointLight.position - fragPos);
  float attenuation = 1.0 /
      (pointLight.constant + pointLight.linear * dist + pointLight.quadratic * (dist * dist));

  vec3 lightDirection = normalize(pointLight.position - fragPos);
  vec3 viewDirection = normalize(-fragPos);
  Light light = calcLight(pointLight.light, lightDirection, viewDirection, normal);
  return Light(light.ambient * attenuation, light.diffuse * attenuation, light.specular * attenuation);
}

Light callSpotLight(SpotLight spotLight, vec3 fragPos, vec3 normal) {
  vec3 lightDirection = normalize(spotLight.position - fragPos);
  float theta = dot(lightDirection, normalize(-spotLight.direction));
  float epsilon = spotLight.cutoff - spotLight.outerCutoff;
  float intensity = clamp((theta - spotLight.outerCutoff) / epsilon, 0.0, 1.0);
  vec3 viewDirection = normalize(-fragPos);
  Light light = calcLight(spotLight.light, lightDirection, viewDirection, normal);
  return Light(light.ambient * intensity, light.diffuse * intensity, light.specular * intensity);
}
