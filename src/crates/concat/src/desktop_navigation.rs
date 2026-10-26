// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Small desktop navigation calculations, independent of the window.

/// The next/previous distinct edit in the real frame-rate timebase.
/// All lanes contribute; selection and clip order do not affect navigation.
pub fn edit_point(
    spans: impl IntoIterator<Item = (f64, f64)>,
    playhead: f64,
    fps: f64,
    forward: bool,
) -> f64 {
    if !fps.is_finite() || fps <= 0.0 || !playhead.is_finite() {
        return playhead;
    }
    let frame = |seconds: f64| (seconds * fps).round() as i64;
    let here = frame(playhead);
    let points = spans
        .into_iter()
        .flat_map(|(start, duration)| [start, start + duration])
        .chain([0.0])
        .filter(|s| s.is_finite() && *s >= 0.0)
        .map(frame);
    let found = if forward {
        points.filter(|p| *p > here).min()
    } else {
        points.filter(|p| *p < here).max()
    };
    found.map_or(playhead, |p| p as f64 / fps)
}

/// Top and height of the fixed main track, in the reversed visible stack.
pub fn main_track_geometry(heights: &[f32], main_index: usize) -> (f32, f32) {
    let Some(row) = main_index
        .checked_add(1)
        .and_then(|index| heights.len().checked_sub(index))
    else {
        return (0.0, 0.0);
    };
    (heights[..row].iter().sum(), heights[row])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walks_all_lanes_without_repeating_the_current_cut() {
        let spans = [(0.0, 2.0), (2.0, 3.0), (1.0, 2.0)];
        for (from, forward, expected) in [
            (0.0, true, 1.0),
            (1.0, true, 2.0),
            (2.001, true, 3.0),
            (2.001, false, 1.0),
            (5.0, true, 5.0),
            (0.0, false, 0.0),
        ] {
            assert_eq!(edit_point(spans, from, 30.0, forward), expected);
        }
    }
    #[test]
    fn fractional_frame_rates_keep_exact_frame_positions() {
        for fps in [30000.0 / 1001.0, 24000.0 / 1001.0] {
            let a = 301.0 / fps;
            let b = 607.0 / fps;
            assert!((edit_point([(a, b - a)], 0.0, fps, true) - a).abs() < 1e-10);
            assert!((edit_point([(a, b - a)], a, fps, true) - b).abs() < 1e-10);
        }
    }
    #[test]
    fn main_track_is_found_above_audio_and_below_overlay_tracks() {
        assert_eq!(
            main_track_geometry(&[28.0, 48.0, 64.0, 48.0, 48.0], 2),
            (76.0, 64.0)
        );
        assert_eq!(main_track_geometry(&[], 0), (0.0, 0.0));
    }
}
