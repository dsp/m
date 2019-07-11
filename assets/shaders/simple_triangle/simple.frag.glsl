#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out vec4 outColor;

const float START = -10.0;
const float END = 10.0;
const float STOP_THRESHOLD = 0.0001;
const int MAX_MARCHING_STEPS = 512;
const float PI = 3.14159265359;
const float DEG_TO_RAD = PI / 180.0;

float sdf_sphere(vec3 p) {
    return length(p) - .5;
}

vec3 ray_dir(float fov, vec2 size, vec2 pos) {
    vec2 xy = pos - (size * 0.5);
    float cot_half_fov = tan((90.0 - fov * 0.5) * DEG_TO_RAD);
    float z = size.y * 0.5 * cot_half_fov;

    return normalize(vec3(xy, -z));
}

bool ray_march(vec3 origin, vec3 dir) {
    float depth = START;
    for (int i = 0; i < MAX_MARCHING_STEPS; i++) {
        vec3 v = origin + dir * depth;
        float dist = sdf_sphere(v);
        if (dist < STOP_THRESHOLD) {
            return true;
        }

        depth += dist;

        if (depth >= END) {
            return false;
        }
    }

    return false;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord ) {
//    vec3 dir = ray_dir(45.0, iResolution.xy, fragCoord.xy);
    vec3 dir = ray_dir(45.0, vec2(1920.0, 2160.0), fragCoord.xy);
    vec3 eye = vec3(0.0, 0.0, 3.5);
    float color = 0.0;
    if (!ray_march(eye, dir)) {
        color = 1.0;
    }
    fragColor = vec4(color, color, color, 1.0);
}

void main() {
	mainImage(outColor, gl_FragCoord.xy);
}

// vim: ts=4 sw=4 sts=4

