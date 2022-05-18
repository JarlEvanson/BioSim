#version 330 core

layout (location = 0) in float pos;

uniform int width; 

void main() {
    float convert = float(gl_InstanceID);
    gl_Position = vec4( convert / (float(width) / 2) - 1.0, pos, 0.0, 1.0);
    
}