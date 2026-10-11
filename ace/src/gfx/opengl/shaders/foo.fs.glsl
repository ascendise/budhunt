#version 330 core

in vec4 FragPos;

uniform vec3 uColor;

out vec4 fColor;

void main() {
  fColor = vec4(uColor, 1.0);
}
