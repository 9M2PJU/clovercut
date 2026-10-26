// SPDX-License-Identifier: AGPL-3.0-or-later
import { readFile } from 'node:fs/promises';
export async function connectBridge(c, { reconnect = false, sleep = ms => new Promise(r => setTimeout(r, ms)) } = {}) {
  const evaluate = async expression => {
    const r = await c.call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.exceptionDetails) throw Error('Studio interface is unavailable; sign in again');
    return r.result.value;
  };
  const status = await evaluate('window.__clovercutAudioBridge?.status()');
  if (!reconnect && Array.isArray(status) && ['get_tracks', 'list_tracks'].every(kind => status.some(x => x.endsWith('/' + kind)))) return;
  await c.call('Page.enable');
  const source = await readFile(new URL('./browser-bridge.js', import.meta.url), 'utf8');
  const { identifier } = await c.call('Page.addScriptToEvaluateOnNewDocument', { source });
  try {
    await c.call('Page.reload');
    let ready = false;
    for (let i = 0; i < 60; i++) {
      await sleep(500);
      ready = await evaluate('Boolean(window.__clovercutAudioBridge?.status().some(p=>p.endsWith("/list_tracks")))');
      if (ready) break;
    }
    if (!ready) throw Error('Studio sign-in is required before connecting the Audio Library');
    let played = false;
    for (let i = 0; i < 40; i++) {
      played = await evaluate(`(() => { const button = document.querySelector('ytcp-icon-button[aria-label^="Reproduzir faixa de áudio"], ytcp-icon-button[aria-label^="Play audio track"]'); if (!button) return false; document.addEventListener('play', e => { if (e.target instanceof HTMLMediaElement) e.target.pause(); }, { capture: true, once: true }); button.click(); return true; })()`);
      if (played) break;
      await sleep(500);
    }
    if (!played) throw Error('The Studio music preview controls are unavailable');
    for (let i = 0; i < 20; i++) {
      await sleep(250);
      await evaluate('document.querySelectorAll("audio").forEach(a=>a.pause())');
      if (await evaluate('window.__clovercutAudioBridge.status().some(p=>p.endsWith("/get_tracks"))')) return;
    }
    throw Error('Studio audio could not be connected');
  } finally { await c.call('Page.removeScriptToEvaluateOnNewDocument', { identifier }); }
}
