layout(local_size_x = 16, local_size_y = 1) in;

struct Vertex {
    float x; float y; float z;
    float nx; float ny; float nz;
};

layout(std430, set=0, binding=0) buffer Src {
    Vertex comp_vertices[];
};
layout(set=1, binding=0) uniform ComputeData {
    float u_sea_level;
    float u_horizontal_scale;
    float u_vertical_scale;
    uint u_size;
    uint u_octaves;
    uint u_current_stage;
    uint u_stage_count;
};