#version 330 core

in vec2 vTex;

uniform sampler2D uTexture;

out vec4 fColor;

void main() {
  //float d = texture(uTexture, vTex).r;
  vec3 color = texture(uTexture, vTex).rgb;
  fColor = vec4(color, 1.0);
}
