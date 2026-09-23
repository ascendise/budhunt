#version 330 core

in vec4 FragPos;

uniform vec3 uLightPos;
uniform float uFrustumFar;

void main() {
  //float lightDistance = length(FragPos.xyz - uLightPos);
  gl_FragDepth = 0.5; //lightDistance / uFrustumFar;
}
