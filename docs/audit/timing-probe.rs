#[allow(dead_code)]
#[path = "../../src/crates/concat-core/src/time.rs"]
mod time;
use time::{FrameRate, Rational};
fn main() {
    for (num, den) in [
        (24, 1),
        (25, 1),
        (30, 1),
        (60, 1),
        (24000, 1001),
        (30000, 1001),
    ] {
        let rate = FrameRate::checked(num, den).unwrap();
        let fps = rate.fps().as_f64();
        let mut example = None;
        for n in 1..10000 {
            let t = rate.time_of_frame(n);
            assert_eq!(rate.frame_at(t), n);
            let ui = t.as_f64() as f32;
            let tick = (f64::from(ui) * fps + 1e-6).floor() as i64;
            let rendered = (f64::from(ui) * fps).round().max(0.) as i64;
            if tick != rendered {
                example = Some((n, ui, tick, rendered));
                break;
            }
        }
        println!(
            "rate={num}/{den}: first transport-floor / preview-round mismatch below 10000 frames: {example:?}"
        );
    }
    let rate = FrameRate::from_int(60);
    let n = 15_728_641;
    let exact = rate.time_of_frame(n);
    let ui = exact.as_f64() as f32;
    let restored = (f64::from(ui) * 60.).round() as i64;
    println!(
        "long timeline: frame={n}, seconds={}, f32={ui}, quantized frame={restored}, frame difference={}",
        exact.as_f64(),
        restored - n
    );
    assert_ne!(restored, n);
    let rate = FrameRate::checked(30000, 1001).unwrap();
    let rational = Rational::approximate(1001.0 / 30000.0).unwrap();
    println!(
        "microsecond approximation at first NTSC frame: rational={rational}, frame_at={}",
        rate.frame_at(rational)
    );
    println!(
        "All exact rational frame roundtrips above passed. Mismatches demonstrate arithmetic boundaries, not a full GUI reproduction."
    );
}
