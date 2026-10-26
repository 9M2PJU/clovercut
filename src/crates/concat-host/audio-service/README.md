# Local Audio Library connector

Vendored from https://github.com/tulcoturo/clovercut-audio-api at `3efd33c`
(AGPL-3.0-or-later).
The desktop app extracts these source files into its private data directory and
starts the connector only when YouTube Audio Library is used. No catalogue,
browser profile or credentials are included in the application or repository.

A pinned, SHA-256-verified Node runtime is downloaded on first use if no suitable
Node runtime is installed. Chrome, Chromium or Edge must be installed. The Studio
session stays in a dedicated local browser profile, restored in the background.
Use **Sign in to YouTube Studio** when it expires; no external server is required.
The first catalogue is collected locally after sign-in; refresh to see it.
