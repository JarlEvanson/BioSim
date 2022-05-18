#version 330 core

layout (location = 0) in vec2 vertices;
layout (location = 1) in vec2 offset;
layout (location = 2) in vec3 color;

out vec3 outColor;

uniform int width;
uniform int height;

void main() {
    outColor = color;


    gl_Position = vec4(
        vertices.x / float(width) * 2.0  + offset.x, 
        vertices.y / float(height) * 2.0 + offset.y, 
        0.0,
        1.0
    );
    
}