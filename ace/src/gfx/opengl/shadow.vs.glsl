#version 330 core

layout(location = 0) in vec3 iPosition;

uniform mat4 uLightViewProjection;
uniform mat4 uModel;

void main() {
  gl_Position = uLightViewProjection * uModel * vec4(iPosition, 1.0);
}
