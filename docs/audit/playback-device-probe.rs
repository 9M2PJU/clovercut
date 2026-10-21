// SPDX-License-Identifier: AGPL-3.0-or-later
// Probe for Task 4.1: Inspect host audio devices, CPAL host/stream configuration,
// and measure active audio stream callback execution.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    println!("=== CPAL Host and Device Probe ===");
    let host = cpal::default_host();
    println!("Default CPAL Host ID: {:?}", host.id());

    let default_device = match host.default_output_device() {
        Some(d) => d,
        None => {
            eprintln!("ERROR: No default output audio device found!");
            std::process::exit(1);
        }
    };

    let dev_name = default_device
        .name()
        .unwrap_or_else(|e| format!("<unknown: {e}>"));
    println!("Default Output Device Name: {}", dev_name);

    let default_config = match default_device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Could not get default output config: {e}");
            std::process::exit(1);
        }
    };

    println!("Default Output Config:");
    println!("  Channels: {}", default_config.channels());
    println!("  Sample Rate: {} Hz", default_config.sample_rate().0);
    println!("  Sample Format: {:?}", default_config.sample_format());
    println!("  Buffer Size: {:?}", default_config.buffer_size());

    // Enumerate all available output devices
    println!("\nAll Output Devices:");
    if let Ok(devices) = host.output_devices() {
        for (i, dev) in devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| "<unknown>".into());
            let is_default = if name == dev_name { " [DEFAULT]" } else { "" };
            println!("  [{i}] {name}{is_default}");
        }
    }

    // Now attempt to build and run an active audio stream to verify callback firing
    println!("\nTesting Active Stream on Default Device...");
    let callback_count = Arc::new(AtomicUsize::new(0));
    let sample_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = Arc::clone(&callback_count);
    let sample_count_clone = Arc::clone(&sample_count);

    let stream_config: cpal::StreamConfig = default_config.clone().into();
    let sample_format = default_config.sample_format();

    let err_fn = |err| eprintln!("Audio stream callback error: {err}");

    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            default_device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    callback_count_clone.fetch_add(1, Ordering::Relaxed);
                    sample_count_clone.fetch_add(data.len(), Ordering::Relaxed);
                    // Output silence
                    for s in data.iter_mut() {
                        *s = 0.0;
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::I16 => default_device.build_output_stream(
            &stream_config,
            move |data: &mut [i16], _info: &cpal::OutputCallbackInfo| {
                callback_count_clone.fetch_add(1, Ordering::Relaxed);
                sample_count_clone.fetch_add(data.len(), Ordering::Relaxed);
                for s in data.iter_mut() {
                    *s = 0;
                }
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => default_device.build_output_stream(
            &stream_config,
            move |data: &mut [u16], _info: &cpal::OutputCallbackInfo| {
                callback_count_clone.fetch_add(1, Ordering::Relaxed);
                sample_count_clone.fetch_add(data.len(), Ordering::Relaxed);
                for s in data.iter_mut() {
                    *s = u16::MAX / 2;
                }
            },
            err_fn,
            None,
        ),
        other => {
            eprintln!("Unsupported sample format: {other:?}");
            std::process::exit(1);
        }
    };

    let stream = match stream {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: Failed to build output stream: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = stream.play() {
        eprintln!("ERROR: Failed to start playback stream: {e}");
        std::process::exit(1);
    }

    let start = Instant::now();
    let run_duration = Duration::from_millis(250);
    std::thread::sleep(run_duration);
    let elapsed = start.elapsed();

    let callbacks = callback_count.load(Ordering::Relaxed);
    let samples = sample_count.load(Ordering::Relaxed);

    println!("Stream execution elapsed: {:.2?}", elapsed);
    println!("Callbacks received: {}", callbacks);
    println!(
        "Total sample frames: {}",
        samples / stream_config.channels as usize
    );
    if callbacks > 0 {
        let avg_buffer_frames = (samples / stream_config.channels as usize) / callbacks;
        println!(
            "Average buffer size per callback: {} frames",
            avg_buffer_frames
        );
        println!(
            "Stream callback rate: {:.1} callbacks/sec",
            callbacks as f64 / elapsed.as_secs_f64()
        );
        println!("STREAM_VERIFICATION: SUCCESS (callbacks active)");
    } else {
        eprintln!("STREAM_VERIFICATION: FAILURE (0 callbacks received)");
        std::process::exit(1);
    }
}
