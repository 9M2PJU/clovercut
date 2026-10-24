#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use bindings::*;

#[cfg(whisper_backend_dl)]
mod backend_features;
// Explicit exports override only these four legacy declarations from the glob.
#[cfg(whisper_backend_dl)]
pub use backend_features::{
    ggml_cpu_has_avx, ggml_cpu_has_avx2, ggml_cpu_has_f16c, ggml_cpu_has_fma,
};
