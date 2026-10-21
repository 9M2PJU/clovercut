// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Characterizes exported cut frames for the current UI's half-frame inputs.
use concat_export::{ClipKind, ExportClip, ExportRequest};
use std::sync::atomic::AtomicBool;

#[test]
fn half_frame_cut_positions_reach_the_encoded_video() {
    let scratch = tempfile::tempdir().unwrap();
    let red = scratch.path().join("red.png");
    let blue = scratch.path().join("blue.png");
    for (path, colour) in [(&red, "red"), (&blue, "blue")] {
        let output = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-f", "lavfi", "-i"])
            .arg(format!("color=c={colour}:s=64x36"))
            .args(["-frames:v", "1", "-threads", "1"])
            .arg(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut differences = 0;
    for (num, den) in [
        (24_i64, 1_i64),
        (25, 1),
        (30, 1),
        (60, 1),
        (24000, 1001),
        (30000, 1001),
    ] {
        let fps = num as f64 / den as f64;
        let input = (50.5 / fps) as f32;
        let menu = f64::from(input);
        // Exact values of the existing Studio handlers, independently observed
        // by studio::phase3_timeline_tests, before selecting a new policy.
        let narrow_fps = fps;
        let razor = (f64::from(input) * narrow_fps).round() / narrow_fps;
        let mut cuts = Vec::new();
        for (label, cut) in [("menu", menu), ("razor", razor)] {
            let output = scratch.path().join(format!("{num}-{den}-{label}.mp4"));
            let mut head = ExportClip::blank(ClipKind::Image, 0.0, cut, 0);
            head.path = red.to_string_lossy().into_owned();
            let mut tail = ExportClip::blank(ClipKind::Image, cut, 96.0 / fps - cut, 0);
            tail.path = blue.to_string_lossy().into_owned();
            let mut request: ExportRequest = serde_json::from_value(serde_json::json!({
                "output":output.to_string_lossy(), "width":64,"height":36,
                "rateNum":num,"rateDen":den,"crf":18,"preset":"ultrafast","clips":[]
            }))
            .unwrap();
            request.clips = vec![head, tail];
            concat_host::export::run(&request, &AtomicBool::new(false), |_| {}).unwrap();
            let decoded = std::process::Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(&output)
                .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-threads", "1", "-"])
                .output()
                .unwrap();
            assert!(
                decoded.status.success(),
                "{}",
                String::from_utf8_lossy(&decoded.stderr)
            );
            let frames: Vec<_> = decoded.stdout.chunks_exact(64 * 36 * 3).collect();
            assert_eq!(frames.len(), 96);
            let blue_frame = frames
                .iter()
                .position(|frame| {
                    let p = &frame[(18 * 64 + 32) * 3..][..3];
                    p[2] > 180 && p[0] < 50
                })
                .unwrap();
            let expected = (cut * fps).round() as usize;
            assert_eq!(blue_frame, expected);
            for (index, frame) in frames.iter().enumerate() {
                let p = &frame[(18 * 64 + 32) * 3..][..3];
                if index < expected {
                    assert!(p[0] > 180 && p[2] < 50);
                } else {
                    assert!(p[2] > 180 && p[0] < 50);
                }
            }
            println!("export cut {num}/{den} {label}: first blue frame {blue_frame}");
            cuts.push(blue_frame);
        }
        if cuts[0] != cuts[1] {
            differences += 1;
        }
    }
    assert_eq!(
        differences, 0,
        "precise-rate razor must agree with the video grid"
    );
}
