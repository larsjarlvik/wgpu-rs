#ifndef CAMERA_INITIALIZED
    #ifndef CAMERA_SET
        #define CAMERA_SET 0
    #endif

    layout(set=CAMERA_SET, binding=0) uniform CameraUniforms {
        mat4 view_proj;
        vec3 eye_pos;
        float z_near;
        vec3 look_at;
        float z_far;
        vec4 clip;
        vec2 viewport_size;
    } cam;
#endif

#define CAMERA_INITIALIZED 1