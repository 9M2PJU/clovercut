// SPDX-License-Identifier: AGPL-3.0-or-later
// Runs only inside the user's own Studio tab. Session headers stay in this closure.
(() => {
  if (window.__clovercutAudioBridge) return;
  const templates = new Map();
  const allowed = new Set(['/youtubei/v1/creator_music/list_tracks', '/youtubei/v1/creator_music/get_tracks']);
  const pathOf = url => { try { const u = new URL(url, location.href); return u.origin === location.origin && allowed.has(u.pathname) ? u.pathname : null; } catch { return null; } };
  const remember = (url, init) => {
    const path = pathOf(url);
    if (path && typeof init?.body === 'string') {
      try { templates.set(path, { url, init, body: JSON.parse(init.body) }); } catch { /* Not a JSON catalogue request. */ }
    }
  };
  const nativeFetch = window.fetch;
  const wrappedFetch = function (input, init) { remember(typeof input === 'string' ? input : input.url, init); return nativeFetch.apply(this, arguments); };
  window.fetch = wrappedFetch;
  const open = XMLHttpRequest.prototype.open, header = XMLHttpRequest.prototype.setRequestHeader, send = XMLHttpRequest.prototype.send;
  const states = new WeakMap();
  const wrappedOpen = function (method, url) { states.set(this, { method, url, headers: {} }); return open.apply(this, arguments); };
  const wrappedHeader = function (k, v) { const s = states.get(this); if (s) s.headers[k] = v; return header.apply(this, arguments); };
  const wrappedSend = function (body) { const s = states.get(this); if (s) remember(s.url, { method: s.method, headers: s.headers, body }); return send.apply(this, arguments); };
  XMLHttpRequest.prototype.open = wrappedOpen;
  XMLHttpRequest.prototype.setRequestHeader = wrappedHeader;
  XMLHttpRequest.prototype.send = wrappedSend;
  window.__clovercutAudioBridge = {
    version: 1,
    status() { return [...templates.keys()]; },
    async request(path, overrides = {}) {
      if (!allowed.has(path)) throw Error('Unsupported catalogue operation');
      const permitted = path.endsWith('/list_tracks') ? ['filter', 'pageInfo'] : ['trackIds'];
      if (Object.keys(overrides).some(k => !permitted.includes(k))) throw Error('Unsupported query field');
      const t = templates.get(path);
      if (!t) throw Error('Open the corresponding Audio Library view first');
      const response = await nativeFetch.call(window, t.url, { ...t.init, body: JSON.stringify({ ...t.body, ...overrides }), credentials: 'same-origin', signal: AbortSignal.timeout(25000) });
      if (!response.ok) throw Error('Studio request failed: ' + response.status);
      const data = await response.json();
      return { tracks: data.tracks, pageInfo: data.pageInfo };
    },
    dispose() {
      if (window.fetch === wrappedFetch) window.fetch = nativeFetch;
      if (XMLHttpRequest.prototype.open === wrappedOpen) XMLHttpRequest.prototype.open = open;
      if (XMLHttpRequest.prototype.setRequestHeader === wrappedHeader) XMLHttpRequest.prototype.setRequestHeader = header;
      if (XMLHttpRequest.prototype.send === wrappedSend) XMLHttpRequest.prototype.send = send;
      templates.clear(); delete window.__clovercutAudioBridge;
    },
  };
})();
