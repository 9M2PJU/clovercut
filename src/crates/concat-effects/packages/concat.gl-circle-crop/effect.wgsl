fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let aspect = frame.size.x / frame.size.y;
    let c = (uv - vec2<f32>(0.5)) * vec2<f32>(aspect, 1.0);
    let dist = length(c);
    let max_radius = length(vec2<f32>(aspect * 0.5, 0.5));
    let radius = p * max_radius;
    let edge = 0.015;
    let m = smoothstep(radius - edge, radius + edge, dist);
    return mix(to_at(uv), from_at(uv), m);
}
