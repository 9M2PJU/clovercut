// SPDX-License-Identifier: AGPL-3.0-or-later
export const SOURCE = 'https://www.youtube.com/audiolibrary';
export function normalizeTrack(track, type) {
  if (!['music', 'sound-effect'].includes(type)) throw Error('Unknown track type');
  if (!/^[A-Za-z0-9_-]{1,128}$/.test(track.trackId ?? '') || typeof track.title !== 'string' || !track.title.trim()) throw Error('Invalid Studio track');
  const seconds = Number(track.duration?.seconds ?? 0), nanos = Number(track.duration?.nanos ?? 0);
  if (!Number.isFinite(seconds) || seconds < 0 || !Number.isFinite(nanos) || nanos < 0 || nanos >= 1e9) throw Error('Invalid duration');
  const string = v => typeof v === 'string' && v.trim() ? v.trim() : null;
  const list = v => Array.isArray(v) ? v.filter(x => typeof x === 'string') : [];
  const license = string(track.licenseType);
  const link = value => { try { const url = new URL(value); return ['https:', 'http:'].includes(url.protocol) && !url.username && !url.password ? url.href : null; } catch { return null; } };
  const ccBy = license === 'CREATOR_MUSIC_LICENSE_TYPE_CCBY_4';
  return {
    id: track.trackId, name: track.title.trim(), type,
    author: string(track.artist?.name), duration: seconds + nanos / 1e9,
    category: string(track.attributes?.category),
    genres: list(track.attributes?.genres), moods: list(track.attributes?.moods),
    instruments: list(track.attributes?.instruments),
    license: { code: license, requiresAttribution: license === 'CREATOR_MUSIC_LICENSE_TYPE_YOUTUBE' ? false : ccBy ? true : null,
      url: ccBy ? 'https://creativecommons.org/licenses/by/4.0/' : null,
      attribution: string(track.attributionText), artistUrl: link(track.externalArtistUrl), trackUrl: link(track.externalTrackUrl), sourceUrl: SOURCE },
    publishedAt: track.publishTime?.seconds && Number.isFinite(Number(track.publishTime.seconds))
      ? new Date(Number(track.publishTime.seconds) * 1000).toISOString() : null,
    sourceUrl: SOURCE,
  };
}
export function validateCatalogue(data) {
  if (data?.version !== 1 || !Array.isArray(data.tracks) || !data.tracks.length) throw Error('A nonempty version 1 catalogue is required');
  const seen = new Set();
  for (const t of data.tracks) {
    if (!t || !/^[A-Za-z0-9_-]{1,128}$/.test(t.id ?? '') || !['music', 'sound-effect'].includes(t.type) ||
        typeof t.name !== 'string' || !t.name.trim() || !Number.isFinite(t.duration) || t.duration < 0 || seen.has(t.id)) throw Error('Invalid or duplicate catalogue track');
    seen.add(t.id);
  }
  return data;
}
export function queryTracks(catalogue, params) {
  const type = params.get('type');
  if (type && !['music', 'sound-effect'].includes(type)) throw Error('type must be music or sound-effect');
  const number = (name, fallback, max = Infinity) => {
    const v = params.get(name);
    if (v === null) return fallback;
    if (!/^\d+(?:\.\d+)?$/.test(v) || !Number.isFinite(Number(v)) || Number(v) > max) throw Error('Invalid ' + name);
    return Number(v);
  };
  const offset = number('offset', 0, 1e7), limit = number('limit', 50, 200);
  if (!Number.isInteger(offset) || !Number.isInteger(limit) || limit < 1) throw Error('offset and limit must be integers; limit must be 1..200');
  const min = number('durationMin', 0), max = number('durationMax', Infinity);
  if (min > max) throw Error('durationMin must not exceed durationMax');
  const q = (params.get('q') ?? '').normalize('NFKC').toLocaleLowerCase();
  if (q.length > 500) throw Error('Search query is too long');
  const items = catalogue.tracks.filter(t => (!type || t.type === type)
    && (!params.get('category') || t.category === params.get('category'))
    && (!params.get('genre') || t.genres.includes(params.get('genre')))
    && (!params.get('mood') || t.moods.includes(params.get('mood')))
    && t.duration >= min && t.duration <= max
    && (!q || `${t.name} ${t.author ?? ''}`.normalize('NFKC').toLocaleLowerCase().includes(q)));
  return { total: items.length, offset, limit, items: items.slice(offset, offset + limit),
    nextOffset: offset + limit < items.length ? offset + limit : null };
}
export function audioUrl(value, now = Date.now()) {
  let url; try { url = new URL(value); } catch { throw Error('Missing audio URL'); }
  if (url.protocol !== 'https:' || url.username || url.password || url.port ||
      !(url.hostname === 'googlevideo.com' || url.hostname.endsWith('.googlevideo.com')) ||
      url.pathname !== '/videoplayback') throw Error('Unexpected audio source');
  const expires = Number(url.searchParams.get('expire'));
  if (!Number.isFinite(expires) || expires * 1000 <= now + 60_000) throw Error('Audio URL expired or near expiry');
  // Google streams may stall on an unbounded request. Use the exact declared
  // file range so native preview/import can finish downloading the whole MP3.
  const length = Number(url.searchParams.get('clen'));
  if (Number.isSafeInteger(length) && length > 0 && length <= 512 * 1024 * 1024) url.searchParams.set('range', `0-${length - 1}`);
  return { url: url.href, expiresAt: expires * 1000 };
}
