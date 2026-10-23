fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dir = uv - vec2<f32>(0.5, 0.5);
    let dist = length(dir);
    if (dist > p) {
        return mix(from_at(uv), to_at(uv), p);
    } else {
        let offset = dir * (sin(dist * 32.0 - p * 30.0) * 0.035 * (1.0 - p));
        let clamped_uv = clamp(uv + offset, vec2<f32>(0.0), vec2<f32>(1.0));
        return mix(from_at(clamped_uv), to_at(clamped_uv), p);
    }
}
