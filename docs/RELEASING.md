# Clovercut AppImage releases

Clovercut publishes Linux AppImages only, for x86_64 and aarch64. The release
workflow builds on Ubuntu 22.04 runners and keeps native dependency downloads
pinned to hashes in `scripts/ci/deps.lock`.

Before creating a release:

1. Set `[workspace.package].version` in `src/Cargo.toml` and update the workspace entries in `src/Cargo.lock`.
2. Write the matching `docs/releases/<version>.md` and review the README.
3. Run formatting, Clippy, tests, locale and model inventory checks. Compile the editor and check `clovercut --version`, `clovercut --smoke`, editing and export.
4. Commit and push the reviewed changes. Create and push the matching tag only when ready (for this release, `v0.0.1`).

Before tagging, the manual **Build Clovercut AppImages** workflow can build a
specific commit from the updated main branch. Push all intended changes first,
then dispatch with that exact commit as `ref`; a successful build produces
artifacts without publishing a release. Inspect those artifacts before tagging.
Do not rerun an old version tag expecting it to include newer main-branch changes.

The tag starts `.github/workflows/release.yml`: CI, two Linux builds, staged-binary
checks, CPU compatibility checks on x86_64, AppImage packaging, and AppImage startup
checks in Ubuntu 22.04 and Debian 12 containers. No Windows, macOS, mobile, DEB,
RPM or Arch packages are built by this workflow.

CI uses matching dev/test debug and codegen settings across lint and test
steps. Cold test compilation has a 75-minute budget, separate from test execution;
this avoids the previous 25-minute compilation timeout blocking packaging.

The result is a **draft** GitHub release containing:

- `Clovercut-<version>-x86_64.AppImage`
- `Clovercut-<version>-aarch64.AppImage`
- `manifest.json` (binary URLs and digests, read by the in-app updater)
- `SHA256SUMS`

Download and try each AppImage before publishing the draft manually. Preparing
code or pushing the main branch does not publish a release. The Build Clovercut
AppImages workflow can also be run manually to test packaging without making a tag.

`scripts/build-appimage.sh STAGE ARCH OUTPUT.AppImage` can package a prepared
stage locally. The stage needs the `clovercut` executable with an `$ORIGIN/lib`
RPATH, its runtime libraries in `lib/`, `clovercut.png`, the desktop entry,
`LICENSE` and `THIRD_PARTY_NOTICES.md`. A normal development binary is not a
portable release: use the workflow's dependency setup and shipping build profile.

Application state belongs to `io.github.tulcoturo.clovercut`. Project folders
retain the inherited format; the original application's settings are not moved.
The model mirror and community sound catalogue remain upstream services with
upstream attribution; binary updates are fetched exclusively from Clovercut.
