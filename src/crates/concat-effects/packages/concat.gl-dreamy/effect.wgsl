fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let wave = sin(uv.y * 20.0 + p * 12.0) * 0.04 * sin(p * 3.14159265);
    let offset = vec2<f32>(wave, 0.0);
    let ca = vec2<f32>(wave * 1.5, 0.0);
    
    let from_r = from_at(clamp(uv + offset + ca, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let from_g = from_at(clamp(uv + offset, vec2<f32>(0.0), vec2<f32>(1.0))).g;
    let from_b = from_at(clamp(uv + offset - ca, vec2<f32>(0.0), vec2<f32>(1.0))).b;
    let from_col = vec4<f32>(from_r, from_g, from_b, 1.0);
    
    let to_r = to_at(clamp(uv - offset + ca, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let to_g = to_at(clamp(uv - offset, vec2<f32>(0.0), vec2<f32>(1.0))).g;
    let to_b = to_at(clamp(uv - offset - ca, vec2<f32>(0.0), vec2<f32>(1.0))).b;
    let to_col = vec4<f32>(to_r, to_g, to_b, 1.0);
    
    return mix(from_col, to_col, p);
}
