#version 330 core
layout(location = 0) in vec3 iPos;
layout(location = 1) in vec3 iNormal;
layout(location = 2) in vec2 iTexPos;

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;
uniform mat4 uNormal;

out vec3 vFragPos;
out vec3 vNormal;
out vec2 vTexPos;

void main() {
  gl_Position = uProjection * uView * uModel * vec4(iPos, 1.0);
  vFragPos = vec3(uView * uModel * vec4(iPos, 1.0));
  vNormal = mat3(uNormal) * iNormal;
  vTexPos = iTexPos;
}
