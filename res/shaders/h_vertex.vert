precision highp float;
precision highp int;

struct Camera {
    vec4 view_pos;
    mat4x4 view_proj;
};

struct VertexInput {
    vec3 position;
    vec3 color;
};

struct VertexOutput {
    vec4 clip_position;
    vec3 color;
};

struct InstanceInput {
    vec4 model_matrix_0_;
    vec4 model_matrix_1_;
    vec4 model_matrix_2_;
    vec4 model_matrix_3_;
};

