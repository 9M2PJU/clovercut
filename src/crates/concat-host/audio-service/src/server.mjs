// SPDX-License-Identifier: AGPL-3.0-or-later
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { resolve as resolvePath } from 'node:path';
const catalogueFile = () => process.env.CLOVERCUT_AUDIO_DATA ? pathToFileURL(resolvePath(process.env.CLOVERCUT_AUDIO_DATA, 'catalogue.json')) : new URL('../data/catalogue.json', import.meta.url);
import { validateCatalogue, queryTracks, audioUrl } from './catalogue.mjs';
import { studioRequest, StudioUnavailable } from './studio.mjs';
import { ensureStudio, openLogin } from './session.mjs';
import { collectCatalogue, saveCatalogue } from './sync.mjs';
export function createApi(catalogue, { resolve = async id => {
  const data = await studioRequest('get_tracks', { trackIds: [id] });
  return data.tracks.find(t => t.trackId === id)?.streamingAudioUrl;
}, now = Date.now, login = openLogin, load = null } = {}) {
  if (catalogue) validateCatalogue(catalogue);
  else if (!load) throw Error('A catalogue or loader is required');
  let byId = new Map((catalogue?.tracks ?? []).map(t => [t.id, t]));
  let loading;
  function prepareCatalogue() {
    if (catalogue) return;
    if (!loading) loading = load().then(value => {
      catalogue = validateCatalogue(value);
      byId = new Map(catalogue.tracks.map(t => [t.id, t]));
    }).catch(() => {}).finally(() => { loading = undefined; });
    throw new StudioUnavailable('Sign in to Studio, then refresh while the first catalogue loads');
  }
  const cache = new Map(), pending = new Map();
  async function resolveAudio(id) {
    const saved = cache.get(id);
    if (saved && saved.expiresAt > now() + 60_000) return saved.url;
    if (pending.has(id)) return pending.get(id);
    const task = (async () => { const result = audioUrl(await resolve(id), now()); cache.set(id, result); return result.url; })();
    pending.set(id, task);
    try { return await task; } finally { pending.delete(id); }
  }
  return createServer(async (req, res) => {
    const send = (status, data) => { res.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8' }); res.end(JSON.stringify(data)); };
    res.setHeader('Cache-Control', 'no-store');
    res.setHeader('X-Content-Type-Options', 'nosniff');
    const localHost = /^(?:127\.0\.0\.1|localhost)(?::\d+)?$/;
    if (!localHost.test(req.headers.host ?? '') || (req.headers.origin && req.headers.origin !== 'http://' + req.headers.host)) return send(403, { error: 'Local access only' });
    const requestUrl = new URL(req.url, 'http://127.0.0.1');
    if (requestUrl.pathname === '/v1/session/login' && req.method === 'POST') {
      // A custom header prevents a website from triggering browser side effects.
      // No CORS preflight is authorized by this local service.
      if (req.headers['x-clovercut-client'] !== 'desktop') return send(403, { error: 'Desktop request required' });
      try { await login(); return send(200, { status: 'sign-in-opened' }); }
      catch { return send(503, { error: 'The dedicated Studio browser could not open' }); }
    }
    if (req.method !== 'GET') { res.setHeader('Allow', 'GET'); return send(405, { error: 'GET required' }); }
    const url = new URL(req.url, 'http://127.0.0.1');
    try {
      if (url.pathname === '/health') return send(200, { status: 'ok', ready: !!catalogue, updatedAt: catalogue?.updatedAt, counts: catalogue?.counts });
      prepareCatalogue();
      if (url.pathname === '/v1/tracks') return send(200, queryTracks(catalogue, url.searchParams));
      if (url.pathname === '/v1/facets') {
        const type = url.searchParams.get('type');
        if (type && !['music', 'sound-effect'].includes(type)) return send(400, { error: 'Invalid type' });
        const tracks = catalogue.tracks.filter(t => !type || t.type === type);
        const unique = key => [...new Set(tracks.flatMap(t => Array.isArray(t[key]) ? t[key] : t[key] ? [t[key]] : []))].sort();
        return send(200, { categories: unique('category'), genres: unique('genres'), moods: unique('moods'), instruments: unique('instruments') });
      }
      const match = /^\/v1\/tracks\/([A-Za-z0-9_-]{1,128})(?:\/(audio|download))?$/.exec(url.pathname);
      if (!match || !byId.has(match[1])) return send(404, { error: 'Track not found' });
      const track = byId.get(match[1]);
      if (!match[2]) return send(200, track);
      const location = await resolveAudio(track.id);
      res.writeHead(307, { Location: location }); res.end();
    } catch (e) {
      if (e instanceof StudioUnavailable) return send(503, { error: e.message });
      if (url.pathname === '/v1/tracks') return send(400, { error: e.message });
      return send(502, { error: 'The current audio URL could not be obtained from Studio' });
    }
  });
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  let catalogue;
  try { catalogue = JSON.parse(await readFile(catalogueFile(), 'utf8')); validateCatalogue(catalogue); } catch { catalogue = null; }
  const port = Number(process.env.PORT ?? 8917);
  if (!Number.isInteger(port) || port < 1024 || port > 65535) throw Error('Invalid PORT');
  createApi(catalogue, { load: async () => { const value = await collectCatalogue(); await saveCatalogue(value); return value; } }).listen(port, '127.0.0.1', () => {
    console.log(`Clovercut Audio API: http://127.0.0.1:${port}`);
    ensureStudio().then(() => console.log('Studio session ready in the background.')).catch(() => console.log('Studio sign-in is needed. Run npm run login once.'));
  });
}
