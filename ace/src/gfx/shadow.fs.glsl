#version 330 core

in vec4 FragPos;

uniform vec3 uLightPos;
uniform float uFrustumFar;

out vec4 fColor;

void main() {
  //float lightDistance = length(FragPos.xyz - uLightPos);
  gl_FragDepth = 1.0; //lightDistance / uFrustumFar;
}
