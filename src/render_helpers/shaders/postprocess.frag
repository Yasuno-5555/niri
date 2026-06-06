uniform float noise;
uniform float saturation;
uniform vec4 bg_color;

uniform float edge_highlight;
uniform float specular;

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

    // Apply liquid specular and edge highlights
    if (liquid > 0.0) {
        vec3 coords_geo = input_to_geo * vec3(v_coords, 1.0);
        
        // 1. SPECULAR (鏡面反射)
        vec2 centered = coords_geo.xy - vec2(0.5);
        vec2 normal = normalize(centered) * (1.0 - length(centered));
        float light_source = max(0.0, dot(vec3(normal, 0.5), normalize(vec3(-0.3, -0.5, 1.0))));
        float specular_glow = pow(light_source, 16.0) * specular;
        color.rgb += vec3(specular_glow);

        // 2. EDGE HIGHLIGHT
        vec2 dist_to_edge = min(coords_geo.xy, vec2(1.0) - coords_geo.xy) * geo_size;
        float border_distance = min(dist_to_edge.x, dist_to_edge.y);
        if (border_distance < 1.5) {
            color.rgb = mix(color.rgb, vec3(1.0), edge_highlight);
        }
    }

    return color;
}
