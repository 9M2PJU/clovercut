# YouTube Audio API desktop integration — 25 October 2026

The desktop provider now uses the Clovercut Audio API instead of the community
Google Drive index. Music and sound effects have independent sidebar choices,
search and 24-item pages. Provider/type changes invalidate stale replies.
The service address is configurable in Content libraries and defaults to
`http://127.0.0.1:8917`. HTTP is accepted only on loopback; remote addresses
require HTTPS.

Per-track author, duration and licence are retained. CC BY 4.0 is explicitly
marked as requiring attribution. A compact copy button copies supplied credit
text or available metadata and source links; missing official credit text is
not fabricated. New controls and connection errors are translated in all 25
language catalogues.

Search pages are cached by canonical request URL. Cached audio uses stable
track identities and SHA-256 verification; expiring signed Google URLs are
resolved just before download and are not stored in project metadata.
Changing the API address preserves usable audio and current credit metadata.
Previously cached community-index tracks remain supported.

## Verification

- Five focused client tests passed: endpoint and stream validation, metadata and
  licence retention, pagination/query encoding and offline page reuse, redirect
  handling and Studio errors, cached audio reuse after service-address changes.
- Six existing sound-library tests passed.
- Host Clippy passed for all targets with warnings denied.
- Slint interface compilation passed.
- Strict locale validation passed for 1031 keys across 25 languages; all six
  locale-checker tests passed.
- The full application built with the existing quick profile in 19m48s after
  correcting a message-format argument type. Priority nice15/ionice7 and
  temporary 24 GiB Btrfs swap at priority150 were used; profiles, jobs and CGUs
  were unchanged. Build log: `/tmp/clovercut-youtube-integration-build-fixed.log`.
- Isolated startup smoke passed on Radeon RX6600XT Vulkan and Whisper.
- The opt-in test `live_api_search_preview_and_cache_both_types` passed against
  the authenticated Studio catalogue: music Mess Call and effect Straw Squeak
  were downloaded, probed, hashed and reused from cache. It uses the same host
  code as preview/import. GUI clicks and clipboard behaviour were not automated.

Binary: `src/target/quick/clovercut`, version0.1.6, 390515056 bytes.
SHA-256: `58a300a0fea437d3b71713368db278f9b36c2a771cd8c10de1870d909d49419a`.

The companion service is a local prototype. Fresh downloads require its
running service and authenticated Studio bridge; catalogue/audio caches remain
usable offline. Public hosting and unattended renewal are separate work.
No release or GitHub push was performed.
