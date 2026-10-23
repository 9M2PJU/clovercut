fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dy = abs(uv.y - 0.5);
    let dx = abs(uv.x - 0.5);
    let threshold = p * 1.4;
    let inside = (dx <= (0.5 - dy) * threshold) || (p >= 0.99);
    if (inside) {
        return to_at(uv);
    } else {
        return from_at(uv);
    }
}
