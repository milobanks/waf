#version 460

%include res/shaders/h_vertex.vert

layout(binding = 0) uniform CameraData { Camera camera; };

layout(location = 0) in vec3 model_matrix_0;
layout(location = 1) in vec3 model_matrix_1;
layout(location = 5) in vec4 model_matrix_5;
layout(location = 6) in vec4 model_matrix_6;
layout(location = 7) in vec4 model_matrix_7;
layout(location = 8) in vec4 model_matrix_8;
layout(location = 0) smooth out vec3 vertex_color;

void main() {
    VertexOutput vertex_out = VertexOutput(vec4(0.0), vec3(0.0));
    VertexInput model = VertexInput(model_matrix_0, model_matrix_1);
    InstanceInput instance = InstanceInput(model_matrix_5, model_matrix_6, model_matrix_7, model_matrix_8);
    mat4x4 model_matrix = mat4x4(instance.model_matrix_0_, instance.model_matrix_1_, instance.model_matrix_2_, instance.model_matrix_3_);

    vertex_out.color = model.color;
    vertex_out.clip_position = ((camera.view_proj * model_matrix) * vec4(model.position, 1.0));
    vertex_color = vertex_out.color;

    gl_Position = vertex_out.clip_position;
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);

    return;
}

