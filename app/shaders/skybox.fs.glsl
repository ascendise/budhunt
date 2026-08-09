#version 330 core

#define PI 3.14159265359

in vec3 vTexPos;

uniform sampler2D uSkybox;

out vec4 fColor;

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 cartesianToUv(vec3 cartesian) {
  vec2 uv = vec2(atan(cartesian.z, cartesian.x), asin(cartesian.y));
  uv *= invAtan;
  uv += 0.5;
  uv.y *= -1;
  return uv;
}
vec3 hdrToSdr(vec3 hdrColor);
vec3 gammaCorrection(vec3 sdrColor);

void main() {
  vec3 color = texture(uSkybox, cartesianToUv(normalize(vTexPos))).rgb;
  color = hdrToSdr(color);
  color = gammaCorrection(color);
  fColor = vec4(color, 1.0);
}
