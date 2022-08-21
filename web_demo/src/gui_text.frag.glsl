#version 300 es
precision highp float;
precision highp int;

in vec2 fragUv;

layout(location=0) out vec4 outColor;

uniform float midValue;
uniform sampler2D fontAtlas;

void main(){
  float d = texture(fontAtlas, fragUv).r;
  float s = d - midValue;
  float v = s / fwidth( s );
  float a = clamp( v + 0.5, 0.0, 1.0 );
  if (a < 0.001) {
    discard;
  }

  outColor = vec4(0.0, 0.0, 0.0, a);
}
