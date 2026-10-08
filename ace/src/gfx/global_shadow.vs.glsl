#version 330 core
layout(location = 0) in vec3 iPos;

uniform mat4 uModel;
uniform mat4 uLightSpaceTransform;

void main() {
  gl_Position = uLightSpaceTransform * uModel * vec4(iPos, 1.0);
}
