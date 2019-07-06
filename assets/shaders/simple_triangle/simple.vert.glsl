#version 450

vec2 positions[] = vec2[](
    vec2(-0.5,-0.5),  // top left
    vec2( 0.5,-0.5),  // top right
    vec2( 0.5, 0.5),  // bottom right
    vec2(-0.5, 0.5)   // bottom left
);

int indices[] = {
	0, 1, 2,
	0, 2, 3
};

void main() {
	int idx = indices[gl_VertexIndex];
    gl_Position = vec4(positions[idx], 0.0, 1.0);
}
