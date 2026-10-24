//! A tiny native link/startup regression for whisper-rs with runtime CPU modules.
use whisper_rs::{SystemInfo, whisper_rs_sys as sys};

fn main() {
    // Force all four wrapper symbols into the final executable. The old shared
    // build fails to link this program, independent of Slint and the editor.
    let before = SystemInfo::default();
    println!("Before explicit backend load: {before:?}");
    let directory = std::env::var("WHISPER_MODULE_DIR").expect("WHISPER_MODULE_DIR");
    let directory = std::ffi::CString::new(directory).unwrap();
    unsafe {
        sys::ggml_backend_load_all_from_path(directory.as_ptr());
    }
    let registry = unsafe { sys::ggml_backend_reg_by_name(c"CPU".as_ptr()) };
    assert!(!registry.is_null(), "No CPU module loaded");
    let info = SystemInfo::default();
    println!("Selected CPU module: {info:?}");
    // The selected backend must not claim instructions unavailable to this CPU.
    assert!(!info.avx || std::is_x86_feature_detected!("avx"));
    assert!(!info.avx2 || std::is_x86_feature_detected!("avx2"));
    assert!(!info.fma || std::is_x86_feature_detected!("fma"));
    assert!(!info.f16c || std::is_x86_feature_detected!("f16c"));
    assert!(
        info.avx,
        "This runner supports AVX; its selected backend should too"
    );
}
