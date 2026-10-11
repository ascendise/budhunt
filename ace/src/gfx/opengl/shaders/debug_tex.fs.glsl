#version 330 core

in vec2 vTex;

uniform sampler2D uTexture;

out vec4 fColor;

void main() {
  float d = texture(uTexture, vTex).r;
  fColor = vec4(d, d, d, 1.0);
}
