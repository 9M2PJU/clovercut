# Audio library

Clovercut’s **Sounds → OpenGameArt** shelf searches the existing public
[OpenGameArt](https://opengameart.org/) website directly. No account, API key,
Clovercut server or GitHub asset mirror is required. Audio remains hosted by
OpenGameArt; Clovercut downloads an original attachment when it is previewed
or added to a project.

## Browsing and licensing

Enable OpenGameArt in **Settings → Content libraries** first. External libraries
are optional and off by default; the Sounds page links to their settings.

Use Search or Browse OpenGameArt to load results. Searches are explicit:
nothing goes online at application startup or on each keystroke. Music and
sound effects have separate shelves, using OpenGameArt’s own audio types.
A short music loop remains music; duration does not determine its category. Results are paginated, 24 entries per page.

The search uses OpenGameArt’s CC0 filter. Before the first download, Clovercut
also checks the entry’s own page for a CC0 1.0 licence link and selects an
original audio attachment on the same host. Other licences, external file
hosts, preview montages and archive-only packs are rejected. The source arrow
opens the original page for credits or manual access to packs. If an entry
contains several audio attachments, this version uses the first one.

The author appears when the original attachment is resolved. Duration is
measured from the downloaded audio and shown in a separate aligned column.
Search pages do not provide original-file durations, so an undownloaded result
shows `—:—` until its first preview or use. The row shows its category and
source before then. A successful download must contain
an audio stream and a positive duration.

## Local files and offline access

The last search and its pagination context are cached in
`<app data>/sounds/manifest.json` (cache format 2). Resolved source metadata is
stored beside it as `<id>.json`. Original audio files are cached using a
source-derived name and a locally observed SHA-256 checksum. That checksum
detects later cache corruption; it is not a publisher-supplied checksum.

Cached results are shown on the next launch without a network request.
Downloaded audio can be previewed and used offline; new searches and first
uses require internet access. Failed refreshes preserve the previous results.
Adding audio copies it into the project’s own audio folder, so projects do
not depend on the cache or continued availability of the original website.
The **Mine** shelf continues to hold the user’s imported local audio.

## Provider adapter and verification

OpenGameArt serves HTML rather than a documented audio API. The adapter is
isolated in `concat-host/src/opengameart.rs`; changes to the website may require
updating its selectors. Requests run on workers, with ten seconds between
HTML page requests to respect the site’s robots.txt crawl delay. Browsing and
timeline editing remain responsive during that wait. Original file downloads
use the existing progress and cancellation mechanism.

Parser tests cover search results, entity decoding, source URLs, CC0 checking,
original attachments and rejection of previews, archives and external hosts.
Run the opt-in real-provider test from `src/` with the native build environment
loaded:

```sh
cargo test -p concat-host --lib live_search_download_and_offline_cache -- --ignored
```

It searches the live catalogue, downloads a known CC0 track, probes its audio,
and verifies that its cached copy can be reused.

## YouTube Audio Library and Freesound

Enable these sources separately in **Settings → Content libraries**. YouTube
uses the Clovercut Audio API and provides separate **Music** and **Sound effects**
sections, with search and pagination. The default API address is
`http://127.0.0.1:8917`; it can be changed in these settings. Remote addresses
must use HTTPS.

For local development, clone the [companion Audio API](https://github.com/tulcoturo/clovercut-audio-api)
and follow its Node.js and Chromium/Chrome/Edge prerequisites. In its directory,
perform the initial setup in this order:

```sh
npm run login   # Sign in yourself, open Audio Library, then close the window.
npm run connect
npm run sync
npm start
```

The service restores the dedicated local profile in a headless browser after
the visible window closes or the service restarts. A revoked or expired session
needs a new interactive sign-in. No VPS is needed. The service's README covers
browser setup, existing profiles and catalogue synchronization. The companion
service is installed separately; desktop packages do not bundle it.

Catalogue pages are cached for offline browsing. Audio is resolved when needed,
so expiring Google download URLs are not saved in preferences or projects.
Previously downloaded audio can be reused without Studio. If Studio is not
connected, Clovercut displays a translated connection message.

Results preserve each track's author and licence. CC BY 4.0 tracks display an
attribution requirement; the copy button copies the provided attribution text
or, when absent, track, author, licence and source information. Check the source
for the required wording. YouTube standard-licence tracks are distinct from
CC BY tracks; the library does not classify them all as CC0. Existing cached
tracks from the earlier community catalogue remain usable.

Freesound uses its [official API](https://freesound.org/docs/api/). Enter your
API key in the Freesound settings; Clovercut saves it in your local preferences
with private file permissions and sends it in the Authorization header.
The key is never needed in source code or shared project files. Search and
import use the high quality MP3 previews provided by Freesound. OAuth login
and downloads of original uploads are not required for these previews.

Freesound results retain their individual licences and authors. Use the source
arrow to check attribution and usage terms; these results are not all CC0.
Downloads from both sources use the same cancellation, audio validation,
checksum cache and project copying as OpenGameArt.

The opt-in end-to-end provider test searches both services, downloads one
track from each, probes the audio and reuses its cached copy:

```sh
# Supply your Freesound token through the environment, never in a committed file.
cargo test -p concat-host --test audio_providers_live -- --ignored
```

The test reads `CLOVERCUT_FREESOUND_KEY`. The YouTube portion also requires the
local Clovercut Audio API and its authenticated Studio connector.

The YouTube-specific live test checks both music and sound effects:

```sh
cargo test -p concat-host --lib youtube_audio::tests::live_api_search_preview_and_cache_both_types -- --ignored
```
