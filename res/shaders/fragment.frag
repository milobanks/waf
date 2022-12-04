#version 460

%include res/shaders/h_vertex.vert

layout(location = 0) smooth in vec3 vertex_color;
layout(location = 0) out vec4 vertex_clip_position;

void main() {
    VertexOutput vertex_out = VertexOutput(gl_FragCoord, vertex_color);

    vertex_clip_position = vec4(vertex_out.color, 1.0);

    return;
}

