# Desktop release failure investigation — 24 October 2026

Failed run: https://github.com/tulcoturo/clovercut/actions/runs/40084264670

## Confirmed failures

- CI: `every_built_in_picture_package_ships_a_card` found six missing legacy GL JPEGs. Generate and bundle all six with the real GPU compositor; retain the inventory test.
- CI: `every_package_probe_holds` exposed Doorway's inverted source-UV translation. Move the doors outward in screen space, retain the midpoint expectation, and add endpoint probes. All 381 probes pass locally.
- Both Macs: the bundle compiled, signed and ran `--version`, but `--smoke` reported missing `rubberband`. Select official `ffmpeg-full` and check rubberband/asendcmd/asetnsamples before compilation.
- Windows x64: with LTO disabled, MSVC could not resolve four unused whisper-rs SystemInfo CPU-capability FFI references. Restore the original fat LTO to eliminate unused wrappers while retaining runtime CPU modules. Requires native build validation.
- Windows ARM64: compilation and dependency checks passed, but even `--version` crashed with 0xC0000005. A native ARM runner reproduced that fault by loading the old FFmpeg avcodec DLL alone; its dependent avformat/avfilter/avdevice loads also crashed. ONNX and Sherpa loaded successfully. The updated 24 October checksum-pinned FFmpeg passes all native library probes.

Diagnostic runs:
- Old dependency fails: https://github.com/tulcoturo/clovercut/actions/runs/40101841106
- Updated dependency succeeds: https://github.com/tulcoturo/clovercut/actions/runs/40102058693

## Upstream

Fetched upstream/main again: 971ad1d9214e85755c05edc8777c05b4d4e255dc.
No commits were added since the previous review. The non-equivalent commit list
includes manually adapted changes and intentional divergence; it is not a list
of missing fixes. Global SoundLayer audio and Android-specific renderer changes
remain deferred; this release does not import them.

## Release identity

Use 0.1.5 for the corrected source; preserve the failed v0.1.4 tag. The release
continues to require all eight desktop artifacts, full CI and native startup
smoke tests. Ordinary pushes still do not start engine/package builds.

## Follow-up: Windows x64 still fails with LTO restored

Run 40102255808 confirms that fat LTO does not resolve the four symbols. The
previous LTO-only diagnosis/fix was insufficient. The actual boundary is the
legacy whisper-rs SystemInfo wrapper calling CPU functions not present in the
import libraries of ggml's dynamic backend build.

For GGML_BACKEND_DL builds only, the patched sys crate now resolves those four
queries via the selected CPU backend's public `ggml_backend_get_features`
procedure. Static builds retain their original FFI. It does not implicitly load
modules from the working directory or hard-code CPU capabilities. A standalone
Windows probe forces all four wrappers to link without LTO, explicitly loads
the packaged CPU variants and checks their reported instructions against the
runner's available CPU features before another full editor build is attempted.
Package jobs also retain dependency caches after a failure.
