#version 330 core

out vec4 fColor;

void main() {
  // sit back and let OpenGL render the models and spit out those
  // sweet sweet depth values
  fColor = vec4(1.0, 0.0, 1.0, 1.0);
}
