#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision highp float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

uniform float niri_scale;

uniform vec2 geo_size;
uniform vec4 corner_radius;
uniform mat3 input_to_geo;

uniform float liquid;
uniform float refraction;
uniform float chromatic_aberration;
uniform float time;

float niri_rounding_alpha(vec2 coords, vec2 size, vec4 corner_radius);
vec4 postprocess(vec4 color);

void main() {
    vec3 coords_geo = input_to_geo * vec3(v_coords, 1.0);

    vec4 color;

    if (liquid > 0.0 && refraction > 0.0) {
        // 1. Compute normal for refraction
        vec2 centered = coords_geo.xy - vec2(0.5);
        float dist = length(centered) * 1.414;

        // Subtle animated shimmer
        float shimmer = sin(dist * 8.0 - time * 1.5) * 0.015 +
                        cos(dist * 5.5 + time * 1.1) * 0.01;
        vec2 normal = normalize(centered) * (1.0 - length(centered) * 0.85) + shimmer;

        // Fresnel factor: edges distort more, center stays clear
        float fresnel = pow(clamp(dist, 0.0, 1.0), 2.0);

        // 2. DUAL-LAYER REFRACTION — glass thickness simulation
        // Near layer: subtle displacement (front glass surface)
        vec2 near_uv = v_coords + normal * refraction * 0.55;
        vec4 near_color = texture2D(tex, near_uv);

        // Far layer: stronger displacement (rear glass surface / backdrop)
        vec2 far_uv = v_coords + normal * refraction * 1.3;
        vec4 far_color = texture2D(tex, far_uv);

        // Blend near and far refractions
        vec4 refracted = mix(near_color, far_color, 0.4);

        // 3. CHROMATIC ABERRATION — only on the refracted component
        if (chromatic_aberration > 0.0) {
            float ca = chromatic_aberration * (0.6 + fresnel * 0.4);
            float r = texture2D(tex, far_uv - vec2(ca, 0.0)).r;
            float b = texture2D(tex, far_uv + vec2(ca, 0.0)).b;
            refracted.r = r;
            refracted.b = b;
        }

        // 4. Fresnel-weighted blend: edges = more refraction, center = clearer
        float blend_factor = fresnel * 0.85;
        color = mix(texture2D(tex, v_coords), refracted, blend_factor);
    } else if (liquid > 0.0 && chromatic_aberration > 0.0) {
        // CA without refraction
        float r = texture2D(tex, v_coords - vec2(chromatic_aberration, 0.0)).r;
        float g = texture2D(tex, v_coords).g;
        float b = texture2D(tex, v_coords + vec2(chromatic_aberration, 0.0)).b;
        color = vec4(r, g, b, 1.0);
    } else {
        color = texture2D(tex, v_coords);
    }

#if defined(NO_ALPHA)
    color = vec4(color.rgb, 1.0);
#endif

    color = postprocess(color);

    if (coords_geo.x < 0.0 || 1.0 < coords_geo.x || coords_geo.y < 0.0 || 1.0 < coords_geo.y) {
        // Clip outside geometry.
        color = vec4(0.0);
    } else {
        // Apply corner rounding inside geometry.
        color = color * niri_rounding_alpha(coords_geo.xy * geo_size, geo_size, corner_radius);
    }

    // Apply final alpha and tint.
    color = color * alpha;

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = vec4(0.0, 0.2, 0.0, 0.2) + color * 0.8;
#endif

    gl_FragColor = color;
}
