#version 330 core

uniform float uExposure;
uniform float uGamma;

in vec4 fColor;
out vec4 oColor;

vec3 hdrToSdr(vec3 hdrColor);
vec3 gammaCorrection(vec3 sdrColor);

vec3 hdrToSdr(vec3 hdrColor) {
  return vec3(1.0) - exp(-hdrColor * uExposure);
}

vec3 gammaCorrection(vec3 sdrColor) {
  return pow(sdrColor, vec3(1.0 / uGamma));
}
