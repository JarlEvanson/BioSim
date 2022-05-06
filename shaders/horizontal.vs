#version 330 core

layout (location = 0) in float pos;

void main() {
    float convert = float(gl_InstanceID);
    gl_Position = vec4( pos, convert  / 50.0 - 1.0, 0.0, 1.0);
    
}