fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let scale = 30.0;
    let d1 = fract((uv.x + uv.y) * scale);
    let d2 = fract((uv.x - uv.y) * scale);
    let hatch = min(d1, d2);
    let threshold = smoothstep(0.0, 1.0, p);
    if (hatch < threshold || p > 0.95) {
        return to_at(uv);
    } else {
        return from_at(uv);
    }
}
