#version 330 core

in vec2 vTexPos;

uniform sampler2D uTexture;

out vec4 fColor;

void main() {
  float depth = texture(uTexture, vTexPos).r;
  fColor = vec4(vec3(depth), 1.0);
}
