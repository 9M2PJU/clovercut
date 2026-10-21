// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Only local offsets cross Slint's f32 boundary. Absolute edit times stay f64.
/// A rebasing interval in seconds. It is an integer so origin changes can be
/// sent losslessly as i32, while visible offsets stay small even in long cuts.
pub const ORIGIN_SPAN: f64 = 1024.0;

pub fn origin(scroll: f64) -> f64 {
    (scroll / ORIGIN_SPAN).floor() * ORIGIN_SPAN
}
pub fn local(absolute: f64, origin: f64) -> f32 {
    (absolute - origin) as f32
}
pub fn absolute(local: f32, origin: f64) -> f64 {
    origin + f64::from(local)
}
pub fn nearest_frame(seconds: f64, num: i64, den: i64) -> i64 {
    (seconds.max(0.0) * num as f64 / den.max(1) as f64).round() as i64
}
pub fn frame_time(frame: i64, num: i64, den: i64) -> f64 {
    frame.max(0) as f64 * den.max(1) as f64 / num.max(1) as f64
}

/// A bounded display slice; the document and gesture origins remain unchanged.
#[derive(Clone, Copy, Debug)]
pub struct ViewSlice {
    pub start: f64,
    pub duration: f64,
    pub offset: f64,
}
pub fn visible_slice(start: f64, duration: f64, span: Option<(f64, f64)>) -> ViewSlice {
    let (from, to) = span.unwrap_or((start, start + duration));
    let shown = start.max(from).min(start + duration);
    let end = (start + duration).min(to).max(shown);
    ViewSlice {
        start: shown,
        duration: end - shown,
        offset: shown - start,
    }
}
/// Duration and shade at each end of the visible part of a fade.
pub fn fade_slice(
    view: ViewSlice,
    duration: f64,
    fade_in: f64,
    fade_out: f64,
) -> [(f64, f32, f32); 2] {
    let fade_in = fade_in.min(duration);
    let fade_out = fade_out.min(duration);
    let from = view.offset;
    let to = from + view.duration;
    let head = if fade_in > 0.0 {
        let end = to.min(fade_in).max(from);
        (
            end - from,
            (1.0 - from / fade_in).clamp(0.0, 1.0) as f32,
            (1.0 - end / fade_in).clamp(0.0, 1.0) as f32,
        )
    } else {
        (0.0, 1.0, 0.0)
    };
    let tail = if fade_out > 0.0 {
        let begin = from.max(duration - fade_out).min(to);
        (
            to - begin,
            ((begin - duration + fade_out) / fade_out).clamp(0.0, 1.0) as f32,
            ((to - duration + fade_out) / fade_out).clamp(0.0, 1.0) as f32,
        )
    } else {
        (0.0, 0.0, 1.0)
    };
    [head, tail]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn long_clip_geometry_and_fades_are_windowed_without_changing_the_cut() {
        let duration = 262144.0 + 1.0 / 60.0;
        let view = visible_slice(0.0, duration, Some((262143.5, 262145.0)));
        assert_eq!(view.offset, 262143.5);
        assert!((view.duration - (0.5 + 1.0 / 60.0)).abs() < 1e-10);
        assert_eq!(nearest_frame(view.start + view.duration, 60, 1), 15_728_641);
        let view = ViewSlice {
            start: 3.0,
            duration: 2.0,
            offset: 3.0,
        };
        assert_eq!(
            fade_slice(view, 10.0, 10.0, 10.0),
            [(2.0, 0.7, 0.5), (2.0, 0.3, 0.5)]
        );
        assert_eq!(fade_slice(view, 10.0, 1.0, 1.0)[0].0, 0.0);
    }
    #[test]
    fn local_coordinates_preserve_all_six_frame_grids_at_long_positions() {
        for (num, den) in [
            (24, 1),
            (25, 1),
            (30, 1),
            (60, 1),
            (24000, 1001),
            (30000, 1001),
        ] {
            for scroll in [0.0, 60.0, 3600.0, 21600.0, 86400.0, 262144.0, 1_000_000.0] {
                let base = origin(scroll);
                for n in -100..=100 {
                    let frame = nearest_frame(scroll, num, den) + n;
                    if frame < 0 {
                        continue;
                    }
                    let exact = frame_time(frame, num, den);
                    assert_eq!(
                        nearest_frame(absolute(local(exact, base), base), num, den),
                        frame
                    );
                }
            }
        }
    }
    #[test]
    fn exact_frame_picks_and_half_ties_use_the_project_fraction() {
        for (num, den) in [
            (24, 1),
            (25, 1),
            (30, 1),
            (60, 1),
            (24000, 1001),
            (30000, 1001),
        ] {
            for n in [0, 50, 10_000, 15_728_641] {
                assert_eq!(nearest_frame(frame_time(n, num, den), num, den), n);
            }
            assert_eq!(
                nearest_frame(
                    frame_time(50, num, den) + 0.51 * den as f64 / num as f64,
                    num,
                    den
                ),
                51
            );
        }
        assert_eq!(nearest_frame(50.5 / 24.0, 24, 1), 51);
        let input = 2.1062707901000977;
        assert_eq!(nearest_frame(input, 24000, 1001), 50);
    }
}
