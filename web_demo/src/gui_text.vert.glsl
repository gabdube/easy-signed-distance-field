#version 300 es

layout(location=0) in vec2 inPosition;
layout(location=1) in vec2 inUv;

out vec2 fragUv;

void main() {
  fragUv = inUv;
  gl_Position = vec4(inPosition, 0.0, 1.0);
}
