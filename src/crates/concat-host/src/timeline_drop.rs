// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Pointer geometry for moving clips across variable-height timeline tracks.

/// Narrow logical-pixel band around a track boundary that requests a new track.
pub const INSERT_MARGIN: f32 = 5.0;

/// Converts a drag's delta into a point in the track stack. The UI includes
/// the original grab offset in `delta`; callers without one start at the centre.
/// Using the track top as the origin makes a horizontal drag request insertion.
pub fn pointer_y(top: f32, height: f32, delta: f32) -> f32 {
    top + height / 2.0 + delta
}

/// A move inside its original track must not request insertion, even if the
/// initial grab was on the clip's top border. Crossing out is deliberate.
pub fn inside_source(top: f32, height: f32, pointer: f32) -> bool {
    pointer >= top && pointer < top + height
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizontal_and_small_vertical_drags_stay_inside_the_track() {
        for height in [40.0, 60.0, 96.0, 140.0] {
            for delta in [-8.0, 0.0, 8.0] {
                let y = pointer_y(100.0, height, delta);
                assert!(y > 100.0 + INSERT_MARGIN && y < 100.0 + height - INSERT_MARGIN);
            }
            // A deliberate move to the boundary still reaches the insertion band.
            assert_eq!(pointer_y(100.0, height, -height / 2.0), 100.0);
        }
    }
    #[test]
    fn only_leaving_the_original_track_allows_insertion() {
        for y in [100.0, 101.0, 104.0, 130.0, 159.9] {
            assert!(inside_source(100.0, 60.0, y));
        }
        assert!(!inside_source(100.0, 60.0, 99.9));
        assert!(!inside_source(100.0, 60.0, 160.0));
    }
    #[test]
    fn the_original_grab_position_is_preserved() {
        // Track height 60, clip begins 4 px down, pointer grabbed 12 px into it.
        let correction = 4.0 + 12.0 - 60.0 / 2.0;
        assert_eq!(pointer_y(100.0, 60.0, correction), 116.0);
        assert_eq!(pointer_y(100.0, 60.0, correction + 9.0), 125.0);
        assert!(pointer_y(100.0, 60.0, correction) > 100.0 + INSERT_MARGIN);
    }
}
