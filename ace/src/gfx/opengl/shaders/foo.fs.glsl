#version 330 core

in vec4 FragPos;

uniform vec3 uColor;

out vec4 fColor;

void main() {
  //fColor = vec4(uColor, 1.0);
  fColor = vec4(1.0, 0.0, 1.0, 1.0);
}
