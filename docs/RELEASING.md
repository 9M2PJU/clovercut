# Clovercut releases

Clovercut prepares Linux AppImages for x86_64 and aarch64, plus a Windows x64
installer and portable ZIP. Linux builds use Ubuntu 22.04; Windows builds use
Windows Server 2022. Native dependency downloads stay checksum-pinned in
`scripts/ci/deps.lock`.

Before creating a release:

1. Set `[workspace.package].version` in `src/Cargo.toml` and update the workspace entries in `src/Cargo.lock`.
2. Write the matching `docs/releases/<version>.md` and review the README.
3. Run formatting, Clippy, tests, locale and model inventory checks. Compile the editor and check `clovercut --version`, `clovercut --smoke`, editing and export.
4. Commit and push the reviewed changes. Create and push the matching tag only when ready (for this release, `v0.0.2`).

Before tagging, the manual **Build Clovercut packages** workflow can build a
specific commit from the updated main branch. Push all intended changes first,
then dispatch with that exact commit as `ref`; a successful build produces
artifacts without publishing a release. Inspect those artifacts before tagging.
Do not rerun an old version tag expecting it to include newer main-branch changes.

The tag starts `.github/workflows/release.yml`: CI, two Linux builds and a Windows x64 build, staged-binary
checks, CPU compatibility checks on x86_64, AppImage packaging, and AppImage startup
checks in Ubuntu 22.04 and Debian 12 containers. Windows checks validate DLL
completeness, startup with a clean PATH, baseline CPU compatibility and the
instruction set before creating the installer and portable ZIP. Windows ARM64,
macOS, mobile, DEB, RPM and Arch packages are not built.

CI uses matching dev/test debug and codegen settings across lint and test
steps. Cold test compilation has a 75-minute budget, separate from test execution;
this avoids the previous 25-minute compilation timeout blocking packaging.

The result is a **draft** GitHub release containing:

- `Clovercut-<version>-x86_64.AppImage`
- `Clovercut-<version>-aarch64.AppImage`
- `Clovercut-<version>-windows-x86_64-setup.exe`
- `Clovercut-<version>-windows-x86_64-portable.zip`
- `manifest.json` (binary URLs and digests, read by the in-app updater)
- `SHA256SUMS`

Download and try the AppImages, Windows installation/uninstallation and portable
Windows launch before publishing the draft manually. The installer has a
Clovercut-specific application ID and does not replace Concat. Signing is not
configured; Windows packages are unsigned. All four files must be present before
the release workflow creates its checksums. Preparing
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
