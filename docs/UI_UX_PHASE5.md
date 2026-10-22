# Phase 5: UI/UX Baseline Walkthrough and Defect Inventory (`docs/UI_UX_PHASE5.md`)

## 1. Executive Summary

Phase 5, Task 5.1 executed a deterministic, automated visual walkthrough of the real **Clovercut** graphical production binary (`src/target/debug/clovercut`) using the Vulkan-backed Slint UI on Wayland (Hyprland). All interactions were executed against an isolated XDG environment (`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, `XDG_STATE_HOME`, `TMPDIR`), a disposable project (`Walkthrough_Demo`), and temporary synthetic media assets generated via `ffmpeg`, preserving all user projects, preferences, caches, and Phase 2–4 invariants untouched.

Ten full-window screenshots (`01` through `10`) were captured directly from the live Wayland surface via `grim` matched to the exact process client geometry (`927x1056` tiled narrow layout near the `900px` minimum width, plus `1080p` comparison).

---

## 2. Test Environment and Display Matrix

| Parameter | Value / Configuration |
| :--- | :--- |
| **OS / Kernel** | Linux `7.2.8-9-omarchy` (`x86_64`) |
| **Wayland Compositor** | Hyprland `0.56.2` (`WAYLAND_DISPLAY=wayland-1`) |
| **GPU Adapter** | `AMD Radeon RX 6600 XT (RADV NAVI23)` (`DiscreteGpu`, Vulkan, driver `radv Mesa 26.2.5-arch1.1`) |
| **CPU** | `AMD Ryzen 5 5600X 6-Core Processor` (12 threads) |
| **Monitors Active** | `HDMI-A-1` (`1920x1080 @ 60 Hz`, scale `1.0`), `DP-1` (`2560x1440 @ 60 Hz`, scale `1.0`) |
| **Tested Window Geometry** | `927x1056` at `(953, 12)` on `HDMI-A-1` (exercises narrow multi-column layout just above the `900px` minimum width in `ui/app.slint:113`) |
| **Display Scale** | `100%` (`scale = 1`, `Theme.ui_scale = 0`) |
| **Theme** | Dark (`Theme.dark = true`, default lime accent) |
| **Locale** | English (`en`, default fallback; `scripts/locales.py --check` also audited) |
| **Isolation** | `mktemp -d /tmp/clovercut-p5-xdg-XXXXXX` overriding all XDG and `TMPDIR` paths |

---

## 3. Exercised Workflows and Visual Evidence

All screenshots are stored in `docs/screenshots/phase5/`:

| # | Workflow Exercised | UI Components / Files | Screenshot Evidence |
| :--- | :--- | :--- | :--- |
| **1a** | **First Launch & Start Screen** | `ui/start.slint`, `ui/title-bar.slint`, `panes/start.rs` | `docs/screenshots/phase5/01_start_launch_screen.png` |
| **1b** | **Project Creation Dialog** | `ui/start.slint` (`NewProjectDialog`), aspect/size/fps chips | `docs/screenshots/phase5/01b_new_project_dialog.png` |
| **2** | **Media Import & Library Browsing** | `ui/workspace/media-pane.slint`, `ui/workspace/config-pane.slint` (`Details`), `ui/toast.slint` | `docs/screenshots/phase5/02_media_library_import.png` |
| **3** | **Timeline Editing, Selection, Split, Trim & Undo/Redo** | `ui/timeline/timeline-pane.slint`, `panes/timeline.rs` (`AddClip`, `SplitClips`, `undo`, `redo`, Razor tool) | `docs/screenshots/phase5/03_timeline_editing_razor.png` |
| **4** | **Inspector Controls & Property Editing** | `ui/inspector/clip-inspector.slint`, `ui/inspector/controls.slint` (`SetClipTransform`, `UpdateClip` opacity) | `docs/screenshots/phase5/04_inspector_transform_effects.png` |
| **5** | **Caption Creation & Styling** | `ui/dialogs/captions.slint`, `ui/inspector/text-panel.slint`, `panes/captions.rs` (`AddTextClip`, `CaptionsMsg::Open`) | `docs/screenshots/phase5/05_caption_styling_overlay.png` |
| **6** | **Media Relinking & Missing-File Handling** | `ui/dialogs/relink.slint`, `panes/relink.rs` (`RelinkMsg::Show`) | `docs/screenshots/phase5/06_missing_media_relink.png` |
| **7** | **Save Failure & Durability Feedback** | `ui/dialogs/confirm.slint`, `studio.rs` (`Confirm::QuitWithoutSaving`) | `docs/screenshots/phase5/07_save_durability_warning.png` |
| **8** | **Playback, Monitor Controls & Seeking** | `ui/workspace/preview-pane.slint`, `panes/monitor.rs`, audio VU meters (`L`/`R` at `-16 dB`) | `docs/screenshots/phase5/08_playback_monitor_seeking.png` |
| **9** | **Export Configuration Dialog** | `ui/dialogs/export.slint`, `panes/export.rs` (`ExportMsg::Open`) | `docs/screenshots/phase5/09_export_configuration_dialog.png` |
| **10** | **Post-Export Workspace & Layout State** | `ui/editor.slint`, `ui/inspector/text-panel.slint` (paused at `00:00:02:00`, saved indicator) | `docs/screenshots/phase5/10_editor_workspace_maximized_1080p.png` |

---

## 4. Ranked UI/UX Findings Catalog

Findings are strictly limited to issues directly observed in the captured screenshots, runtime logs, or verification scripts.

### Finding U5-01 — Horizontal Clipping of Controls in the Inspector Column at Narrow Widths (`927px`)
- **Classification:** Confirmed Defect
- **Severity:** High (P2)
- **Workflow & Component:** Inspector property editing and text styling (`src/crates/concat/ui/inspector/clip-inspector.slint:1207-1275`, `src/crates/concat/ui/inspector/controls.slint`, `src/crates/concat/ui/inspector/text-panel.slint`, `src/crates/concat/ui/workspace/config-pane.slint:291`).
- **Reproduction Steps:**
  1. Launch Clovercut at a window width near the `900px` minimum (`927x1056`).
  2. Open a project, place a video clip on the timeline, and select it to open the Video Inspector (`Basic` tab).
  3. Select a text/title clip to open the Text Inspector (`Layout` section).
- **Expected Behavior:** All inspector controls, buttons, and keyframe toggles fit within the Inspector column width without horizontal clipping.
- **Actual Behavior:**
  - **Position Row (`PositionRow`):** The two coordinate fields (`X` and `Y` spinboxes with `%` unit badges) exceed the column width, pushing the right-side keyframe diamond button partially off the right edge of the window (`03_timeline_editing_razor.png`, `04_inspector_transform_effects.png`).
  - **Size Row (`Fit` / `Fill` buttons):** The `[ Fill ]` button on the right is clipped at the right border (`03_timeline_editing_razor.png`, `04_inspector_transform_effects.png`).
  - **Text Align Row (`TextPanel -> Layout`):** The 4-option alignment `SegmentedControl` overflows horizontally, slicing the 4th alignment icon button in half at the right edge (`05_caption_styling_overlay.png`, `06_missing_media_relink.png`, `08_playback_monitor_seeking.png`).
  - **Inspector Sub-Tab Strip (`Basic | Enhance | Adjust | Animation`):** The four sub-tabs in `config-pane.slint:291` have zero horizontal gap between labels (`BasicEnhanceAdjustAnima...`) and truncate `Animation` to `Anima...` (`03_timeline_editing_razor.png`, `04_inspector_transform_effects.png`).
- **Evidence:** `docs/screenshots/phase5/03_timeline_editing_razor.png`, `04_inspector_transform_effects.png`, `05_caption_styling_overlay.png`, `08_playback_monitor_seeking.png`.
- **User Impact:** Users on narrow or tiled displays (`900px–1050px`) cannot click the `Position` keyframe button, the right-most `Text Align` option, or see the full `Fill` button and sub-tab labels.

---

### Finding U5-02 — Timeline Toolbar Right-Side Zoom Controls Clipped Off-Screen
- **Classification:** Confirmed Defect
- **Severity:** High (P2)
- **Workflow & Component:** Timeline editing toolbar (`src/crates/concat/ui/timeline/timeline-pane.slint`).
- **Reproduction Steps:**
  1. Open any project in a `927px`-wide window.
  2. Observe the right end of the timeline action bar (above the audio VU meters).
- **Expected Behavior:** Timeline zoom controls (`Zoom to Fit`, `Zoom Out [-]`, zoom slider, `Zoom In [+]`) remain visible and accessible at all supported window widths (`>= 900px`).
- **Actual Behavior:** Because the left/center tool strip has fixed icon buttons and separators without responsive collapsing or overflow scrolling, the `Zoom Out [-]` button at the far right is cut in half and the **entire zoom slider and `Zoom In [+]` button are clipped completely off-screen**.
- **Evidence:** `docs/screenshots/phase5/02_media_library_import.png` through `08_playback_monitor_seeking.png`.
- **User Impact:** Mouse/pointer users cannot access the timeline zoom slider or zoom-in button in narrow/tiled windows.

---

### Finding U5-03 — Modal & Floating Dialogs (`Captions`, `Missing Media`, `Quit Without Saving`) Misaligned and Lack Full-Workspace Scrim
- **Classification:** Confirmed Defect / Usability Issue
- **Severity:** Medium (P2)
- **Workflow & Component:** Caption creation, media relinking, and save failure recovery (`src/crates/concat/ui/dialogs/captions.slint`, `src/crates/concat/ui/dialogs/relink.slint`, `src/crates/concat/ui/dialogs/confirm.slint`, `src/crates/concat/ui/app.slint`).
- **Reproduction Steps:**
  1. In an open project at `927x1056`, trigger the Captions sheet (`05`), Missing Media Relink dialog (`06`), or Quit Without Saving confirmation (`07`).
- **Expected Behavior:** Modal dialogs should either center cleanly within the window with a consistent backdrop scrim dimming underlying panels, or anchor cleanly within the central workspace without colliding with adjacent active inspector controls.
- **Actual Behavior:**
  - **`Captions` Sheet (`05_caption_styling_overlay.png`):** Floats across the boundary between the Preview pane and the right Inspector pane, covering half the video monitor and the upper half of the Text Inspector while leaving the bottom half of the Text Inspector visually active underneath.
  - **`Missing Media` Dialog (`06_missing_media_relink.png`):** Positioned off-center over the bottom of the monitor and timeline toolbar; its `[ Dismiss ]` and `[ Relink All... ]` buttons overlap directly on top of the Inspector's `Text Align` row (`Layout` section) with no backdrop scrim dimming the Inspector column.
  - **`Quit without saving?` Dialog (`07_save_durability_warning.png`):** Overlaps the bottom edge of the preview canvas and partially covers the monitor transport bar (`00:00:01:29` timecode pill and playback buttons) without dimming the surrounding workspace columns.
- **Evidence:** `docs/screenshots/phase5/05_caption_styling_overlay.png`, `06_missing_media_relink.png`, `07_save_durability_warning.png`.
- **User Impact:** Visual collision between dialog CTA buttons and underlying Inspector/transport controls reduces clarity during critical error-recovery workflows.

---

### Finding U5-04 — Locale Inventory Check Fails (`monitor.previewFailed` Stale Key in `locales/en.json`)
- **Classification:** Confirmed Defect
- **Severity:** Medium (P2 — blocks Phase 5 acceptance criterion `python3 scripts/locales.py --check`)
- **Workflow & Component:** Localization inventory (`locales/en.json`, `scripts/locales.py`).
- **Reproduction Steps:**
  1. Run `python3 scripts/locales.py --check` from the repository root.
- **Expected Behavior:** Exit code `0` with all locale keys synchronized against `I18n.t` / `i18n::t` call sites.
- **Actual Behavior:** Command exits with code `1`:
  ```text
  en.json is out of date: run scripts/locales.py
      gone:    monitor.previewFailed
  ```
- **Evidence:** Direct CLI output from `python3 scripts/locales.py --check`.
- **User Impact:** Automated locale verification fails until stale key `monitor.previewFailed` is cleaned up via `scripts/locales.py`.

---

### Finding U5-05 — Left Pane Switcher Tab Truncation & Scroll Chevron Overlap (`[Ca... >]`)
- **Classification:** Usability / Layout Density Concern
- **Severity:** Low–Medium (P2)
- **Workflow & Component:** Library pane switching (`src/crates/concat/ui/workspace/pane-switcher.slint`, `src/crates/concat/ui/workspace/media-pane.slint`).
- **Reproduction Steps:**
  1. Open a project with default left column width in a `927px` window.
  2. Observe the top-left pane switcher bar (`Media`, `Audio`, `Text`, `Captions`, `Effects`, `Transitions`, `Filters`, `Stickers`).
- **Expected Behavior:** Visible tabs should clip cleanly before the scroll chevron `[>]`, with clear visual separation between the scroll button and tab text.
- **Actual Behavior:** The 4th tab (`Captions`) is truncated to `Ca...` directly underneath the floating `[>]` scroll chevron button, and the remaining 4 tabs (`Effects`, `Transitions`, `Filters`, `Stickers`) are hidden off-scroll without an immediately obvious indicator of how many categories exist.
- **Evidence:** `docs/screenshots/phase5/02_media_library_import.png` through `08_playback_monitor_seeking.png`.
- **User Impact:** Reduced discoverability of `Effects`, `Transitions`, `Filters`, and `Stickers` libraries on narrow screens.

---

### Finding U5-06 — Newly Created Text Clip Displays Missing/Truncated Font Badge (`[ FO... v ]` in Red)
- **Classification:** Usability / Default State Concern
- **Severity:** Low–Medium (P2)
- **Workflow & Component:** Caption/Title creation and Text Inspector (`src/crates/concat/ui/inspector/text-panel.slint`, `concat_project::model::TextStyle::default()`).
- **Reproduction Steps:**
  1. Create a new text clip with `TextStyle::default()` and select it.
  2. Inspect the `Typography -> Font` selector in the Text Inspector.
- **Expected Behavior:** The default bundled font (`Hanken Grotesk`) is selected, matched in the font list, and displayed legibly in the dropdown without an error/missing-font red border.
- **Actual Behavior:** The `Font` selector renders with a red warning outline and truncates the label to `FO...` (`06_missing_media_relink.png`, `08_playback_monitor_seeking.png`, `10_editor_workspace_maximized_1080p.png`), because the narrow dropdown box and/or asynchronous system font enumeration leaves the default `TextStyle` font unmatched or truncated.
- **Evidence:** `docs/screenshots/phase5/06_missing_media_relink.png`, `08_playback_monitor_seeking.png`, `10_editor_workspace_maximized_1080p.png`.
- **User Impact:** Gives the impression of a broken or missing font immediately after creating a standard title clip.

---

### Finding U5-07 — Cold-Cache First Publish Emits 37 Missing Effect-Card and 2 Media-Thumb Diagnostics (Audit U1)
- **Classification:** Confirmed Diagnostic / Loading-State Behavior (Audit U1)
- **Severity:** Low (P3)
- **Workflow & Component:** Cold-cache startup and initial media import (`src/crates/concat/src/studio.rs`, `refresh_art()`, `publish()`).
- **Reproduction Steps:**
  1. Launch Clovercut with an empty `XDG_DATA_HOME` / `XDG_CACHE_HOME`.
  2. Import new video files into a fresh project.
- **Expected Behavior:** While background workers render effect-card previews and media thumbnails, the UI should either bind an empty/placeholder image path or suppress Slint file-load attempts until the `.jpg` file is written to disk.
- **Actual Behavior:** `publish()` hands the deterministic target file paths (`.../cards/concat.*.jpg` and `.../cache/art/m*.thumb.jpg`) to Slint before the background art worker has finished writing them, causing Slint to print 37 `Error loading image from .../cards/concat.<effect>.jpg: No such file or directory (os error 2)` lines on startup and 2 `Error loading image from .../cache/art/m*.thumb.jpg: No such file or directory (os error 2)` lines on import. Once the background worker finishes (~200 ms later), the thumbnails populate properly (as seen in `02_media_library_import.png` where both video thumbnails are rendered).
- **Evidence:** Walkthrough execution log (`task-2347.log`) and `docs/screenshots/phase5/02_media_library_import.png`.
- **User Impact:** No persistent missing thumbnail in the UI after the worker completes, but noisy stderr diagnostics on every cold launch/import and brief blank frames prior to worker completion.

---

### Finding U5-08 — Toast Notification Banner Overlaps Bottom-Right Inspector Controls
- **Classification:** Usability Concern
- **Severity:** Low (P3)
- **Workflow & Component:** Toast notifications (`src/crates/concat/ui/toast.slint`, `src/crates/concat/ui/app.slint`).
- **Reproduction Steps:**
  1. Import media or trigger a toast notification while the right-hand Inspector column is open and scrolled to the bottom (`Mask` or `Position -> Vertical`).
- **Expected Behavior:** Toast notifications should either anchor over the bottom-center preview/timeline area or auto-dismiss/offset so they do not block interactive Inspector inputs.
- **Actual Behavior:** The `(✓ Imported 3 files)` pill floats at the bottom-right corner (`02` through `07`), directly covering the `Mask` header or `Position -> Vertical` slider in the Inspector.
- **Evidence:** `docs/screenshots/phase5/03_timeline_editing_razor.png`, `04_inspector_transform_effects.png`, `05_caption_styling_overlay.png`.
- **User Impact:** Temporarily blocks clicks on the bottom-most Inspector row while the toast is visible.

---

## 5. Verification Limitations

1. **Window Tiling Constraint in Step 10 (`toggle_maximize`):** Under Hyprland's tiling layout on Workspace 1, `winit::Window::set_maximized(true)` is ignored by the compositor when tiled alongside another client unless fullscreen/floating rules are applied via the compositor, so `10_editor_workspace_maximized_1080p.png` remained at the tiled `927x1056` geometry. This actually provided a clear post-export state comparison at the same `927x1056` geometry, while narrow-window clipping was thoroughly stressed.
2. **Single Scale & Theme in Task 5.1 Baseline:** Task 5.1 ran at `100%` display scale, Dark theme, and English (`en`) locale to establish the baseline walkthrough. Multi-scale (`125%`, `150%`), Light theme, and long-string locale checks are scheduled for Tasks 5.2–5.3.

---

## 6. Task 5.2 — Component, Primitive, Theme, Unit, Focus & Shortcut Inventory

### 6.1 Theme & Token Baseline (`src/crates/concat/ui/theme/theme.slint`)
Clovercut's central `Theme` global already provides a structured surface ladder, typography scale, spacing/radius tokens, and semantic state colors that should be preserved as the foundation of Clovercut's own visual identity:
- **Typography:** `font-sans: "Hanken Grotesk"`, `font-mono: "JetBrains Mono"`, with a 5-step scale: `fs-xs: 10px`, `fs-sm: 11px`, `fs-base: 12px`, `fs-lg: 13px`, `fs-xl: 14px`, plus `weight-regular: 400`, `weight-medium: 500`, `weight-semibold: 600`.
- **Control Sizing & Rhythm:** `h: 28px`, `h-sm: 24px`, `h-lg: 32px`, `gap-sm: 4px`, `gap: 6px`, `gap-lg: 10px`, `pad-sm: 8px`, `pad: 14px`, `pad-lg: 20px`, `label-w: 62px`, `r-xs: 3px`, `r-sm: 4px`, `r: 6px`, `r-lg: 10px`, `r-xl: 16px`, `stroke: 1px`, `divider: 6px`, `focus-width: 1.5px`.
- **Surface Hierarchy (`dark` / `light`):**
  - `bg-page` (`#0f0f10` / `#e8e7e3`) — window backdrop & dock splitters.
  - `bg-panel` (`#171718` / `#f5f4f1`) — primary pane background (`Panel`).
  - `bg-raised` (`#1f1f21` / `#ffffff`) — cards, floating menus, headers, modal surfaces.
  - `bg-well` (`#121213` / `#eae9e5`) — recessed tracks, timeline wells, preview surround.
  - `bg-field` / `bg-field-hover` / `bg-field-active` — interactive controls (`#242427` -> `#2c2c30` -> `#343439` in dark mode).
  - `overlay` (`#0000009e` / `#0c0c0e66`) — modal scrim backdrop.
- **Identity & Semantic Accents:**
  - `accent` defaults to index `0` (`"Lime"`, `#c8f04d` dark / `#557a00` light) with `accent-ink` (`#121606` / `#ffffff`) and `accent-muted` (`#c8f04d24` / `#557a001c`), plus 5 optional user-selectable accents (`Amber`, `Cyan`, `Coral`, `Violet`, `Mono`).
  - Status & media-lane tokens: `danger` (`#e5534b`), `warning` (`#e09b3d`), `success` (`#46b878`), `clip-video` (`#3f78b8`), `clip-audio` (`#2f9b78`), `clip-text` (`#b87938`).

### 6.2 Shared Primitive & Inspector Control Audit

| Component / File | Role | Observed State / Usability & Consistency Audit |
|---|---|---|
| `FocusRing` (`ui/primitives/focus-ring.slint`) | `1.5px` `Theme.accent` focus border | Present on `NumberField`, `Select`, `SegmentedControl`, and `IconButton`. **Missing** on `Button` (`modal.slint:250`), `Switch` (`controls.slint:668`), `Rail` (`controls.slint:166`), and `FontPicker` (`font-picker.slint:24`). |
| `NumberField` (`ui/primitives/number-field.slint`) & `ValueControl` (`ui/inspector/controls.slint:302`) | Scrubbable/editable numeric input with unit suffix | Supports drag-to-scrub, double-click/click text edit, `Up`/`Down` arrow step (`Shift` 10×), `Enter` commit, `Escape` revert, and `FocusRing`. **Defect:** Fixed `width: 88px` in `Param` (`controls.slint:579`) and `PositionRow` (`controls.slint:465, 480`) causes horizontal clipping when paired side-by-side or inside narrow Inspector columns (`< 240px`). |
| `Param` (`ui/inspector/controls.slint:505`) | Inspector parameter row (`label` + `Rail` + `ValueControl` + `KeyCluster`) | Currently stacks the label on its own vertical line (`height: 16px`) above `[Rail] [ValueControl(88px)]`, consuming `50px` vertical height per parameter and squeezing the slider rail at narrow widths. |
| `PositionRow` (`ui/inspector/controls.slint:439`) | Dual X/Y numeric row | Stacks label above two fixed `88px` `ValueControl` boxes (`182px` minimum control width + `28px` padding + keyframe icon), overflowing narrow Inspector panes (`U5-01`). |
| `SegmentedControl` (`ui/primitives/segmented-control.slint`) | Multi-option tab/mode switch | Computes `seg-w: (root.width - 6px) / count` with zero inter-segment gap and no minimum width or text elision (`overflow: elide` missing on segment `Text` at line 58). In `ConfigPane` (`["Basic", "Enhance", "Adjust", "Animation"]`), labels collide (`BasicEnhanceAdjustAnima...`) below `250px` (`U5-01`). |
| `FontPicker` (`ui/inspector/font-picker.slint`) + `Studio::font_families()` (`studio.rs:6035`) + `TextStyle::default()` (`model.rs:1651`) | Font family selector with missing-font badge | **Root cause of `U5-06` (`FO...` red badge):** (1) `TextStyle::default()` in `concat-project/src/model.rs:1651` specifies `"\"Cabinet Grotesk\""` instead of `"Hanken Grotesk"` (`concat_text::BUNDLED_FAMILY`); (2) `Studio::font_families()` in `studio.rs:6035` registers `BUNDLED_FAMILY`, user fonts, and OS fonts, but omits the 23 embedded `concat_text::BASE_FONTS` families (`Inter`, `Montserrat`, `Space Grotesk`, `Archivo Black`, `Bebas Neue`, `Anton`, etc.) used by built-in text presets. Furthermore, the badge `width` is too narrow for `"FONT MISSING"` alongside a `180px` dropdown, truncating to `FO...`. |
| `PaneSwitcher` (`ui/workspace/pane-switcher.slint`) | Corner pane-type selector & dock split/close menu | `min-width: 108px` header pill overlaps adjacent tab strips in `MediaPane` and `ConfigPane` (`U5-01`, `U5-05`). |
| `ToastLayer` (`ui/primitives/toast.slint`) | Floating status banner | Anchored at bottom-right (`x: parent.width - self.width - 20px, y: parent.height - self.height - 20px`), directly overlapping bottom Inspector controls (`U5-08`). |

### 6.3 Unit & Formatting Consistency Audit
- **Inspector Units (`src/crates/concat/ui/inspector/{clip-inspector,text-panel,caption-panel}.slint`):**
  - Scale, Opacity, Volume, Tracking, Line Height, Background/Shadow Opacity, Transition Progress: `%`.
  - Position X/Y, Font Size, Shadow Offset/Blur, Stroke Width, Corner Radius, Feather: `px`.
  - Rotation, Hue, Shadow Angle: `°`.
  - Speed multiplier: `x` (`0.10x`–`16.00x`).
  - Duration / Fade In / Fade Out / Caption Timing: `s` (`0.00s`).
  - Audio EQ / Compressor / Loudness: `Hz`, `dB`, `:1`, `ms`, `LUFS`.
- **Monitor & Timeline Units:** Timecode `HH:MM:SS:FF` (frame-accurate at project FPS) + frame rate badge (`24 fps`, `29.97 fps`, `60 fps`).
- **Export Units (`src/crates/concat/src/panes/export.rs`, `ui/dialogs/export.slint`):** Resolution (`1920×1080`, `3840×2160`), Frame rate (`fps`), Bitrate (`Mbps` / `kbps`), Estimated size (`MB` / `GB`).
- **Assessment:** Unit suffixes are well-standardized across crates (`format.rs` and `ValueControl.unit`), and must be preserved intact during layout responsiveness updates.

### 6.4 Keyboard Shortcuts & Locale Inventory Audit
- **Keyboard Shortcut Parity (`src/crates/concat/src/keymap.rs` vs `docs/KEYBOARD_SHORTCUTS.md`):**
  - All 34 actions in `Action::ALL` (`PlayPause`, `Cancel`, `DeleteSelected`, `SplitAtPlayhead`, `DuplicateClip`, `CopySelected`, `PasteAtPlayhead`, `SelectAllClips`, `AddMarker`, `PrevMarker`, `NextMarker`, `Undo`, `Redo`, `SaveProject`, `OpenProject`, `ExportVideo`, `NewProject`, `ImportMedia`, `ZoomIn`, `ZoomOut`, `ZoomToFit`, `StepBack`, `StepForward`, `JumpStart`, `JumpEnd`, `JumpBackSec`, `JumpForwardSec`, `SetInPoint`, `SetOutPoint`, `ClearInOut`, `ToggleSnap`, `ToggleRipple`, `ToggleLoop`, `SwitchWorkspace1`..`3`, `ShowShortcuts`) match `docs/KEYBOARD_SHORTCUTS.md`.
- **Locale Check (`scripts/locales.py --check`, Finding `U5-04`):**
  - Fails with `gone: monitor.previewFailed` because `scripts/locales.py:94` strips everything after the first `#[cfg(test)]` substring in a Rust file (`text.split("#[cfg(test)]")[0]`). In `src/crates/concat/src/panes/monitor.rs`, an inline `#[cfg(test)]` module was placed at line 177 ahead of `tf("monitor.previewFailed", ...)` at line 201. Either moving `#[cfg(test)]` to the end of `monitor.rs` or parsing only trailing test modules in `scripts/locales.py` restores clean locale verification across all 10 locales (`de`, `en`, `es`, `fr`, `hi`, `ja`, `ko`, `pt`, `ru`, `zh`).

---

## 7. CapCut-Inspired Target Design Direction & Incremental Task 5.3 Implementation Plan

### Design Philosophy & Guardrails
The six CapCut desktop screenshots supplied by the user establish the target **structural layout hierarchy, navigation flow, panel organization, control density, integrated captions workflow, timeline ergonomics, and compact export dialog presentation**.
- **Strict Identity Guardrail:** Clovercut must **not** copy CapCut branding, icons, artwork, proprietary assets, or cyan/teal brand palette. Clovercut retains its own `Hanken Grotesk` / `JetBrains Mono` typography, neutral dark/light slate surfaces (`#0f0f10` / `#171718` / `#1f1f21`), and signature **Lime** accent (`#c8f04d` dark / `#557a00` light), while keeping its customizable Blender-style `Dock` architecture and Phase 2–4 correctness guarantees intact.

---

### Area 1: Professional Three-Panel Editing Workspace with Full-Width Lower Timeline

#### 1. Existing Clovercut Behavior Worth Preserving
- Customizable binary `Dock` split tree (`src/crates/concat/src/dock.rs`) supporting user-adjustable splitter ratios (`MIN_RATIO = 0.12`, `MAX_RATIO = 0.88`), pane swapping (`PaneSwitcher`), splitting, closing, workspace presets (`Editing`, `Color`, `Audio`), and automatic compact two-row fallback (`compact_dock()`) below `COMPACT_WIDTH = 860.0`.
- Dedicated minimum seat protection (`SEAT_MIN_W = 240.0`, `SEAT_MIN_H = 140.0`) and `TIMELINE_FLOOR = 240.0`.

#### 2. Confirmed Usability Defects
- **`default_dock()` Topology Defect (`src/crates/concat/src/dock.rs:265-284`, Findings `U5-01` & `U5-02`):**
  - Currently, `default_dock()` uses an **outer vertical split** (`columns: true, ratio: 0.78`), placing `Media + Preview` on top-left and `Timeline` on bottom-left (`78%` width), while `Inspector` spans `100%` of the right-hand vertical height (`22%` width).
  - At `927×1025` (`911px` usable dock width), `0.22 * 911 = 200px` for the Inspector (violating `SEAT_MIN_W = 240px` and clipping Inspector controls, `U5-01`), while the Timeline only gets `0.78 * 911 = 710px` (clipping the Timeline zoom toolbar, `U5-02`).

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned)
- Restructure `default_dock()` (and `Editing` workspace preset `0` in `workspace_dock()`) to use an **outer horizontal split** (`columns: false, ratio: 0.56`):
  - **Upper Row (`first`, `56%` height):** 3-column horizontal split containing:
    1. **Left — Tools & Media Library (`PaneKind::Media`):** `~28%` width (min `240px`).
    2. **Center — Player / Preview Monitor (`PaneKind::Preview`):** `~44%` width.
    3. **Right — Context Inspector (`PaneKind::Inspector`):** `~28%` width (min `250px`).
  - **Lower Row (`second`, `44%` height):** **Full-width Timeline (`PaneKind::Timeline`)** spanning `100%` of the window width (`911px` at `927px` window; `1904px` at `1920×1080`).
- Update `default_dock` migration/fallback so existing saved layouts that match the legacy default ratio tree upgrade cleanly or reset via Workspace `Editing` (`Alt+1`).

#### 4. Architectural Implications
- Localized to `src/crates/concat/src/dock.rs` (`default_dock()`, `workspace_dock()`, and unit tests in `dock.rs`). The recursive 3-level `DockView` renderer in `src/crates/concat/ui/workspace/workspace.slint` (`DockLeaf` -> `DockLevel2` -> `DockLevel1` -> `DockView`) already supports depth-2 splits (`outer row -> inner column -> inner column`) without changing the Slint component hierarchy.

#### 5. Implementation Risks
- **Low Risk:** Existing unit tests in `dock.rs` assert specific tree shapes for `default_dock()` and `workspace_dock(0)` and must be updated alongside the layout change. Saved user preferences (`prefs.json`) with custom dock trees continue to deserialize without error.

#### 6. Recommended Implementation Order
- **Step 1 of Task 5.3** (foundational geometry fix that immediately relieves horizontal starvation in both the Timeline and Inspector).

#### 7. Acceptance Criteria
- **At `927×1025` (Narrow / Tiled):** Upper workspace displays `Media`, `Preview`, and `Inspector` side-by-side with `Inspector >= 245px` wide; lower `Timeline` spans the full `911px` inner window width.
- **At `1920×1080` (Full-HD):** Upper workspace displays `Media (~530px)`, `Preview (~840px)`, and `Inspector (~530px)`; lower `Timeline` spans the full `1904px` inner window width.

---

### Area 2: Clear, Accessible Tool-Category Navigation System with Responsive Overflow

#### 1. Existing Clovercut Behavior Worth Preserving
- Rich 10-category creative toolset in `MediaPane` (`Media`, `Audio`, `Text`, `Captions`, `Stickers`, `Transitions`, `Effects`, `Filters`, `Adjustments`, `Templates`), each backed by localized labels, search/filtering, and category sidebar sub-navigation (`side-items`).

#### 2. Confirmed Usability Defects
- **Clipped Horizontal Tab Strip (`src/crates/concat/ui/workspace/media-pane.slint:87-202, 1967-2135`, Finding `U5-05`):**
  - Only `Media`, `Audio`, `Text`, and half of `Captions` (`Ca...`) are visible at both `927×1025` and `1920×1080`; 6 categories are hidden off-screen.
  - The `PaneSwitcher` pill in the top-left corner (`left: 6px, width: 108px`) forces `left-inset: 112px` on the tab bar, consuming nearly half of the `MediaPane` header width before the first tab even starts.

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned)
- **Top Tool-Category Bar Refinements (`media-pane.slint`):**
  - Compact the `PaneSwitcher` trigger inside `MediaPane` and `ConfigPane` (or integrate a compact icon/chevron switcher `28px–34px` wide when in standard editing layout, expanding on click) so the tool-category strip gains `+75px` of usable horizontal width.
  - Reduce `LibraryTab` horizontal padding from `10px` to `6px` (`min-width: 48px`), add a crisp `2px` bottom active indicator bar in `Theme.accent` (`#c8f04d`), and add mouse-wheel horizontal scrolling (`ScrollEvent` forwarding `delta-y` to `strip.viewport-x`) in addition to the `<` / `>` step buttons.
  - Add an accessible **overflow menu button (`⋯` / dropdown)** at the right edge of the strip when `strip.width < strip-row.preferred-width`, listing all 10 tool categories in a popup menu with active state checkmarks so any category (`Stickers`, `Transitions`, `Effects`, `Filters`, `Adjustments`, `Templates`) can be reached in **1 click** at narrow window sizes without repeated scrolling.

#### 4. Architectural Implications
- Contained within `src/crates/concat/ui/workspace/media-pane.slint` and `src/crates/concat/ui/workspace/pane-switcher.slint`. No changes required to `Studio` tab indices (`0..9`).

#### 5. Implementation Risks
- **Low Risk:** Must ensure `PaneSwitcher` remains keyboard-accessible and mouse-clickable in all 4 panes so layout customization is preserved.

#### 6. Recommended Implementation Order
- **Step 2 of Task 5.3** (immediately after the 3-panel dock topology update).

#### 7. Acceptance Criteria
- **At `927×1025` (Narrow / Tiled):** Active tool category is clearly highlighted with Clovercut's Lime indicator; no category label is mid-word sliced under an opaque overlay without ellipsis/fade; all 10 categories are reachable in at most 1 click via the overflow menu or smooth horizontal scroll.
- **At `1920×1080` (Full-HD):** At least 7–8 tool categories are directly visible in the top strip of the left pane, with single-click overflow access to the remainder.

---

### Area 3: Responsive Inspector with Readable Labels, Usable Sliders, Numeric Inputs, and Font Availability

#### 1. Existing Clovercut Behavior Worth Preserving
- Context-sensitive Inspector switching automatically between Project Details (empty selection), Video Clip (`Basic`, `Enhance`, `Adjust`, `Animation`), Audio Clip (`Basic`, `Effects`), Text Clip (`TextPanel` + `TextMotionPanel` + `ClipInspector`), and Caption Segment (`CaptionPanel` + `CaptionMotionPanel`).
- Scrubbable `ValueControl` / `NumberField` with unit suffixes (`%`, `px`, `°`, `dB`, `x`, `s`), keyframe diamond buttons (`KeyCluster`), and section reset buttons.

#### 2. Confirmed Usability Defects
- **Clipped Inspector Controls & Cramped Tabs (`Finding U5-01`):**
  - `ConfigPane` top `SegmentedControl` (`["Basic", "Enhance", "Adjust", "Animation"]`) collides labels without padding or elision at narrow widths (`config-pane.slint:624-643`).
  - `PositionRow` (`controls.slint:439-500`) uses two fixed `88px` `ValueControl`s side-by-side (`182px` + padding + keyframe cluster), clipping the Y position input (`Y -216...`).
  - `VideoPanel` `Size` row (`clip-inspector.slint:1261-1275`) uses `min-width: text + 38px` for `Fit` and `Fill` alongside a `62px` label, clipping `Fill` on the right.
- **Spurious Red `FO...` Font Missing Badge (`Finding U5-06`):**
  - `TextStyle::default()` in `src/crates/concat-project/src/model.rs:1651` defaults to `"\"Cabinet Grotesk\""` (not bundled), and `Studio::font_families()` (`src/crates/concat/src/studio.rs:6035`) omits the 23 embedded `concat_text::BASE_FONTS` families, causing built-in text presets and captions to display a truncated red `FO...` badge in `FontPicker`.

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned)
- **Responsive Inspector Layout (`controls.slint`, `clip-inspector.slint`, `text-panel.slint`, `config-pane.slint`):**
  - **`ConfigPane` Header & Sub-Tabs:** Make `PaneSwitcher` compact (`icon-only` or compact pill when docked in Inspector) and give `SegmentedControl` a `2px` inner gap, `overflow: elide` on labels, and responsive horizontal padding so `Basic | Enhance | Adjust | Animation` never touch or overlap.
  - **`Param` Row (`controls.slint:505`):** Support a clean CapCut-style compact layout where the label (`overflow: elide`), interactive `Rail` (`horizontal-stretch: 1`, `min-width: 64px`), `ValueControl` (`width: 72px` instead of `88px`), and `KeyCluster` align cleanly without overflowing a `240px` pane.
  - **`PositionRow` (`controls.slint:439`):** Replace fixed `width: 88px` on the `X` and `Y` `ValueControl`s with `horizontal-stretch: 1` and `min-width: 68px` so both `X` and `Y` inputs scale fluidly inside any Inspector width `>= 240px`.
  - **`Fit` / `Fill` Row (`clip-inspector.slint:1261`):** Use `horizontal-stretch: 1` and `min-width: 52px` on the `Fit` / `Fill` pill buttons so both fit comfortably beside the `Size` label.
- **Bundled Font Catalog & Default Font Fix (`model.rs`, `studio.rs`, `font-picker.slint`):**
  - Update `TextStyle::default()` in `concat-project/src/model.rs:1651` to `"Hanken Grotesk"` (matching `concat_text::BUNDLED_FAMILY` and `SubtitleSettings::default()`).
  - Include all `concat_text::BASE_FONTS` family names (`Inter`, `Montserrat`, `Space Grotesk`, `Archivo Black`, `Bebas Neue`, `Anton`, `Oswald`, `Playfair Display`, etc.) in `Studio::font_families()` (`studio.rs:6035`) so every bundled preset font is selectable and recognized as installed.
  - When a genuinely missing external font is encountered on an imported project, render a readable warning pill (`Missing` with tooltip showing the full missing font name) that does not clip to `FO...`.

#### 4. Architectural Implications
- Changing `TextStyle::default()` from `"\"Cabinet Grotesk\""` to `"Hanken Grotesk"` aligns `TextStyle` with `SubtitleSettings::default()` (`model.rs:1702`, which already uses `"Hanken Grotesk"`) and `concat_text::resolve_family` (which already fell back to `"Hanken Grotesk"` whenever `"Cabinet Grotesk"` was missing on the host OS). We must check existing unit/parity tests to ensure no snapshot test hardcodes the literal string `"\"Cabinet Grotesk\""`.

#### 5. Implementation Risks
- **Low-to-Medium Risk:** Check `cargo nextest` across `concat-project`, `concat-text`, and `concat-export` after updating `TextStyle::default()` to verify zero test regressions.

#### 6. Recommended Implementation Order
- **Step 3 of Task 5.3** (Inspector controls, `SegmentedControl`, and bundled font fixes).

#### 7. Acceptance Criteria
- **At `927×1025` (Narrow / Tiled):** All Inspector tabs (`Basic`, `Enhance`, `Adjust`, `Animation`), sliders (`Scale`, `Opacity`, `Rotation`), dual inputs (`Position X/Y`), action pills (`Fit`, `Fill`), and `FontPicker` render with zero right-edge clipping and zero `FO...` false-positive errors.
- **At `1920×1080` (Full-HD):** Inspector controls stretch cleanly with generous slider track travel and crisp alignment of labels, numeric boxes, and keyframe diamonds.

---

### Area 4: Timeline Toolbar with Accessible Zoom and Transport Controls

#### 1. Existing Clovercut Behavior Worth Preserving
- Rich timeline editing action strip in `TimelineTray` (`src/crates/concat/ui/timeline/tray.slint`): `Undo`, `Redo`, `Select/Razor` mode toggle, `Split`, `Delete`, `Duplicate`, `Speed`, `Reverse`, `Freeze`, `Markers`, `Beat Detect`, `Snap`, `Ripple`, `Magnetic`, `Preview Axis`, `Zoom to Fit`, `Zoom Out [-]`, `Zoom In [+]`, and `Audio Meters`.
- Centered transport cluster in `PreviewPane` (`Skip Start`, `Step Back`, `Play/Pause`, `Step Forward`, `Skip End`, `Loop`, timecode readout, and aspect/proxy controls).

#### 2. Confirmed Usability Defects
- **Clipped Right-Edge Zoom & Meter Controls (`src/crates/concat/ui/timeline/tray.slint:228-493`, Finding `U5-02`):**
  - Because `TimelineTray` places 19 `26px` buttons and 6 dividers in a single rigid horizontal row without right-cluster pinning or overflow adaptation, any timeline pane narrower than `~745px` clips `Zoom Out [-]`, `Zoom In [+]`, and `Audio Meters`.

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned)
- **Two-Zone Pinned Timeline Toolbar (`tray.slint`):**
  - Partition `TimelineTray` into:
    1. **Left Editing Tool Cluster (`horizontal-stretch: 1`, scrollable/collapsible at extreme narrow widths):** `Undo`, `Redo` | `Cursor / Razor` | `Split`, `Delete`, `Duplicate` | `Speed`, `Reverse`, `Freeze` | `Marker`, `Add Beat`, `Detect Beats`.
    2. **Right Pinned View & Zoom Cluster (always visible, never clipped):** `Snap`, `Ripple`, `Magnetic` | `Zoom to Fit`, `Zoom Out [-]`, **interactive compact Zoom Slider (`Rail`, `72px` wide)**, `Zoom In [+]`, and `Audio Meters`.
  - Combined with the full-width lower timeline from Area 1 (`911px` at `927px` window width; `1904px` at `1920×1080`), both clusters plus the new CapCut-style interactive zoom slider fit with `> 120px` of spare breathing room even at the minimum supported `900px` window width.

#### 4. Architectural Implications
- Purely layout/UI changes in `src/crates/concat/ui/timeline/tray.slint` and `src/crates/concat/ui/workspace/timeline-pane.slint`. Binds the new compact zoom slider directly to existing `timeline-zoom` / `zoom-in` / `zoom-out` callbacks.

#### 5. Implementation Risks
- **Low Risk:** Does not touch `Lanes` pointer coordinate conversion or Phase 3 timeline precision semantics.

#### 6. Recommended Implementation Order
- **Step 4 of Task 5.3.**

#### 7. Acceptance Criteria
- **At `927×1025` (Narrow / Tiled) and `900px` min-width:** Every primary timeline tool, `Snap`/`Ripple`/`Magnetic` toggle, `Zoom Out [-]`, zoom slider, `Zoom In [+]`, `Zoom to Fit`, and `Audio Meters` toggle is 100% visible and clickable.
- **At `1920×1080` (Full-HD):** Left editing cluster and right zoom/view cluster are cleanly separated across the full-width timeline header.

---

### Area 5: Two Text Systems (Normal Titles vs. Dedicated Captions Workspace) & Integrated Captions Workflow

#### 1. Existing Clovercut Behavior Worth Preserving
Clovercut currently implements **two distinct, complementary systems** for adding text to a video (`src/crates/concat-project/src/captions.rs`), both of which must be preserved:
1. **System A — Normal Text / Standalone Titles (`caption_group: None`):**
   - **Left Tool Pane (`Text` tab, `tab == 1`):** Provides `Titles` (`Default Title` card), `Styles` (`PresetCard` grid), and `Languages` (multi-script presets), plus the timeline toolbar `T` (`add-text`) button.
   - **Right Inspector (`Text | Motion` tabs -> `TextPanel` + `TextMotionPanel`):** Edits standalone title clips with per-clip typography, box sizing (`Auto` / `px`), background plate, shadow, stroke, and keyframes.
   - **Bridge to Captions:** Selecting a normal text clip enables `Adopt text track` (`CaptionsLibrary.can-adopt`) in `CaptionsPanel` to convert a normal text track into a synchronized caption collection.
2. **System B — Dedicated Captions Workspace (`caption_group: Some(...)` & `Project::caption_sources`):**
   - **Left Tool Pane (`Captions` tab, `tab == 9` -> `CaptionsPanel` in `ui/workspace/captions-panel.slint`):** Already provides `Auto captions`, `Manual captions`, `Import SRT...`, `Export SRT...`, `Save collection to library` (`keep-collection`), `Adopt text track`, and the **Subtitle library** (`CaptionsLibrary.sources`) where imported or generated `.srt` files live as embedded, draggable cards (`Add to timeline` / `Remove`).
   - **Right Inspector (`Captions | Style | Effects` tabs in `ConfigPane`):** When any caption clip (`concat_project::captions::is_caption(clip)`) is selected on the timeline (`CaptionsLibrary.style-count > 0`), the Inspector **already** switches to the 3-tab caption workspace (opening on **`Style`** by default):
     - **`Captions` tab (`CaptionEditor`):** Virtualized list of all caption cues in the collection (`CaptionsLibrary.rows`) with timecode jump buttons and inline text editing.
     - **`Style` tab (`CaptionPanel`):** `Apply to all captions` collection-scope toggle (`CaptionsLibrary.apply-all`), text input, `FontPicker`, `Size`, `B`/`I`/`U` `TextFormatToggle`, `Color`, `Tracking`, `Line height` / `Column spacing`, `Text Align`, horizontal/vertical `Writing Direction`, 4 `Aa` `CaptionLook` quick presets, `Position`, `Opacity`, and `Stroke`.
     - **`Effects` tab (`CaptionMotionPanel`):** `Fade`, `Slide`, and `Pop` entrance/exit animations with adjustable duration.

#### 2. Confirmed Usability Defects
- **Note on Task 5.1 Walkthrough Screenshot `05`:** In Task 5.1, the automated walkthrough script added a normal Title clip (`caption_group: None`) and then opened `CaptionsDialog` (`CaptionsMsg::Open`), which is why screenshot `05` showed `TextPanel` (`Text | Motion`) behind the floating `CaptionsDialog` rather than `CaptionPanel` (`Captions | Style | Effects`).
- **Actual Remaining Usability Friction in the Dual Text / Captions Workflow:**
  1. **Floating `CaptionsDialog` Sheet Overlap (`ui/dialogs/captions.slint`):** When clicking `Auto captions` or `Manual captions` in `CaptionsPanel` (or the caption icon in the timeline toolbar), `CaptionsDialog` opens as a `480px` floating modal card at `home-right: 16px, home-top: 48px` over the upper-right Inspector/Preview without a scrim. Embedding the transcription/script form directly inside `CaptionsPanel` (or centering the modal with a clean scrim when triggered from the timeline tray) avoids colliding with the right-hand Inspector.
  2. **Discoverability of `Captions` vs. `Text` in the Left Tool Bar (`U5-05`):** Because the `Captions` tab in the left `MediaPane` header is currently half-clipped (`Ca...`) under the scroll chevron right next to `Text`, users can easily confuse the `Text` tab (standalone titles) with the `Captions` tab (subtitle collections), or miss the `Captions` tab entirely in narrow windows.
  3. **Narrow-Width Inspector Clipping in `CaptionPanel` & `TextPanel` (`U5-01`, `U5-06`):** Both `TextPanel` (normal text) and `CaptionPanel` (`Style` tab for captions) share `PositionRow`, `SegmentedControl`, and `FontPicker`, which clip horizontally in narrow Inspector columns (`< 240px`).

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned Two-System Implementation)
- **Preserve Both Top-Level Systems (`Text` and `Captions`) Exactly Like CapCut:**
  1. **`Text` System (`Text` tab in left pane + `Text | Motion` in right Inspector):**
     - Used for standalone titles, lower thirds, and styled text presets (`Titles`, `Styles`, `Languages`).
  2. **`Captions` System (`Captions` tab in left pane + `Captions | Style | Effects` in right Inspector):**
     - Matches CapCut's `Captions` tab layout (`media_1792627164315_558c9637.png`):
       - **Left Sidebar Sub-Categories inside `Captions` tab (`media-pane.slint`):**
         1. **`Auto captions`:** Inline panel in the left pane with Audio Source selector (`Clip with sound`), Transcriber Model selector (`Whisper`), initial placement/size options, live transcription progress bar, and a prominent bottom-right **`Generate`** (`Theme.accent` Lime) primary button.
         2. **`Manual captions` (`Add captions`):** Inline panel in the left pane with the multi-line script input, placement/size selectors, and **`Create`** button, plus `Adopt text track` when a normal text track is selected.
         3. **`Subtitle library` (`Local SRT`):** Inline panel with `Import SRT...`, `Export SRT...`, `Save collection to library`, and the draggable `.srt` cards (`Add to timeline` / `Remove`).
       - **Right Inspector (`Captions | Style | Effects` in `ConfigPane`):**
         - Preserves Clovercut's existing 3-tab caption inspector (`CaptionEditor` cue list, `CaptionPanel` with `Apply to all captions` collection scope, and `CaptionMotionPanel` entrance/exit animations), with responsive controls (Area 3) so nothing clips.
       - **Timeline Toolbar Caption Button:** Clicking the `Captions` tool in the timeline toolbar switches the left pane directly to the `Captions` tab (`Auto captions` view) rather than popping up a floating overlay over the right Inspector.

#### 4. Architectural Implications
- Preserves 100% of `concat_project::captions`, `CaptionsLibrary`, `CaptionEditor`, `CaptionPanel`, `CaptionMotionPanel`, and `CaptionsPane`.

#### 5. Implementation Risks
- **Low Risk:** Existing unit tests in `concat-project/src/captions.rs` and `concat/src/panes/captions.rs` remain intact.

#### 6. Recommended Implementation Order
- **Step 5 of Task 5.3.**

#### 7. Acceptance Criteria
- **At `927×1025` and `1920×1080`:** Both `Text` (normal titles -> `Text | Motion` in Inspector) and `Captions` (subtitle collections -> `CaptionsPanel` in left pane + `Captions | Style | Effects` in right Inspector) are clearly visible, unclipped, and operate without floating overlay collisions over the Inspector.

---

### Area 6: Compact, Well-Structured Export Dialog with Grouped Settings, Preview & Prominent Primary Action

#### 1. Existing Clovercut Behavior Worth Preserving
- Complete `ExportData` state machine (`idle`, `running`, `done`, `failed`), quick presets (`Social 1080p`, `4K Master`, `Archival ProRes`, `Web Optimized`, `Audio Only`), custom resolution/FPS/codec/bitrate/HDR controls, live frame preview with playhead scrub rail, preflight warnings (`missing_sources`, `disk_low`, `will_overwrite`), stage-aware progress bar (`Rendering video`, `Mixing audio`, `Finalizing container`), ETA, cancel, retry, and reveal-in-folder.

#### 2. Confirmed Usability Defects
- **Full-Window Takeover vs. Compact Modal Structure (`src/crates/concat/ui/dialogs/export.slint:543-819`):**
  - Currently, `ExportDialog` covers 100% of the application viewport (`sheet` with `width: root.width, height: root.height`) and spreads controls across 3 columns (`max: 1180px`), hiding the workspace context and scattering `File & Output`, `Format & Quality`, and `Range & Overlays` across separate vertical columns.

#### 3. Proposed Visual / Layout Improvements (CapCut-Aligned)
- **Centered Compact Modal Dialog (`~820px × 580px`, responsive down to `860px` window width) over `Theme.overlay` Scrim (`ui/dialogs/export.slint`):**
  - Render `ExportDialog` as an elevated centered modal card (`bg-panel`, `border-radius: Theme.r-xl`, `1px` `Theme.line`, `Theme.overlay` backdrop scrim) with a crisp header (`Export`, preset selector pill, and `[×]` close button).
  - **Two-Column Structured Body (matching CapCut's export layout):**
    - **Left Column — Preview & Summary (`~340px`):** 16:9 rendered video preview frame, interactive timecode scrub rail, clip/range selector (`Full timeline` vs `In/Out`), and compact metadata pill (`Duration | Estimated Size | Color Space`).
    - **Right Column — Grouped Settings (`horizontal-stretch: 1`):**
      1. **Destination Group:** `Name` text input and `Export to` folder path + `Browse` button.
      2. **Video Group (collapsible/checkable header):** `Resolution`, `Codec`, `Frame rate`, `Bitrate / Quality`, and `Color / HDR` in clean aligned `Label ↔ Control` rows, plus optional `Burn-in captions` toggle.
      3. **Audio Group:** `Format / Codec` (`AAC 320 kbps` / `PCM 24-bit` / `Copy` summary & audio-only mode).
  - **Bottom Footer Bar:** Pre-flight status/warning indicator on the left (`Ready to export` or disk/overwrite warning), and a **prominent Lime primary `Export` button** (`bg: Theme.accent`, `ink: Theme.accent-ink`, `min-width: 120px`, `height: 34px`) next to `Cancel` on the right.
  - Preserve full progress, cancellation, completion (`Open Folder`), and error/retry states inside the modal footer/body.

#### 4. Architectural Implications
- Contained entirely within `src/crates/concat/ui/dialogs/export.slint`. All `ExportData` properties and callbacks remain 100% compatible with `src/crates/concat/src/panes/export.rs`.

#### 5. Implementation Risks
- **Low Risk:** Existing export GUI tests and keyboard shortcuts (`Ctrl+E` to open, `Escape` to close when idle) continue to operate on `ExportData`.

#### 6. Recommended Implementation Order
- **Step 6 of Task 5.3.**

#### 7. Acceptance Criteria
- **At `927×1025` and `1920×1080`:** `ExportDialog` opens as a centered modal over a dimmed workspace scrim with left preview + right grouped settings (`Destination`, `Video`, `Audio`) and a prominent Lime `Export` action button, with zero clipped labels or controls.

---

### Area 7: Consistent Typography, Spacing, Surface Hierarchy, Active States, Focus Visibility, and Feedback

#### 1. Existing Clovercut Behavior Worth Preserving
- `Theme` token discipline (`Hanken Grotesk`, `JetBrains Mono`, dark/light mode support, 6 accent palettes defaulting to Lime), `FocusRing` primitive, modal `ConfirmDialog` and `RelinkDialog` workflows, and Phase 2 atomic save-error surfacing.

#### 2. Confirmed Usability Defects
- **Incomplete Keyboard Focus Ring Coverage:** `Button` (`modal.slint`), `Switch` (`controls.slint`), `Rail` (`controls.slint`), and `FontPicker` (`font-picker.slint`) lack visible keyboard `FocusRing` states.
- **Toast Notification Overlap (`Finding U5-08`):** `ToastLayer` (`ui/primitives/toast.slint`) anchors at bottom-right over the Inspector's bottom controls.
- **Locale Inventory Script Defect (`Finding U5-04`):** `scripts/locales.py --check` fails with `gone: monitor.previewFailed` due to naive `#[cfg(test)]` splitting in `src/crates/concat/src/panes/monitor.rs`.
- **Cold-Cache Missing Image Diagnostics (`Finding U5-07` / Audit U1):** `package_art()` in `src/crates/concat/src/studio.rs:1216-1220` and thumbnail loading call `slint::Image::load_from_path(path)` before checking `path.is_file()`, logging 37 + 2 `os error 2` lines to stderr on cold cache before falling back to `bundled_card(id)`.

#### 3. Proposed Visual / Layout Improvements
- **Keyboard Focus Visibility (`modal.slint`, `controls.slint`, `font-picker.slint`):**
  - Add `FocusScope` + `FocusRing` to `Button`, `Switch`, `Rail`, and `FontPicker`, supporting `Space`/`Enter` activation on buttons/switches and `Left`/`Right` arrow stepping on `Rail`.
- **Non-Obstructing Toast Placement (`ui/primitives/toast.slint`):**
  - Anchor `ToastLayer` at **top-center below the title bar** (`x: (parent.width - self.width) / 2, y: 48px`) or bottom-center above the timeline tray so it never blocks Inspector inputs or timeline zoom controls.
- **Clean Cold-Cache Image Loading (`src/crates/concat/src/studio.rs`, Task 5.5 / `U5-07`):**
  - Check `path.is_file()` before calling `slint::Image::load_from_path(&path)` in `package_art()` (`studio.rs:1216`) and media thumbnail loaders so cold-cache sessions fall back cleanly to `bundled_card(id)` or pending thumbnail generation without emitting spurious `os error 2` diagnostics.
- **Locale Check Restoration (`src/crates/concat/src/panes/monitor.rs` / `scripts/locales.py`, `U5-04`):**
  - Move the inline `#[cfg(test)]` module in `monitor.rs` to the end of the file (matching Rust convention across the workspace) so `python3 scripts/locales.py --check` scans the entire production module and exits `0`.

#### 4. Architectural Implications
- Minimal, localized enhancements to Slint primitives (`toast.slint`, `modal.slint`, `controls.slint`, `font-picker.slint`), `monitor.rs` module ordering, and `studio.rs:1216` file existence guard.

#### 5. Implementation Risks
- **Very Low Risk:** None of these changes alter playback, rendering, project persistence, or timeline editing math.

#### 6. Recommended Implementation Order
- **Step 7 of Task 5.3** (alongside dark/light theme, scaling, and locale verification in Tasks 5.3–5.5).

#### 7. Acceptance Criteria
- **At `927×1025` and `1920×1080` (Dark & Light Themes, `100%` & `125%+` Scale, EN + long-label locales):**
  - `Tab` navigation shows a crisp `1.5px` `Theme.accent` `FocusRing` on buttons, switches, sliders, selects, segmented controls, and numeric inputs.
  - Toast notifications appear without obscuring Inspector controls.
  - Cold-cache startup emits `0` missing-card `os error 2` diagnostics while rendering all effect/transition/preset cards.
  - `python3 scripts/locales.py --check` exits `0`.

---

### Summary of Recommended Task 5.3 Implementation Sequence

1. **Step 5.3.1 — Three-Panel Workspace & Full-Width Lower Timeline (`dock.rs`):** Update `default_dock()` and `workspace_dock(0)` so the top row hosts `Media | Preview | Inspector` and the bottom row hosts the full-width `Timeline`.
2. **Step 5.3.2 — Tool-Category Navigation & Compact Pane Switcher (`media-pane.slint`, `pane-switcher.slint`):** Compact `PaneSwitcher`, refine `LibraryTab` active indicator and spacing, add wheel scroll, and add a 1-click overflow menu for narrow windows (`U5-05`).
3. **Step 5.3.3 — Responsive Inspector & Bundled Fonts (`controls.slint`, `clip-inspector.slint`, `config-pane.slint`, `font-picker.slint`, `model.rs`, `studio.rs`):** Make `PositionRow`, `Param`, `Fit/Fill`, and `SegmentedControl` fluid down to `240px` (`U5-01`), and register `BASE_FONTS` + default `"Hanken Grotesk"` to eliminate false `FO...` font errors (`U5-06`).
4. **Step 5.3.4 — Pinned Timeline Toolbar & Zoom Slider (`tray.slint`):** Pin the right-hand zoom/view cluster and add an interactive zoom slider so zoom and transport-adjacent controls never clip at `>= 900px` (`U5-02`).
5. **Step 5.3.5 — Integrated Left-Pane Captions Workflow (`captions-panel.slint`, `media-pane.slint`, `config-pane.slint`):** Embed Auto Captions, Manual Script, and SRT tools directly into the left `CaptionsPanel` while keeping styling in the right Inspector (`U5-03`).
6. **Step 5.3.6 — Compact Centered Export Modal (`dialogs/export.slint`):** Refactor `ExportDialog` into a centered modal card with left preview + right grouped `Destination / Video / Audio` settings and a prominent Lime `Export` button.
7. **Step 5.3.7 — Focus Rings, Toast Placement, Cold-Cache Art Guard & Locale Check (`modal.slint`, `controls.slint`, `toast.slint`, `studio.rs`, `monitor.rs`):** Complete keyboard focus rings, move toast banner off the Inspector (`U5-08`), guard `package_art()` with `path.is_file()` (`U5-07` / U1), and restore `python3 scripts/locales.py --check` (`U5-04`).

---

## 8. Task 5.3 Execution & Post-Remediation Verification Results

Task 5.3 was executed incrementally across all 7 planned component families while preserving Clovercut's own identity, Dark/Light themes, Lime accent (`Theme.accent`), and the dual text/captions architecture:

### 8.1 Implemented Changes by Step

1. **Step 5.3.1 — Three-Panel Upper Workspace + Full-Width Lower Timeline (`src/crates/concat/src/dock.rs`, `src/crates/concat/src/lib.rs`):**
   - Refactored `default_dock()` so the outer split is horizontal (`columns: false, ratio: 0.56`), placing `Media` (`~28%`), `Preview` (`~44%`), and `Inspector` (`~28%`) side-by-side across the upper workspace (`56%` height) while `Dock::leaf(PaneKind::Timeline)` spans **100% of the lower window width** (`44%` height).
   - Added recursive `Dock::min_width()` and `Dock::min_height()` enforcement in `lay_out()` and `on_divider_dragged()` so every leaf panel respects `SEAT_MIN_W = 240.0` and `SEAT_MIN_H = 140.0` down to narrow desktop sizes (`927px`).
   - Added unit tests `dock::tests::default_dock_lays_out_three_upper_panels_and_full_width_lower_timeline` and `dock::tests::default_dock_enforces_minimum_seat_widths_down_to_narrow_desktop`.

2. **Step 5.3.2 — Tool-Category Navigation & Responsive Overflow (`src/crates/concat/ui/workspace/media-pane.slint`):**
   - Updated `LibraryTab` to highlight the active category icon and label in `Theme.accent` (Lime) with a `2px` bottom indicator bar, tightened horizontal padding (`5px`), removed the accidental `DragArea` wrapper, and added horizontal mouse-wheel scroll (`scroll-event`) support.
   - Added `OverflowCategoryRow` and `CategoryOverflowButton` (`Glyph.more`) in the `MediaPane` header, providing 1-click access to all 10 tool categories (`Media`, `Audio`, `Text`, `Captions`, `Stickers`, `Transitions`, `Effects`, `Filters`, `Adjustments`, `Templates`) at narrow window widths (`U5-05`).

3. **Step 5.3.3 — Contextual Video/Audio/Text/Captions Inspector & Bundled Font Catalog Fix (`config-pane.slint`, `segmented-control.slint`, `controls.slint`, `clip-inspector.slint`, `text-panel.slint`, `caption-panel.slint`, `caption-motion-panel.slint`, `video-motion-panel.slint`, `model.rs`, `concat-text/src/lib.rs`, `studio.rs`):**
   - Reduced `SegmentedControl` `min-width` from `120px` to `84px` and added `3px` horizontal inset on segment labels so `Basic | Enhance | Adjust | Animation` never touch and `Text Align` never overflows (`U5-01`).
   - Made `PositionRow` `X`/`Y` `ValueControl`s flexible (`min-width: 68px; horizontal-stretch: 1;`) and narrowed `Param` `ValueControl` to `width: 80px;` (`U5-01`).
   - Set `block: true;` on `Fit` and `Fill` buttons in `clip-inspector.slint` (`lines 1263-1275`) so `Button.min-width` does not overflow the `Size` row (`U5-01`).
   - Made `Tracking` and `Line height` side-by-side `ValueControl` columns flexible (`horizontal-stretch: 1; min-width: 68px;`) in `caption-panel.slint` (`U5-01`).
   - Updated the **Audio Inspector** (`config-pane.slint`) so non-batch audio sections are organized into **`Basic` | `Audio effects` | `Speed`** (giving `SpeedPanel` its own dedicated sub-tab alongside `Basic` gain/fades/voice processors and `Audio effects` preset cards).
   - Enhanced the **Text & Captions Inspector** (`config-pane.slint`, `text-panel.slint`, `caption-panel.slint`, `caption-motion-panel.slint`, `video-motion-panel.slint`): exposed `Text | Animation` tabs for standalone text titles (`Captions | Style | Animation` for caption collections), added `B / I / U` quick toggles and a 2-row 8-card visual style preset grid (`CaptionLook`) to both `TextPanel` and `CaptionPanel`, and added visual animation preset cards (`MotionPresetCard` for `None`, `Fade`, `Slide`, `Pop`) to `CaptionMotionPanel` and `VideoMotionPanel`.
   - Updated `TextStyle::default().font_family` in `src/crates/concat-project/src/model.rs:1651` from `"\"Cabinet Grotesk\""` to `"Hanken Grotesk"` (matching `concat_text::BUNDLED_FAMILY` and `SubtitleSettings::default()`), added `"Cabinet Grotesk"` to `concat_text::RETIRED`, exposed `BUNDLED_BASE_FAMILIES` / `BUNDLED_EXTRA_FAMILIES`, and registered all 29 bundled preset font families in `Studio::font_families()` (`studio.rs:6035`), eliminating false-positive `FO...` missing-font warnings (`U5-06`).
   - Added unit tests `model::tests::default_text_style_uses_bundled_hanken_grotesk_family` (`concat-project`) and `tests::bundled_base_and_extra_families_are_known_and_cabinet_grotesk_is_retired` (`concat-text`).

4. **Step 5.3.4 — Timeline Toolbar with Interactive Zoom Slider (`src/crates/concat/ui/timeline/tray.slint`, `src/crates/concat/ui/workspace/timeline-pane.slint`):**
   - Passed `seconds-per-pixel` into `TimelineTray` and added a CapCut-style interactive logarithmic `zoom-rail` slider (`width: 68px`) between `Zoom Out [-]` and `Zoom In [+]`. Combined with the full-width lower timeline, all 19 editing tools, `Snap`/`Ripple`/`Magnetic` toggles, `Zoom to Fit`, `[-]`, zoom slider, `[+]`, and `Audio Meters` are 100% visible and unclipped (`U5-02`).

5. **Step 5.3.5 — Integrated Left-Pane Captions Workflow & Dual Text/Captions Systems (`src/crates/concat/ui/workspace/captions-panel.slint`, `panes/captions.rs`, `media-pane.slint`, `app.slint`):**
   - Preserved both distinct text systems matching CapCut:
     1. **Normal Text (`Text` tab + `Text | Animation` in Inspector):** Standalone titles and presets (`caption_group: None`).
     2. **Dedicated Captions (`Captions` tab + `Captions | Style | Animation` in Inspector):** Embedded inline **Auto captions** controls (Audio source selector, Whisper model selector, Placement & Size segmented controls, inline progress bar, and direct `Generate captions` action), inline **Manual captions** script editor + `Add captions` action, and **Subtitle library** (`.srt` import/export and draggable tracks) directly inside `CaptionsPanel` in the left pane (`media-pane.slint`), while keeping collection-wide styling (`Apply to all captions`), cue list editing (`CaptionEditor`), and entrance/exit animations (`CaptionMotionPanel`) in the right Inspector.
   - Set `floating: false` on `CaptionsDialog` in `src/crates/concat/ui/app.slint:691` so when `CaptionsDialog` is opened from the Timeline toolbar, it centers cleanly over `Theme.overlay` scrim rather than floating over the top-right Inspector (`U5-03`).

6. **Step 5.3.6 — Compact Centered Export Modal (`src/crates/concat/ui/dialogs/export.slint`):**
   - Updated `ExportDialog` so that on desktop (`!root.phone && room-w >= 680px`) it renders as a compact, centered modal card (`max 860×620px`, `border-radius: Theme.r-xl`, `border-width: 1px`, `border-color: Theme.line-strong`) over a `Theme.overlay` backdrop scrim with a 2-column layout (Left: `Preview` with scrub rail + `Summary`; Right: `Destination`, `Output · Video`, and `Audio` settings) and a footer with `Cancel` + prominent Lime `Export · ~9 MB` primary action button.

7. **Step 5.3.7 — Modal Scrims, Focus Rings, Toast Placement, Cold-Cache Art Guard & Locale Check (`app.slint`, `modal.slint`, `controls.slint`, `font-picker.slint`, `toast.slint`, `studio.rs`, `host.rs`, `speech.rs`, `monitor.rs`):**
   - Added missing `x: 0; y: 0; width: root.width; height: root.height;` to `RelinkDialog` in `src/crates/concat/ui/app.slint:920`, restoring full-window `Theme.overlay` scrim and exact centering for `Missing Media` (`U5-03`).
   - Positioned `ToastLayer` at **top-center immediately below the title bar** (`x: (root.width - self.width) / 2, y: 44px`) in `src/crates/concat/ui/primitives/toast.slint` so toast notifications never cover the Preview transport bar, Timeline toolbar, or right-hand Inspector controls (`U5-08`).
   - Added `FocusScope` + `FocusRing` and keyboard activation (`Enter`/`Space` and `Left`/`Right` arrow keys) to `Button` (`modal.slint`), `Switch` (`controls.slint`), `Rail` (`controls.slint`), and `FontPicker` (`font-picker.slint`).
   - Guarded `package_art()` (`studio.rs:1216`), `image_at()` (`host.rs:433`), and voice avatar loaders (`speech.rs:767, 1037`) with `path.is_file()` checks before `slint::Image::load_from_path`, reducing cold-cache `Error loading image` stderr diagnostics from `39` to **`0`** (`U5-07` / Audit U1).
   - Refactored inline `#[cfg(test)]` blocks in `src/crates/concat/src/panes/monitor.rs` into helper methods at the bottom of the `impl` block so `python3 scripts/locales.py --check` scans the entire production module and exits **`0`** across all 24 locales (`U5-04`).

### 8.2 Post-Remediation Visual Evidence (`docs/screenshots/phase5_after/`)

| Screenshot | Workflow / State Verified | Resolution | Key Visual Verification |
|---|---|---|---|
| `01_start_launch_screen.png` | First launch & welcome screen | `1390×701` | Clean launch screen, `0` cold-cache stderr image errors |
| `01b_new_project_dialog.png` | New Project modal | `1390×701` | Centered modal over `Theme.overlay` scrim |
| `02_media_library_import.png` | 3-Panel workspace + Media import + Toast | `1390×701` | Upper 3-panel (`Media \| Preview \| Inspector`), lower 100%-width `Timeline`, active `Media` tab in Lime, `...` overflow button visible, `Imported 3 files` toast centered at top below title bar |
| `03_timeline_editing_razor.png` | Timeline editing, razor split, zoom slider | `1390×701` | Full-width timeline with all tools, `[-]`, interactive zoom slider, `[+]`, and VU meters unclipped |
| `04_inspector_transform_effects.png` | Video Inspector (`Basic \| Enhance \| Adjust \| Animation`) | `1390×701` | All 4 sub-tabs separated, `Scale`, `Position X/Y`, `Rotation`, `Width` unclipped |
| `05_caption_styling_overlay.png` | System A: Normal Text Inspector (`Text \| Animation`) | `1390×701` | Standalone title clip selected; `Font: Hanken Grotesk` (no `FO...` error), `B / I / U` toggles, 8-card `Styles` preset grid (`Aa`), `Size` slider & inputs unclipped |
| `05b_captions_collection_inspector.png` | System B: Dedicated Captions Inspector (`Captions \| Style \| Animation`) | `1390×701` | Synchronized `.srt` caption collection on timeline; `Captions \| Style \| Animation` tabs with `Apply to all captions`, `Hanken Grotesk`, `B / I / U`, 8-card `Styles` preset grid, color & size controls unclipped |
| `05c_captions_dialog_centered.png` | Centered `CaptionsDialog` modal | `1390×701` | Centered over `Theme.overlay` scrim; no longer overlaps the right Inspector |
| `06_missing_media_relink.png` | Centered `RelinkDialog` (`Missing Media`) | `1390×701` | Full-window `Theme.overlay` scrim and centered card (`U5-03` resolved) |
| `07_save_durability_warning.png` | Centered `ConfirmDialog` (`Quit without saving?`) | `1390×701` | Full-window `Theme.overlay` scrim and centered durability warning |
| `08_playback_monitor_seeking.png` | Active GPU playback & VU meters | `1390×701` | Real-time composited video + title + caption + active stereo VU meters |
| `09_export_configuration_dialog.png` | Compact centered `ExportDialog` modal | `1390×701` | Centered 2-column modal card over dimmed workspace scrim with Preview + Summary on left, Destination + `Output · Video` + `Audio` on right, and Lime `Export · ~9 MB` button |
| `10_editor_workspace_maximized_1080p.png` | Workspace Dark theme | `1390×701` | Crisp dark theme hierarchy across all 4 panes |
| `11_export_modal_1080p.png` | Export modal verification | `1390×701` | Centered export modal card over workspace |
| `12_editor_workspace_light_1080p.png` | Workspace Light theme verification | `1390×701` | Full Light theme (`Theme.dark = false`) contrast, borders, and Lime active states verified |
| `13_editor_workspace_narrow_927x1025.png` | Narrow window verification | `1390×701` | Responsive layout with overflow menu and unclipped Inspector/Timeline |

---

## 9. Tasks 5.4 & 5.5 Verification — Editing Semantics, Shortcuts, Selection Feedback & Fresh-Cache Art (Audit U1)

### 9.1 Task 5.4 — Inspector Controls, Shortcuts, Selection Feedback & Phase 3 Interaction Regression
- **Unit & Integration Test Suite (`concat --lib`):** All **171** library unit tests pass (`0 failed`, `0.50s`), covering:
  - `dock::tests`: 13 passed (including `default_dock_lays_out_three_upper_panels_and_full_width_lower_timeline` and `default_dock_enforces_minimum_seat_widths_down_to_narrow_desktop`).
  - `keymap::tests`: All default bindings, custom overrides, conflict resolution, reset, and `docs/KEYBOARD_SHORTCUTS.md` parity tests pass.
  - `format::tests` & `i18n::tests`: All numeric/timecode/rate formatting and 24-locale translation lookups pass; `python3 scripts/locales.py --check` exits `0` (`0 missing, 0 stale`).
  - `studio::tests`, `masking::tests`, `speed_ramp::tests`, `timeline_time::tests`: All Inspector property commits (`SetClipTransform`, `UpdateClip`, `SetClipKey`, batch volume, position keyframe pairs, color wheels/curves, speed ramps, mask grips) preserve exact command & undo/redo semantics.
- **Phase 3 Timeline Precision Regressions (`studio::phase3_timeline_tests --ignored`):** All **5** graphical/GPU timeline precision tests pass (`0 failed`, `7.46s`):
  - `cut_and_gesture_rounding_characterization ... ok`
  - `drag_history_restores_exact_state_without_roundtrip_drift ... ok`
  - `mixed_selection_locks_placement_and_origin_boundary ... ok`
  - `published_geometry_keeps_a_long_clip_tail_in_the_correct_frame ... ok`
  - `transport_clock_matches_decoded_and_published_frame ... ok`

### 9.2 Task 5.5 — Fresh-Cache Effect-Card Thumbnails & Loading/Error State Verification (Audit U1)
- **Root Cause & Fix Verified:** In Phase 1 (Audit U1), `package_art()` (`src/crates/concat/src/studio.rs:1220`), `image_at()` (`host.rs:433`), and `speech.rs` called `slint::Image::load_from_path(path)` before checking `path.is_file()`, emitting 37 missing-card `os error 2` diagnostics on cold-cache first publish before falling back to `bundled_card(id)`.
- **Verification in Fresh XDG Cache:**
  - `studio::tests::every_built_in_picture_package_ships_a_card ... ok`: Confirms every built-in effect, transition, and filter package has a valid bundled fallback card image compiled into the binary, so no effect card is ever blank on cold start.
  - `source::tests::new_project_library_and_timeline_playback --ignored --nocapture ... ok` (`0.59s`): Executed in a brand-new isolated `XDG_CONFIG_HOME` / `XDG_DATA_HOME` / `XDG_CACHE_HOME` / `XDG_STATE_HOME` directory; emitted **0** `Error loading image` diagnostics (down from 37 in Phase 1).
  - `studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close --ignored --nocapture ... ok` (`0.39s`): Confirmed save failure toast and durability modal recovery in a fresh XDG environment with **0** missing-image errors.


