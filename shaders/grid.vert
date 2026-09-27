#version 140
            in vec3 position;
            uniform mat4 view_proj;
            void main() {
                gl_Position = view_proj * vec4(position, 1.0);
            }