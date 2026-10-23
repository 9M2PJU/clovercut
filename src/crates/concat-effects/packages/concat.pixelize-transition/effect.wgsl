fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = progress;
    let dist = 1.0 - abs(2.0 * p - 1.0);
    let blocks = max(2.0, 60.0 * (1.0 - dist));
    let coord = floor(uv * blocks) / blocks;
    return mix(from_at(coord), to_at(coord), p);
}
