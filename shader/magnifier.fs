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

// Output fragment color
out vec4 finalColor;

void main()
{
    // Convert texture coordinates to screen coordinates
    vec2 screenCoord = fragTexCoord * vec2(textureWidth, textureHeight);

    // Calculate distance from the center of the magnifier
    float dist = distance(screenCoord, center);

    // Magnification factor
    float magnification = 1.2;
    
    vec4 texelColor;

    // Apply magnification only if within the radius
    if (dist < radius) {
        // Calculate the magnification effect with a falloff
        float falloff = smoothstep(0.0, radius, dist);
        float currentMagnification = mix(magnification, 1.0, falloff);
        
        // Apply magnification
        vec2 magnifiedCoord = (screenCoord - center) / currentMagnification + center;
        vec2 magnifiedTexCoord = magnifiedCoord / vec2(textureWidth, textureHeight);
        vec2 zoomedOutCoord = (screenCoord - center) * (1.0 / currentMagnification) + center;
        
        texelColor = texture(texture0, magnifiedTexCoord);
    } else {
        // Outside the circle, show the original texture
        texelColor = texture(texture0, fragTexCoord);
    }
    
    // Smooth transition at the edge of the circle
    float edgeSmoothness = 2.0; // The width of the smooth edge in pixels
    float alpha = smoothstep(radius, radius - edgeSmoothness, dist);

    // Mix the magnified texture with a semi-transparent black for the "spotlight" effect
    finalColor = mix(vec4(0.0, 0.0, 0.0, 0.75), texelColor, alpha);

    // Apply texture tint (colDiffuse)
    finalColor = finalColor * colDiffuse;
}
