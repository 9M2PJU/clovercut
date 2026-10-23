fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dist = abs(uv.x - 0.5) * 2.0;
    if (dist < p) {
        return to_at(uv);
    }
    return from_at(uv);
}
