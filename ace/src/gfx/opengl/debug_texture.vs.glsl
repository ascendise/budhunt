#version 330 core
layout(location = 0) in vec3 iPos;
layout(location = 2) in vec2 iTexPos;

out vec2 vTexPos;

void main() {
  gl_Position = vec4(iPos, 1.0);
  vTexPos = iTexPos;
}
