#version 330 core
layout(location = 0) in vec3 iPos;

uniform mat4 uModel;

void main() {
  gl_Position = uModel * vec4(iPos, 1.0);
}
