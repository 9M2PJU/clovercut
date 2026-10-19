# Contributing to Clovercut

Clovercut is an independent Linux video editor based on Concat. Report bugs and
suggest improvements in [Clovercut issues](https://github.com/tulcoturo/clovercut/issues).
For bugs, include the version, Linux distribution, steps to reproduce, relevant
logs and a small sample project or media file when possible.

## Development

Follow the dependency and build instructions in [README.md](README.md).
The Rust workspace is in `src/`; [src/README.md](src/README.md) describes its crates.
Release packaging targets Linux AppImages only; see [docs/RELEASING.md](docs/RELEASING.md).

Before opening a pull request, run from `src/`:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked -j 1
```

From the repository root, also run:

```sh
python3 scripts/locales.py --check
python3 scripts/models.py --check
```

Interface strings belong in `src/crates/concat/locales/en.json`; run
`python3 scripts/locales.py` after changing them. See [TRANSLATING.md](TRANSLATING.md).
Keep downloadable models aligned with `models/manifest.toml` and the engine tables.

Use the `log` facade for diagnostics. On Linux, application logs live under
`~/.local/share/io.github.tulcoturo.clovercut/logs/`.

## Contributions and attribution

Contributions are made under the project's **AGPL-3.0-or-later** licence.
Preserve existing copyright notices and identify any third-party code and its
licence in your pull request. New source files should carry an SPDX header:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
```

The inherited Concat name and logo belong to their upstream owners; see
[TRADEMARK.md](TRADEMARK.md). Clovercut uses its own application identity.
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) applies to participation in this repository.
