#version 450

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 pos;
layout(location = 2) in vec2 tex_coord;
layout(location = 0) out vec4 color_out;
layout(location = 1) out vec2 tex_coord_out;

void main() {
	gl_Position = vec4(pos, 0.0, 1.0);
	color_out = color;
	tex_coord_out = tex_coord;
}
