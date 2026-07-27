#version 330 core

#define PI 3.14159265359

in vec3 vTexPos;

uniform sampler2D uSkybox;

out vec4 fColor;

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 cartesian_to_uv(vec3 cartesian) {
  vec2 uv = vec2(atan(cartesian.z, cartesian.x), asin(cartesian.y));
  uv *= invAtan;
  uv += 0.5;
  uv.y *= -1;
  return uv;
}

void main() {
  vec3 color = texture(uSkybox, cartesian_to_uv(normalize(vTexPos))).rgb;
  fColor = vec4(color, 1.0);
}
