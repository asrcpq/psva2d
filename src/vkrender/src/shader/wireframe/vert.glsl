#version 450

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 pos;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform Data {
	vec2 c;
	vec2 r;
} uniforms;

void main() {
	vec2 pos_proj = (pos - uniforms.c) * uniforms.r;
	gl_Position = vec4(pos_proj, 0.0, 1.0);
	out_color = color;
}
