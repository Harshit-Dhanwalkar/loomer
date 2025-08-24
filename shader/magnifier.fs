#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

// Custom uniforms for magnifier effect
uniform vec2 center;
uniform float radius;
uniform float textureWidth;
uniform float textureHeight;
uniform float magnification;

// Output fragment color
out vec4 finalColor;

void main()
{
    // Convert normalized texture coordinates to screen pixel coordinates
    vec2 screenCoord = fragTexCoord * vec2(textureWidth, textureHeight);

    // Calculate distance from the center of the magnifier
    float dist = distance(screenCoord, center);

    vec4 texelColor;

    // Apply magnification only if within the radius
    if (dist < radius) {
        vec2 centeredCoord = screenCoord - center;
        vec2 magnifiedCoord = centeredCoord / magnification;
        vec2 finalCoord = magnifiedCoord + center;
        vec2 magnifiedTexCoord = finalCoord / vec2(textureWidth, textureHeight);

        texelColor = texture(texture0, magnifiedTexCoord);
    } else {
        // Outside the circle, show the original texture
        texelColor = texture(texture0, fragTexCoord);
    }

    // Smooth transition at the edge of the circle
    float edgeSmoothness = 2.0;
    float alpha = smoothstep(radius, radius - edgeSmoothness, dist);

    finalColor = mix(vec4(0.0, 0.0, 0.0, 0.75), texelColor, alpha);
    finalColor = finalColor * colDiffuse;
}
