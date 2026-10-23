fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let middle = 0.5;
    let depth = 3.0;
    
    let scale = 1.0 / (1.0 + p * depth);
    let offset_x = p * 0.75;
    
    if (uv.x < middle) {
        let p_uv = (uv - vec2<f32>(0.25, 0.5)) / scale + vec2<f32>(0.25, 0.5) - vec2<f32>(offset_x, 0.0);
        if (p_uv.x >= 0.0 && p_uv.x <= 0.5 && p_uv.y >= 0.0 && p_uv.y <= 1.0) {
            let shade = mix(1.0, 0.35, p);
            let c = from_at(p_uv);
            return vec4<f32>(c.rgb * shade, c.a);
        }
    } else {
        let p_uv = (uv - vec2<f32>(0.75, 0.5)) / scale + vec2<f32>(0.75, 0.5) + vec2<f32>(offset_x, 0.0);
        if (p_uv.x >= 0.5 && p_uv.x <= 1.0 && p_uv.y >= 0.0 && p_uv.y <= 1.0) {
            let shade = mix(1.0, 0.35, p);
            let c = from_at(p_uv);
            return vec4<f32>(c.rgb * shade, c.a);
        }
    }
    return to_at(uv);
}
