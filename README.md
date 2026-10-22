<p align="center">
  <img src="assets/logo.jpg" alt="Clovercut — a free, open-source video editor" width="800" />
</p>

<h1 align="center">clovercut</h1>
<p align="center"><strong>Your video. Your machine. Your editor.</strong></p>
<p align="center">A free Linux video editor with local processing.</p>

<p align="center">
  <a href="https://github.com/tulcoturo/clovercut"><img src="https://img.shields.io/badge/Project-Clovercut-16b86a" alt="Clovercut project" /></a>
  <a href="https://github.com/tulcoturo/clovercut/issues"><img src="https://img.shields.io/badge/Feedback-Issues-00bcd4" alt="Report bugs and suggest improvements" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-AGPL--3.0--or--later-16b86a" alt="License: AGPL-3.0-or-later" /></a>
</p>

## What is Clovercut?

**Clovercut** is an independent fork of [Concat](https://github.com/jub0t/Concat), built to develop a smoother video editing experience with its own identity. Our initial focus is performance, usability, and improvements informed by editing real projects.

Editing runs locally on a native Rust engine with GPU-accelerated composition. Optional AI models need to be downloaded before they can be used offline.

<p align="center">
  <img src="assets/screenshots/clovercut-captions.png" alt="Clovercut editing a demonstration project with video and audio tracks, an SRT library and caption styling controls" width="1280" />
</p>
<p align="center"><em>Clovercut's captions workspace, with a demonstration project and shared styling controls.</em></p>

## Editing with Clovercut

- **Animated stickers:** connect your own GIPHY API key to search and preview animated stickers, then add or drag them onto the timeline. Chosen originals are copied into the project.
- **Online audio library:** search CC0 music and sound effects on OpenGameArt, preview or add original audio files, and keep downloaded sounds available offline. No account or API key required.
- **Video and audio editing:** arrange multiple tracks, split and trim clips, adjust speed, transform pictures and mix audio.
- **Drag-and-drop effects:** drag a transition onto a cut between adjacent visual clips, drop visual effects and filters directly onto a clip, or drop an audio effect onto its target clip. Enable **Create adjustment layer** to place a visual treatment above the footage instead.
- **Video and image animation:** choose independent Fade, Slide or Pop entrance/exit presets in **Video → Animation**, composed with transform and opacity keyframes. Audio fades remain separate.
- **LUTs on clips:** import `.cube` files or choose a saved LUT in **Video → Adjust → Color**, with intensity, bypass and removal controls. The same controls work on adjustment layers.
- **A clearer inspector:** aligned sliders, compact value fields with units and steppers, paired X/Y position keyframes and separate Color, Curves and Wheels tabs. Caption controls have more spacing and recognisable bold, italic and underline toggles; text and caption outlines use a 0–20 px adjustment range.
- **Titles and captions:** fonts, colours, outlines, shadows, style presets and mixed-script vertical writing. CJK characters remain upright while Latin text reads sideways.
- **Automatic captions:** transcribe locally with Whisper after downloading a model, write captions manually, or import and export SRT files.
- **A dedicated captions workspace:** edit the collection's sentences in Captions, adjust its appearance in Style and choose entry/exit animations in Effects. The inspector opens on **Style** by default.
- **Reusable subtitle files:** drag an SRT from the embedded library onto the timeline. Each copy has its own style scope; changes can affect the whole collection or just the selected caption.
- **Caption animation:** Fade, Slide and Pop entrance/exit presets with adjustable duration, composed with manual position, scale, opacity, fill and outline keyframes.
- **Effects and export:** filters, transitions, colour controls and export formats including H.264, HEVC and AV1.
- **Local tools and automation:** text-to-speech, background removal, a command-line interface, JSON-RPC and an MCP server inherited from Concat.

Clovercut also improves text editing performance by painting only captions visible
at the current frame, keeping long caption lists virtualised and limiting shadow
blur work to the occupied area.

## Optional content libraries

External libraries are off by default. Enable **OpenGameArt** or **GIPHY** in
**Settings → Content libraries**. The Sounds and Stickers pages include a
shortcut to these settings. Imported local audio and built-in shapes work
without an external library.

### OpenGameArt audio

Enable **OpenGameArt**, then open **Sounds → OpenGameArt** to browse separate
Music and Sound effects categories. Preview a sound, add it at the playhead or
drag it onto the timeline. Duration appears after the original file has been
downloaded for preview or use; undownloaded results show `—:—`. Downloaded audio
is cached for offline use and copied into the project when added.
See the [audio library guide](docs/AUDIO_LIBRARY.md) for details.

### GIPHY stickers

Open **Settings → Content libraries**, enable **GIPHY**, choose **Create API key**,
then paste your key into the password field and select **Save key**.
Return to **Stickers → GIPHY** to browse. An empty search shows trending
stickers. Use the play button to preview, the plus button to add at the playhead,
or drag a sticker onto a video track. The source button opens the original page.

The key stays in the local app configuration, outside project files and logs.
`CLOVERCUT_GIPHY_API_KEY` can also configure it through the environment.
Search results and previews are session-only. Selected original GIFs and source
attribution are copied into the project's `stickers/` folder for editing and export.

The integration uses the official GIPHY API and attribution mark. GIPHY's
[API requirements](https://developers.giphy.com/docs/api/) apply, including
production access and permission for storing media. This implementation does
not claim approval from GIPHY.

## Project status

Clovercut **0.0.2** is being prepared for **Linux and Windows**. Linux packages
are AppImages for x86_64 and ARM64; Windows x64 packages include an installer
and a portable ZIP. Features are still being tested with real editing projects.

The editor executable is `clovercut`. Internal Rust crate names and `concat.json`
retain their inherited names for project compatibility. Settings and caches live
under `io.github.tulcoturo.clovercut`, separately from Concat.

## Using captions

Open **Captions** in the library to generate subtitles, write them manually or
import an SRT. Imported files stay in the subtitle library: drag one onto the
timeline or choose **Add to timeline**. Generated captions also insert their
first collection automatically. SRT contents are embedded in the project, so
moving or deleting the original file does not break the library.

Select a timeline caption to open **Captions / Style / Effects** in the inspector.
**Style** opens first. Use **Apply to all captions** for changes to the selected
collection, or deselect it to edit only that caption. Sentence edits stay local.
Use **Save collection to library** to reuse an existing collection as an SRT asset;
it saves the words and timings with the first caption's style as its initial look.

## Downloads

Version **0.0.2** is being prepared; its packages are not published yet.
Windows x64 will have `Clovercut-0.0.2-windows-x86_64-setup.exe` and
`Clovercut-0.0.2-windows-x86_64-portable.zip`. Extract the portable ZIP completely
before opening `clovercut.exe`.

### Published Linux AppImage

Download **0.0.1** from the [Clovercut releases page](https://github.com/tulcoturo/clovercut/releases/tag/v0.0.1).

This README describes the current development branch. The published 0.0.1
AppImages predate the online audio and sticker libraries, direct effect dragging
and latest inspector refinements; build from source to try those changes.

```bash
chmod +x Clovercut-0.0.1-x86_64.AppImage
./Clovercut-0.0.1-x86_64.AppImage
```

Use the `aarch64` build on ARM64 Linux. On systems without FUSE, run with
`APPIMAGE_EXTRACT_AND_RUN=1`. Builds target Ubuntu 22.04 / glibc 2.35 or newer;
system graphics drivers, GTK 3, ALSA and font libraries are required.
AI models download separately. See [release preparation](docs/RELEASING.md).

In development builds, if a graphics driver corrupts the window, start with
`clovercut --software` (or pass `--software` to a newly built AppImage).
This saves software drawing as the default; turn **Draw on the GPU** back on
in Settings to restore hardware drawing. The Linux launcher also offers a **Safe graphics** action.

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

To build the editor with less debug information and incremental compilation disabled:

```bash
cargo build --locked -p concat -j 1 \
  --config 'profile.dev.package.concat.debug=0' \
  --config 'profile.dev.package.concat.codegen-units=256' \
  --config 'profile.dev.package.concat.incremental=false'
```

To open an already compiled executable from the repository root:

```bash
./src/target/debug/clovercut
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

For code contributions, see [`CONTRIBUTING.md`](CONTRIBUTING.md). Inherited technical documentation is available in [`src/README.md`](src/README.md) and [`docs/MCP.md`](docs/MCP.md); the engine retains its inherited crate and API names.

## Credits and license

Clovercut builds on the work of **Jareer and the Concat contributors**. It is an independent project, not an official Concat distribution.

The code remains licensed under **AGPL-3.0-or-later**. See [`LICENSE`](LICENSE) and the dependency notices in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
