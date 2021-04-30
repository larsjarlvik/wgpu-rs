layout(local_size_x = 16, local_size_y = 16) in;
layout (set = 0, binding = 0, rgba32f) uniform image2D elevation_normal_texture;

layout(set=1, binding=0) uniform ComputeData {
    float u_sea_level;
    float u_horizontal_scale;
    float u_vertical_scale;
    uint u_size;
    uint u_octaves;
    uint u_current_stage;
    uint u_stage_count;
};
