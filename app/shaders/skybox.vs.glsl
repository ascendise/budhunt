#version 330 core
layout(location = 0) in vec3 iPos;

uniform mat4 uView;
uniform mat4 uProjection;

out vec3 vTexPos;

void main() {
  vec4 pos = uProjection * uView * vec4(iPos, 1.0);
  gl_Position = pos.xyww;
  vTexPos = iPos;
}
