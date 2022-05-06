#version 330 core

layout (location = 0) in vec2 pos;

void main() {
    gl_Position = vec4( pos.x * 2.0 + 1.0, pos.y * 2.0 + 1.0, 0.0, 1.0);
    
}