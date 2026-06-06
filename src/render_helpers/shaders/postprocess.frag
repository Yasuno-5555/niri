uniform float noise;
uniform float saturation;
uniform vec4 bg_color;

uniform float edge_highlight;
uniform float specular;
uniform float bloom;
// Note: time is declared in clipped_surface.frag (concatenated into the same program)

// Sin-less white noise by David Hoskins (MIT License).
// https://www.shadertoy.com/view/4djSRW
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

vec3 saturate(vec3 color, float sat) {
    const vec3 w = vec3(0.2126, 0.7152, 0.0722);
    return mix(vec3(dot(color, w)), color, sat);
}

vec4 postprocess(vec4 color) {
    if (saturation != 1.0) {
        color.rgb = saturate(color.rgb, saturation);
    }

    if (noise > 0.0) {
        vec2 uv = gl_FragCoord.xy;
        color.rgb += (hash12(uv) - 0.5) * noise;
    }

    // Mix bg_color behind the texture (both premultiplied alpha).
    color = color + bg_color * (1.0 - color.a);

    // Apply liquid specular, Fresnel rim light, and bloom
    if (liquid > 0.0) {
        vec3 coords_geo = input_to_geo * vec3(v_coords, 1.0);
        vec2 centered = coords_geo.xy - vec2(0.5);

        // Fresnel approximation: edges glow, center is clear.
        // Uses distance from center — natural glass look.
        float dist = length(centered) * 1.414; // normalize to [0, ~1] range
        float fresnel = pow(clamp(dist, 0.0, 1.0), 2.5);

        // 1. FRESNEL RIM LIGHT — smooth edge glow
        // Replaces the old hard border-distance check.
        float rim = fresnel * edge_highlight;
        color.rgb += rim * 0.5;
        // Add white light wrap at the very edges
        float rim_soft = pow(fresnel, 0.6);
        color.rgb = mix(color.rgb, color.rgb + vec3(0.15), rim_soft * edge_highlight);

        // 2. SPECULAR HIGHLIGHT — directional light with subtle animation
        // Animated light direction sweeps slowly over time
        vec2 light_dir_2d = normalize(vec2(
            sin(time * 0.25) * 0.4 - 0.3,
            cos(time * 0.3) * 0.35 - 0.5
        ));
        vec3 light_dir = normalize(vec3(light_dir_2d, 0.7));

        // Compute surface normal (approximate glass curvature)
        vec2 normal_2d = normalize(centered) * (1.0 - length(centered) * 0.9);
        vec3 normal = normalize(vec3(normal_2d, 1.0 - length(normal_2d)));

        // Specular with Fresnel weighting: glossy highlights at glancing angles
        float ndotl = max(0.0, dot(normal, light_dir));
        float spec = pow(ndotl, 48.0) * specular * (0.3 + fresnel * 0.7);
        // Add a broader specular lobe for softer light wrap
        float spec_soft = pow(ndotl, 12.0) * specular * 0.15 * fresnel;
        color.rgb += vec3(spec + spec_soft);

        // 3. BLOOM — subtle light bleed from bright areas
        if (bloom > 0.0) {
            float luminance = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
            float bloom_factor = smoothstep(0.5, 1.0, luminance) * bloom * fresnel;
            color.rgb += color.rgb * bloom_factor * 0.25;
        }
    }

    return color;
}
