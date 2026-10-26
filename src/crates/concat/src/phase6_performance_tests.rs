// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Phase 6: Performance and Memory Optimization Benchmarks & Lifecycle Verification.
//!
//! Covers Tasks 6.1, 6.2, 6.3, 6.4, and 6.5:
//! - 6.1: Reference machine, startup, seek latency, frame time, RSS & GPU memory.
//! - 6.2: Long audio, caption-heavy (240 cues), and layered 4K project profiling
//!   with separated decode, text layout/paint, UI publishing, and GPU composition costs.
//! - 6.3: Memory plateau across seek/play/project-switch cycles, cache eviction,
//!   old-project resource release, and 200 undo/redo steps.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use concat_host::projects;
use concat_project::Command;
use concat_project::commands::ClipPatch;
use concat_project::model::TextStyle;

use super::{Models, Studio};

/// Reads current process VmRSS and VmHWM in bytes from `/proc/self/status`.
fn read_process_memory_bytes() -> (u64, u64) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let mut rss_kb = 0u64;
    let mut hwm_kb = 0u64;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("VmHWM:") {
            hwm_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
        }
    }
    (rss_kb * 1024, hwm_kb * 1024)
}

/// Reads observable AMD/DRM GPU VRAM usage in bytes if exposed by sysfs.
fn read_drm_vram_bytes() -> Option<u64> {
    for card in ["card0", "card1", "card2"] {
        let path = format!("/sys/class/drm/{card}/device/mem_info_vram_used");
        if let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(bytes) = text.trim().parse::<u64>()
            && bytes > 0
        {
            return Some(bytes);
        }
    }
    None
}

fn generate_fixtures(dir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let hd_a = dir.join("hd_clip_a.mp4");
    let hd_b = dir.join("hd_clip_b.mp4");
    let uhd_4k = dir.join("uhd_4k_layer.mp4");
    let long_audio = dir.join("long_audio_180s.wav");

    if !hd_a.exists() {
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-v",
                    "error",
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc2=size=1920x1080:rate=30",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=440:sample_rate=48000",
                    "-t",
                    "4",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-g",
                    "15",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                ])
                .arg(&hd_a)
                .status()
                .unwrap()
                .success()
        );
    }

    if !hd_b.exists() {
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-v",
                    "error",
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc=size=1920x1080:rate=30",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=660:sample_rate=48000",
                    "-t",
                    "4",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-g",
                    "15",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                ])
                .arg(&hd_b)
                .status()
                .unwrap()
                .success()
        );
    }

    if !uhd_4k.exists() {
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-v",
                    "error",
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc2=size=3840x2160:rate=30",
                    "-t",
                    "2",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-g",
                    "15",
                    "-pix_fmt",
                    "yuv420p",
                    "-an",
                ])
                .arg(&uhd_4k)
                .status()
                .unwrap()
                .success()
        );
    }

    if !long_audio.exists() {
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-v",
                    "error",
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=330:sample_rate=48000",
                    "-t",
                    "180",
                    "-ac",
                    "2",
                    "-c:a",
                    "pcm_s16le",
                ])
                .arg(&long_audio)
                .status()
                .unwrap()
                .success()
        );
    }

    (hd_a, hd_b, uhd_4k, long_audio)
}

fn stats_summary(samples_ms: &[f64]) -> (f64, f64, f64) {
    if samples_ms.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut sorted = samples_ms.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let p95_idx = ((sorted.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    let p95 = sorted[p95_idx];
    let max = *sorted.last().unwrap();
    (mean, p95, max)
}

#[test]
#[ignore = "requires an isolated graphical session, FFmpeg, GPU and audio device"]
fn phase6_performance_and_memory_suite() {
    const CHILD: &str = "CLOVERCUT_PHASE6_PERF_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let root = std::env::temp_dir().join(format!("clovercut-phase6-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "studio::phase6_performance_tests::phase6_performance_and_memory_suite",
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD, "1")
            .env("TMPDIR", &root)
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("XDG_DATA_HOME", root.join("data"))
            .env("XDG_CACHE_HOME", root.join("cache"))
            .env("XDG_STATE_HOME", root.join("state"))
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        print!("{stdout}");
        eprint!("{stderr}");
        let _ = std::fs::remove_dir_all(&root);
        assert!(output.status.success(), "child failed:\n{stdout}\n{stderr}");
        return;
    }

    let root = PathBuf::from(std::env::var("TMPDIR").unwrap());
    let media_dir = root.join("media");
    std::fs::create_dir_all(&media_dir).unwrap();
    let (hd_a, hd_b, uhd_4k, long_audio) = generate_fixtures(&media_dir);

    println!("=== PHASE 6 PERFORMANCE & MEMORY SUITE ===");
    println!(
        "Total physical RAM: {:.2} GB | ReaderPool budget: {:.1} MB",
        concat_host::memory::total().unwrap_or(0) as f64 / (1024.0 * 1024.0 * 1024.0),
        concat_media::ReaderPool::frame_budget(concat_host::memory::total()) as f64
            / (1024.0 * 1024.0)
    );
    let (rss0, hwm0) = read_process_memory_bytes();
    let vram0 = read_drm_vram_bytes();
    println!(
        "Initial RSS: {:.1} MB | HWM: {:.1} MB | DRM VRAM: {:?}",
        rss0 as f64 / 1048576.0,
        hwm0 as f64 / 1048576.0,
        vram0.map(|v| format!("{:.1} MB", v as f64 / 1048576.0))
    );

    // -------------------------------------------------------------------------
    // TASK 6.1: Cold & Warm Startup, Seek Latency, Continuous Frame Time
    // -------------------------------------------------------------------------
    let t_cold_start = Instant::now();
    let gpu = crate::platform::select_backend(|_| {}, || {}, || {}, || {}).unwrap();
    let mut studio = Studio::new(crate::host::Host::start(gpu).unwrap());
    let app = crate::ui::App::new().unwrap();
    let models = Models::new();
    studio.workspace = (1920.0, 1080.0);
    studio.lanes.width = 1880.0;
    studio.publish(&app, &models);
    let cold_startup_ms = t_cold_start.elapsed().as_secs_f64() * 1000.0;

    let mut warm_open_runs = Vec::new();
    let proj_6_1 =
        projects::create(root.to_str().unwrap(), "Bench_6_1", 1920, 1080, 30, 1).unwrap();
    for run in 0..3 {
        let t0 = Instant::now();
        studio.open_project(proj_6_1.clone()).unwrap();
        studio.publish(&app, &models);
        warm_open_runs.push(t0.elapsed().as_secs_f64() * 1000.0);
        if run < 2 {
            studio.close_project().unwrap();
        }
    }
    println!(
        "[6.1 Startup] Cold init+publish: {:.2} ms | Warm project open (3 runs): {:.2} ms, {:.2} ms, {:.2} ms",
        cold_startup_ms, warm_open_runs[0], warm_open_runs[1], warm_open_runs[2]
    );

    // Populate Bench_6_1 with 2 HD clips
    let m_hd_a = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(hd_a.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let m_hd_b = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(hd_b.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let v_track = studio.timeline().tracks[0].id.clone();
    studio
        .apply(Command::AddClip {
            media_id: m_hd_a.clone(),
            track_id: v_track.clone(),
            start: 0.0,
            ripple: false,
        })
        .unwrap();
    studio
        .apply(Command::AddClip {
            media_id: m_hd_b.clone(),
            track_id: v_track.clone(),
            start: 4.0,
            ripple: false,
        })
        .unwrap();

    // Measure Cold Seek vs Warm Cached Seek across 3 runs (20 timestamps across 8s)
    for run in 0..3 {
        studio.host.monitor.clear();
        let mut cold_seeks = Vec::new();
        let mut warm_seeks = Vec::new();
        for idx in 0..20 {
            let t_sec = (idx as f64 * 0.38).min(7.5);
            let t0 = Instant::now();
            let (clips, settings) = studio.preview_clips(960, 540, t_sec).unwrap();
            let spec = concat_host::preview::FrameSpec {
                canvas: [0, 0, 0, 255],
                time: t_sec,
                width: 960,
                height: 540,
                moving: false,
                proxy: false,
                color_space: concat_project::model::ColorSpace::default(),
            };
            let sources = studio
                .host
                .monitor
                .frame_sources(clips, &settings, spec)
                .unwrap();
            let _tex = studio.host.monitor.texture_of(&sources, spec).unwrap();
            cold_seeks.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        for idx in 0..20 {
            let t_sec = (idx as f64 * 0.38).min(7.5);
            let t0 = Instant::now();
            let (clips, settings) = studio.preview_clips(960, 540, t_sec).unwrap();
            let spec = concat_host::preview::FrameSpec {
                canvas: [0, 0, 0, 255],
                time: t_sec,
                width: 960,
                height: 540,
                moving: false,
                proxy: false,
                color_space: concat_project::model::ColorSpace::default(),
            };
            let sources = studio
                .host
                .monitor
                .frame_sources(clips, &settings, spec)
                .unwrap();
            let _tex = studio.host.monitor.texture_of(&sources, spec).unwrap();
            warm_seeks.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        let (c_mean, c_p95, c_max) = stats_summary(&cold_seeks);
        let (w_mean, w_p95, w_max) = stats_summary(&warm_seeks);
        println!(
            "[6.1 Seek Run {}] Cold seek (20 pts): mean={:.2}ms p95={:.2}ms max={:.2}ms | Warm cached seek: mean={:.3}ms p95={:.3}ms max={:.3}ms",
            run + 1,
            c_mean,
            c_p95,
            c_max,
            w_mean,
            w_p95,
            w_max
        );
    }

    // Measure continuous 90-frame playback frame times (3 runs)
    for run in 0..3 {
        studio.host.monitor.clear();
        let mut frame_times = Vec::new();
        let mut over_33ms = 0usize;
        for f in 0..90 {
            let t_sec = f as f64 / 30.0;
            let t0 = Instant::now();
            let (clips, settings) = studio.preview_clips(960, 540, t_sec).unwrap();
            let spec = concat_host::preview::FrameSpec {
                canvas: [0, 0, 0, 255],
                time: t_sec,
                width: 960,
                height: 540,
                moving: true,
                proxy: false,
                color_space: concat_project::model::ColorSpace::default(),
            };
            let sources = studio
                .host
                .monitor
                .frame_sources(clips, &settings, spec)
                .unwrap();
            let _tex = studio.host.monitor.texture_of(&sources, spec).unwrap();
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            if ms > 33.333 {
                over_33ms += 1;
            }
            frame_times.push(ms);
        }
        let (mean, p95, max) = stats_summary(&frame_times);
        println!(
            "[6.1 Playback 90f Run {}] mean={:.2}ms p95={:.2}ms max={:.2}ms (>33.3ms frames: {}/90)",
            run + 1,
            mean,
            p95,
            max,
            over_33ms
        );
    }

    studio.close_project().unwrap();

    // -------------------------------------------------------------------------
    // TASK 6.2A: Long Audio Project (180s stereo 48kHz WAV + 12 spans)
    // -------------------------------------------------------------------------
    let proj_audio =
        projects::create(root.to_str().unwrap(), "Bench_6_2_Audio", 1920, 1080, 30, 1).unwrap();
    studio.open_project(proj_audio).unwrap();
    let m_audio = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(long_audio.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let a_track = studio.timeline().tracks[1].id.clone();
    for i in 0..12 {
        let cid = studio
            .apply(Command::AddClip {
                media_id: m_audio.clone(),
                track_id: a_track.clone(),
                start: i as f64 * 15.0,
                ripple: false,
            })
            .unwrap();
        let in_offset = i as f64 * 5.0;
        studio.apply(Command::TrimClip {
            clip_id: cid.clone(),
            edge: concat_project::commands::TrimEdge::End,
            delta: -(180.0 - (in_offset + 14.0)),
            ripple: false,
        });
        if in_offset > 0.0 {
            studio.apply(Command::TrimClip {
                clip_id: cid.clone(),
                edge: concat_project::commands::TrimEdge::Start,
                delta: in_offset,
                ripple: false,
            });
            studio.apply(Command::MoveClips {
                moves: vec![concat_project::commands::ClipMove {
                    clip_id: cid,
                    track_id: a_track.clone(),
                    start: i as f64 * 15.0,
                }],
            });
        }
    }

    for run in 0..3 {
        let t_peaks = Instant::now();
        let pyramid = concat_media::Pyramid::of(
            concat_media::peaks::extract(&long_audio, 200, None).unwrap(),
        );
        let peaks_ms = t_peaks.elapsed().as_secs_f64() * 1000.0;
        studio.peaks.insert(m_audio.clone(), Arc::new(pyramid));

        // Clear cached wave SVG strings to measure cold vs warm wave_path generation
        studio.waves.borrow_mut().clear();
        let t_pub_cold = Instant::now();
        studio.publish_lanes(&app, &models);
        let pub_cold_ms = t_pub_cold.elapsed().as_secs_f64() * 1000.0;

        let t_pub_warm = Instant::now();
        studio.publish_lanes(&app, &models);
        let pub_warm_ms = t_pub_warm.elapsed().as_secs_f64() * 1000.0;

        println!(
            "[6.2A Long Audio 180s Run {}] peaks_extract={:.2}ms | publish_lanes(cold_waves)={:.3}ms | publish_lanes(warm_waves)={:.3}ms | mapped_pcm={:.2}MB ({})",
            run + 1,
            peaks_ms,
            pub_cold_ms,
            pub_warm_ms,
            studio.host.playback.mapped_bytes() as f64 / 1048576.0,
            studio.host.playback.mapped_count()
        );
    }
    // Wait briefly so at least some audio decode workers finish mapping spans
    std::thread::sleep(Duration::from_millis(500));
    println!(
        "[6.2A Long Audio After Workers] mapped_pcm={:.2}MB ({} spans)",
        studio.host.playback.mapped_bytes() as f64 / 1048576.0,
        studio.host.playback.mapped_count()
    );
    studio.close_project().unwrap();

    // -------------------------------------------------------------------------
    // TASK 6.2B: Caption-Heavy Project (240 caption cues + video & audio clips)
    // -------------------------------------------------------------------------
    let proj_caps = projects::create(
        root.to_str().unwrap(),
        "Bench_6_2_Captions",
        1920,
        1080,
        30,
        1,
    )
    .unwrap();
    studio.open_project(proj_caps).unwrap();
    let m_vid = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(hd_a.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let v_track = studio.timeline().tracks[0].id.clone();
    for i in 0..30 {
        studio.apply(Command::AddClip {
            media_id: m_vid.clone(),
            track_id: v_track.clone(),
            start: i as f64 * 4.0,
            ripple: false,
        });
    }
    // Build 240 caption cues (0.5s each across 120s)
    let mut srt = String::new();
    for i in 0..240 {
        let s_ms = i * 500;
        let e_ms = s_ms + 450;
        let fmt_srt = |ms: usize| {
            let h = ms / 3_600_000;
            let m = (ms / 60_000) % 60;
            let s = (ms / 1_000) % 60;
            let rem = ms % 1_000;
            format!("{h:02}:{m:02}:{s:02},{rem:03}")
        };
        srt.push_str(&format!(
            "{}\n{} --> {}\nCaption cue number {} with shadow and stroke\n\n",
            i + 1,
            fmt_srt(s_ms),
            fmt_srt(e_ms),
            i + 1
        ));
    }
    studio
        .apply(Command::AddCaptionSource {
            name: "240_cues.srt".into(),
            srt,
            style: TextStyle {
                shadow: true,
                stroke_width: 0.004,
                background: "#00000080".into(),
                ..TextStyle::default()
            },
            offset_y: 0.35,
            place_at: Some(0.0),
        })
        .unwrap();

    println!(
        "[6.2B Caption-Heavy Setup] Total timeline clips: {}",
        studio.timeline().clips.len()
    );

    for run in 0..3 {
        // 1. Measure flatten_timeline_in directly
        let project_dir = PathBuf::from(studio.session.as_ref().unwrap().path());
        let t_flat = Instant::now();
        for _ in 0..60 {
            let _ = concat_export::flatten::flatten_timeline_in(
                studio.project(),
                None,
                Some(&project_dir),
            );
        }
        let flat_60_ms = t_flat.elapsed().as_secs_f64() * 1000.0;

        // 2. Measure preview_clips across 60 caption cue transitions (t = 0.1, 0.6, 1.1, ...)
        let t_preview_clips = Instant::now();
        for cue in 0..60 {
            let time = cue as f64 * 0.5 + 0.1;
            let _ = studio.preview_clips(960, 540, time).unwrap();
        }
        let preview_60_ms = t_preview_clips.elapsed().as_secs_f64() * 1000.0;

        // 3. Measure UI publish_lanes on 270-clip timeline
        let t_pub = Instant::now();
        for cue in 0..20 {
            studio.playhead = cue as f64 * 0.5 + 0.1;
            studio.publish_lanes(&app, &models);
        }
        let pub_20_ms = t_pub.elapsed().as_secs_f64() * 1000.0;

        println!(
            "[6.2B Caption-Heavy 240-cues Run {}] 60x flatten_timeline_in={:.2}ms ({:.3}ms/call) | 60x preview_clips(cue transitions)={:.2}ms ({:.3}ms/cue) | 20x publish_lanes={:.2}ms ({:.3}ms/pub) | titles_live={:.1}MB ({})",
            run + 1,
            flat_60_ms,
            flat_60_ms / 60.0,
            preview_60_ms,
            preview_60_ms / 60.0,
            pub_20_ms,
            pub_20_ms / 20.0,
            studio.host.titles.live_bytes() as f64 / 1048576.0,
            studio.host.titles.live_count()
        );
    }

    // -------------------------------------------------------------------------
    // TASK 6.3 (Part 1): 200 Undo / Redo Steps on the 270-clip Caption Project
    // -------------------------------------------------------------------------
    let first_clip_id = studio.timeline().clips[0].id.clone();
    for run in 0..3 {
        let t_apply = Instant::now();
        for step in 0..210 {
            let op = 0.2 + ((step % 80) as f64) * 0.01;
            studio.apply(Command::UpdateClip {
                clip_id: first_clip_id.clone(),
                patch: ClipPatch {
                    opacity: Some(op),
                    ..Default::default()
                },
            });
        }
        let apply_ms = t_apply.elapsed().as_secs_f64() * 1000.0;

        let t_undo = Instant::now();
        let mut undone = 0usize;
        while studio.session.as_ref().unwrap().can_undo() {
            studio.session.as_mut().unwrap().undo();
            undone += 1;
        }
        let undo_ms = t_undo.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(undone, 200, "Editor undo depth must cap at UNDO_DEPTH=200");

        let t_redo = Instant::now();
        let mut redone = 0usize;
        while studio.session.as_ref().unwrap().can_redo() {
            studio.session.as_mut().unwrap().redo();
            redone += 1;
        }
        let redo_ms = t_redo.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(redone, 200);

        let (rss_now, _) = read_process_memory_bytes();
        println!(
            "[6.3 200-Undo/Redo Run {}] 210 applies={:.2}ms | 200 undos={:.2}ms ({:.4}ms/step) | 200 redos={:.2}ms ({:.4}ms/step) | RSS={:.1}MB",
            run + 1,
            apply_ms,
            undo_ms,
            undo_ms / 200.0,
            redo_ms,
            redo_ms / 200.0,
            rss_now as f64 / 1048576.0
        );
    }

    studio.close_project().unwrap();

    // -------------------------------------------------------------------------
    // TASK 6.2C: Layered 4K Project (3840x2160 3-layer video + 4K shadow title)
    // -------------------------------------------------------------------------
    let proj_4k =
        projects::create(root.to_str().unwrap(), "Bench_6_2_4K", 3840, 2160, 30, 1).unwrap();
    studio.open_project(proj_4k).unwrap();
    let m_4k = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(uhd_4k.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let m_hd = studio
        .apply(Command::AddMedia {
            item: concat_host::media::probe(hd_b.to_str().unwrap())
                .unwrap()
                .to_new_media(),
        })
        .unwrap();
    let t0_id = studio.timeline().tracks[0].id.clone();
    studio
        .apply(Command::AddClip {
            media_id: m_4k,
            track_id: t0_id,
            start: 0.0,
            ripple: false,
        })
        .unwrap();
    let t_pip_id = studio
        .apply(Command::AddTrack {
            bottom: false,
            kind: None,
            insert_at: None,
        })
        .unwrap();
    let pip_cid = studio
        .apply(Command::AddClip {
            media_id: m_hd,
            track_id: t_pip_id,
            start: 0.0,
            ripple: false,
        })
        .unwrap();
    studio.apply(Command::SetClipTransform {
        clip_id: pip_cid,
        scale: Some(0.5),
        offset_x: Some(0.2),
        offset_y: Some(-0.2),
        rotation: Some(5.0),
        stretch_x: None,
        stretch_y: None,
    });
    studio
        .apply(Command::AddTextClip {
            track_id: None,
            above: true,
            start: 0.0,
            style: Some(TextStyle {
                content: "4K Layered Compositing & Shadow Title Benchmark".into(),
                font_size: 0.06,
                shadow: true,
                stroke_width: 0.004,
                background: "#101820cc".into(),
                ..TextStyle::default()
            }),
            duration: Some(2.0),
            offset_y: Some(0.32),
        })
        .unwrap();

    // Measure 4K concat_text::render_frame directly (3 runs of 5 renders)
    let fonts = concat_text::Fonts::new();
    for run in 0..3 {
        let mut paint_times = Vec::new();
        for i in 0..5 {
            let t_paint = Instant::now();
            let rendered = concat_text::render_frame(
                &fonts,
                &concat_text::TitleStyle {
                    content: format!("4K Shadow Title Frame Benchmark #{i}"),
                    font_family: "Hanken Grotesk".into(),
                    font_size: 0.06,
                    font_weight: 700.0,
                    italic: false,
                    underline: false,
                    strikethrough: false,
                    vertical_align: 1,
                    vertical: false,
                    color: "#ffffff".into(),
                    align: concat_text::Align::Center,
                    stroke_width: 0.004,
                    stroke_color: "#000000".into(),
                    shadow: true,
                    background: "#101820cc".into(),
                    background_radius: 0.015,
                    background_padding_x: 0.02,
                    background_padding_y: 0.01,
                    line_height: 1.2,
                    tracking: 0.0,
                    max_width: 0.8,
                    max_height: 0.0,
                },
                3840,
                2160,
            )
            .unwrap();
            assert_eq!(rendered.width, 3840);
            paint_times.push(t_paint.elapsed().as_secs_f64() * 1000.0);
        }
        let (mean, p95, max) = stats_summary(&paint_times);
        println!(
            "[6.2C 4K Title Paint (3840x2160 shadow+stroke+plate) Run {}] mean={:.2}ms p95={:.2}ms max={:.2}ms",
            run + 1,
            mean,
            p95,
            max
        );
    }

    // Measure 4K Layered Decode vs GPU Composite across 3 runs (10 frames at 3840x2160)
    for run in 0..3 {
        studio.host.monitor.clear();
        let mut decode_times = Vec::new();
        let mut gpu_times = Vec::new();
        for f in 0..10 {
            let t_sec = f as f64 / 30.0;
            let (clips, settings) = studio.preview_clips(3840, 2160, t_sec).unwrap();
            let spec = concat_host::preview::FrameSpec {
                canvas: [0, 0, 0, 255],
                time: t_sec,
                width: 3840,
                height: 2160,
                moving: false,
                proxy: false,
                color_space: concat_project::model::ColorSpace::default(),
            };
            let t_dec = Instant::now();
            let sources = studio
                .host
                .monitor
                .frame_sources(clips, &settings, spec)
                .unwrap();
            decode_times.push(t_dec.elapsed().as_secs_f64() * 1000.0);

            let t_gpu = Instant::now();
            let _tex = studio.host.monitor.texture_of(&sources, spec).unwrap();
            gpu_times.push(t_gpu.elapsed().as_secs_f64() * 1000.0);
        }
        let (d_mean, d_p95, d_max) = stats_summary(&decode_times);
        let (g_mean, g_p95, g_max) = stats_summary(&gpu_times);
        println!(
            "[6.2C 4K Layered 3840x2160 Run {}] decode(frame_sources): mean={:.2}ms p95={:.2}ms max={:.2}ms | gpu_composite(texture_of): mean={:.2}ms p95={:.2}ms max={:.2}ms | pool={:.1}MB gpu_pool={:.1}MB",
            run + 1,
            d_mean,
            d_p95,
            d_max,
            g_mean,
            g_p95,
            g_max,
            studio.host.monitor.pool_cached_bytes() as f64 / 1048576.0,
            studio.host.monitor.gpu_pool_bytes() as f64 / 1048576.0,
        );
    }

    // -------------------------------------------------------------------------
    // TASK 6.3 (Part 2): Project-Switch & Close Resource Release Verification
    // -------------------------------------------------------------------------
    let before_close_pool = studio.host.monitor.pool_cached_bytes();
    let before_close_gpu = studio.host.monitor.gpu_pool_bytes();
    let before_close_titles = studio.host.titles.live_bytes();
    let before_close_blocks = studio.title_blocks.len();
    let before_close_pcm = studio.host.playback.mapped_bytes();

    studio.close_project().unwrap();

    let after_close_pool = studio.host.monitor.pool_cached_bytes();
    let after_close_gpu = studio.host.monitor.gpu_pool_bytes();
    let after_close_titles = studio.host.titles.live_bytes();
    let after_close_blocks = studio.title_blocks.len();
    let after_close_pcm = studio.host.playback.mapped_bytes();
    let (rss_end, hwm_end) = read_process_memory_bytes();
    let vram_end = read_drm_vram_bytes();

    println!(
        "[6.3 Lifecycle Close] Before close -> After close:\n  CPU Frame Pool: {:.2} MB -> {:.2} MB\n  GPU Layer Pool: {:.2} MB -> {:.2} MB\n  Live Titles Cache: {:.2} MB -> {:.2} MB\n  Title Blocks Map: {} -> {}\n  PCM Mapped Audio: {:.2} MB -> {:.2} MB\n  Process RSS: {:.1} MB (HWM {:.1} MB) | DRM VRAM: {:?}",
        before_close_pool as f64 / 1048576.0,
        after_close_pool as f64 / 1048576.0,
        before_close_gpu as f64 / 1048576.0,
        after_close_gpu as f64 / 1048576.0,
        before_close_titles as f64 / 1048576.0,
        after_close_titles as f64 / 1048576.0,
        before_close_blocks,
        after_close_blocks,
        before_close_pcm as f64 / 1048576.0,
        after_close_pcm as f64 / 1048576.0,
        rss_end as f64 / 1048576.0,
        hwm_end as f64 / 1048576.0,
        vram_end.map(|v| format!("{:.1} MB", v as f64 / 1048576.0))
    );

    assert_eq!(after_close_pool, 0, "CPU frame pool must be empty on close");
    assert_eq!(after_close_gpu, 0, "GPU layer pool must be shrunk on close");
    assert_eq!(
        after_close_titles, 0,
        "Live titles cache must be cleared on close"
    );
    assert_eq!(
        after_close_blocks, 0,
        "Studio title_blocks must be cleared on close"
    );
    assert_eq!(
        after_close_pcm, 0,
        "PCM memory-mapped audio cache must be cleared on close"
    );
}
