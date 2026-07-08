#version 330 core

in vec3 vTexPos;

uniform samplerCube uSkybox;

out vec4 fColor;

void main() {
  fColor = texture(uSkybox, vTexPos);
}
