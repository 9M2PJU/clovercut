// SPDX-License-Identifier: AGPL-3.0-or-later
export async function connect({ allowStudioHome = false, allowLogin = false } = {}) {
  const response = await fetch('http://127.0.0.1:9223/json/list', { signal: AbortSignal.timeout(5000) });
  if (!response.ok) throw Error('Studio browser is unavailable');
  const pages = await response.json();
  const studio = pages.filter(p => p.type === 'page' && p.url.startsWith('https://studio.youtube.com/') && !/^(Ups!|Oops)/i.test(p.title));
  const page = studio.find(p => /\/music(?:[/?#]|$)/.test(p.url)) ?? (allowStudioHome ? studio[0] : undefined) ?? (allowLogin ? pages.find(p => p.type === 'page' && p.url.startsWith('https://accounts.google.com/')) : undefined);
  if (!page) throw Error('The authenticated Audio Library tab is unavailable');
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => { ws.close(); reject(Error('Studio browser connection timed out')); }, 5000);
    ws.onopen = () => { clearTimeout(timer); resolve(); };
    ws.onerror = () => { clearTimeout(timer); reject(Error('Studio browser connection failed')); };
  });
  let next = 0; const pending = new Map();
  const finish = () => { for (const p of pending.values()) { clearTimeout(p.timer); p.reject(Error('Studio browser disconnected')); } pending.clear(); };
  ws.onclose = finish; ws.onerror = finish;
  ws.onmessage = e => {
    const m = JSON.parse(e.data), p = pending.get(m.id);
    if (!p) return;
    pending.delete(m.id); clearTimeout(p.timer);
    m.error ? p.reject(Error(m.error.message)) : p.resolve(m.result);
  };
  return {
    pageUrl: page.url,
    call(method, params = {}) {
      return new Promise((resolve, reject) => {
        const id = ++next;
        const timer = setTimeout(() => { pending.delete(id); reject(Error(method + ' timed out')); }, 30000);
        pending.set(id, { resolve, reject, timer });
        ws.send(JSON.stringify({ id, method, params }));
      });
    },
    close() { finish(); ws.close(); },
  };
}
