<p align="center">
  <img src="logo.jpg" alt="Clovercut — a free, open-source video editor" width="800" />
</p>

<h1 align="center">Clovercut</h1>
<p align="center"><strong>Your video. Your machine. Your editor.</strong></p>
<p align="center">A free video editor with local processing and room to grow.</p>

<p align="center">
  <a href="https://github.com/tulcoturo/clovercut"><img src="https://img.shields.io/badge/Project-Clovercut-16b86a" alt="Clovercut project" /></a>
  <a href="https://github.com/tulcoturo/clovercut/issues"><img src="https://img.shields.io/badge/Feedback-Issues-00bcd4" alt="Report bugs and suggest improvements" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-AGPL--3.0--or--later-16b86a" alt="License: AGPL-3.0-or-later" /></a>
</p>

## What is Clovercut?

**Clovercut** is an independent fork of [Concat](https://github.com/jub0t/Concat), built to develop a smoother video editing experience with its own identity. Our initial focus is performance, usability, and improvements informed by editing real projects.

Editing runs locally on a native Rust engine with GPU-accelerated composition. Optional AI models need to be downloaded before they can be used offline.

## What you can do

The codebase inherited from Concat includes:

- **Edit video and audio across multiple tracks:** split, trim, move, and arrange clips on the timeline.
- **Create titles:** choose fonts, colours, outlines, shadows, and background plates.
- **Animate and apply effects:** keyframes, filters, transitions, and image and audio adjustments.
- **Use local AI tools:** Whisper captions, text-to-speech, and background removal, with the relevant models installed.
- **Export video:** formats including H.264, HEVC, and AV1, with support for 4K projects.
- **Automate editing:** command-line tools, a JSON-RPC API, and an MCP server.

These capabilities come from the original project. Clovercut-specific validation is ongoing.

## Project status

Clovercut is in early development. The first builds and hands-on checks have been performed on **Linux**. The original codebase also includes support for Windows, macOS, and mobile platforms, which still need validation in this fork.

Our first improvements include:

- Corrected reported text box dimensions for fractional edges.
- Faster title shadow rendering by limiting the blur to the area it needs to process.
- An optimised text module in development builds.

Executables and several internal components are still named `concat`. Adopting the Clovercut identity throughout the application is ongoing.

## Build and run

### Dependencies

- Rust **1.93.1**, pinned in [`src/rust-toolchain.toml`](src/rust-toolchain.toml), with Rustfmt and Clippy.
- **FFmpeg 7 or newer** development libraries.
- CMake, a C++ compiler, and libclang.
- On Linux, development libraries for fontconfig, FreeType, ALSA, GTK 3, and xkbcommon, plus the required graphics drivers.
- ONNX Runtime and sherpa-onnx, configured for building the AI tools.

The engine guide in [`src/README.md`](src/README.md) and the workflow in [`.github/workflows/ci.yml`](.github/workflows/ci.yml) describe the native dependency setup. Versions and hashes for prebuilt dependencies used by CI are recorded in [`scripts/ci/deps.lock`](scripts/ci/deps.lock).

### Get the source

```bash
git clone https://github.com/tulcoturo/clovercut.git
cd clovercut
```

After setting up the dependencies and environment variables:

```bash
cd src
cargo run --locked -p concat -j 1
```

If you have a `.env.local` file with your machine's configuration, load it with `source .env.local` from the repository root before entering `src/`. This file is local and excluded from Git.

To build the editor with less debug information and fewer code generation units:

```bash
cargo build --locked -p concat -j 1 \
  --config 'profile.dev.package.concat.debug=0' \
  --config 'profile.dev.package.concat.codegen-units=16'
```

To open an already compiled executable from the repository root:

```bash
./src/target/debug/concat
```

### Checks

From `src/`:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked --no-fail-fast -j 1
```

The initial UI build can use substantial memory, even with a single build job. Allow time and memory for this step before running the full test suite.

## Get involved

Found a bug or have an idea? Open an [issue in Clovercut](https://github.com/tulcoturo/clovercut/issues). For editing problems, include your operating system, steps to reproduce, and, where possible, details about the media used.

For code contributions, see [`CONTRIBUTING.md`](CONTRIBUTING.md). Inherited technical documentation is available in [`src/README.md`](src/README.md) and [`docs/MCP.md`](docs/MCP.md); some of it still refers to the original Concat identity and services.

## Credits and license

Clovercut builds on the work of **Jareer and the Concat contributors**. It is an independent project, not an official Concat distribution.

The code remains licensed under **AGPL-3.0-or-later**. See [`LICENSE`](LICENSE) and the dependency notices in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
