// SPDX-License-Identifier: Unlicense
// Compatibility with whisper-rs 0.16's legacy CPU feature queries. Dynamic
// CPU modules export those functions themselves, not through ggml's import
// library. Query the selected module through ggml's public registry instead.

use std::ffi::CStr;

unsafe fn cpu_feature(name: &CStr) -> i32 {
    let registry = unsafe { crate::ggml_backend_reg_by_name(c"CPU".as_ptr()) };
    if registry.is_null() {
        return 0;
    }
    let address = unsafe {
        crate::ggml_backend_reg_get_proc_address(registry, c"ggml_backend_get_features".as_ptr())
    };
    if address.is_null() {
        return 0;
    }
    // ggml-backend.h specifies this exact signature for the named procedure.
    let get_features: unsafe extern "C" fn(
        crate::ggml_backend_reg_t,
    ) -> *mut crate::ggml_backend_feature = unsafe { std::mem::transmute(address) };
    let mut feature = unsafe { get_features(registry) };
    if feature.is_null() {
        return 0;
    }
    // The module owns a static, NULL-name-terminated feature array. Modules
    // loaded by the editor remain registered for the process lifetime.
    while !unsafe { (*feature).name }.is_null() {
        let entry = unsafe { &*feature };
        if unsafe { CStr::from_ptr(entry.name) } == name {
            return i32::from(
                !entry.value.is_null() && unsafe { CStr::from_ptr(entry.value) }.to_bytes() == b"1",
            );
        }
        feature = unsafe { feature.add(1) };
    }
    0
}

/// Whether the currently registered CPU module provides AVX kernels.
/// No backend is loaded implicitly from the current working directory.
pub unsafe fn ggml_cpu_has_avx() -> i32 {
    unsafe { cpu_feature(c"AVX") }
}
/// Whether the currently registered CPU module provides AVX2 kernels.
pub unsafe fn ggml_cpu_has_avx2() -> i32 {
    unsafe { cpu_feature(c"AVX2") }
}
/// Whether the currently registered CPU module provides FMA kernels.
pub unsafe fn ggml_cpu_has_fma() -> i32 {
    unsafe { cpu_feature(c"FMA") }
}
/// Whether the currently registered CPU module provides F16C kernels.
pub unsafe fn ggml_cpu_has_f16c() -> i32 {
    unsafe { cpu_feature(c"F16C") }
}
