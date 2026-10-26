// SPDX-License-Identifier: AGPL-3.0-or-later
import { mkdir, writeFile, rename } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { resolve as resolvePath } from 'node:path';
const catalogueFile = () => process.env.CLOVERCUT_AUDIO_DATA ? pathToFileURL(resolvePath(process.env.CLOVERCUT_AUDIO_DATA, 'catalogue.json')) : new URL('../data/catalogue.json', import.meta.url);
import { studioRequest } from './studio.mjs';
import { normalizeTrack, validateCatalogue } from './catalogue.mjs';
export async function collectCatalogue(request = studioRequest, report = () => {}) {
  const all = [], counts = {};
  for (const type of ['music', 'sound-effect']) {
    const tracks = new Map(), seenTokens = new Set(); let pageToken;
    for (let page = 0; ; page++) {
      if (page >= 1000) throw Error('Studio pagination exceeded 1000 pages');
      const data = await request('list_tracks', {
        filter: type === 'music' ? {} : { trackTypeIn: { values: ['CREATOR_MUSIC_TRACK_TYPE_SOUNDEFFECT'] } },
        pageInfo: { pageSize: 100, ...(pageToken ? { pageToken } : {}) },
      });
      if (!Array.isArray(data.tracks)) throw Error('Invalid track page');
      for (const t of data.tracks) { const item = normalizeTrack(t, type); tracks.set(item.id, item); }
      report(type, tracks.size);
      const next = data.pageInfo?.nextPageToken;
      if (!next) break;
      if (!data.tracks.length || seenTokens.has(next)) throw Error('Studio pagination stalled; previous catalogue preserved');
      seenTokens.add(next); pageToken = next;
      await new Promise(resolve => setTimeout(resolve, 200));
    }
    if (!tracks.size) throw Error('Empty ' + type + ' catalogue; previous catalogue preserved');
    counts[type] = tracks.size; all.push(...tracks.values());
  }
  return validateCatalogue({ version: 1, updatedAt: new Date().toISOString(), counts, tracks: all });
}
export async function saveCatalogue(catalogue, file = catalogueFile()) {
  await mkdir(new URL('.', file), { recursive: true });
  const temp = new URL(file.href + '.tmp');
  await writeFile(temp, JSON.stringify(catalogue, null, 2) + '\n', { mode: 0o600 });
  await rename(temp, file);
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const catalogue = await collectCatalogue(studioRequest, (type, count) => console.log(type, count));
  await saveCatalogue(catalogue); console.log('Saved', JSON.stringify(catalogue.counts));
}
