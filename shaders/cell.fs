#version 330 core

out vec4 FragColor;

in vec3 outColor;

void main() {
    FragColor = vec4(1.0, 0.0, 0.0, 1.0);//vec4(outColor, 1.0);
}
