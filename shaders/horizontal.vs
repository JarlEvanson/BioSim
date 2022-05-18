#version 330 core

layout (location = 0) in float pos;

uniform int height;

void main() {
    float convert = float(gl_InstanceID);
    gl_Position = vec4( pos, convert / (float(height) / 2) - 1.0, 0.0, 1.0);
    
}