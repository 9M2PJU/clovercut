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
uses the public community catalogue maintained by
[YouTube-Free-Audio-Library-API](https://github.com/ThibaultJanBeyer/YouTube-Free-Audio-Library-API),
with downloads from Google Drive. This is a community index, so individual
tracks can disappear. The catalogue is refreshed after 24 hours.

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

The test reads `CLOVERCUT_FREESOUND_KEY`.
