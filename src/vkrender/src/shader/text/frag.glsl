#version 450

layout(location = 0) in vec4 color_in;
layout(location = 1) in vec2 tex_coord;
layout(location = 0) out vec4 color_out;

layout(set = 1, binding = 0) uniform sampler2D tex;

void main() {
	color_out = texture(tex, tex_coord) * color_in;
}
