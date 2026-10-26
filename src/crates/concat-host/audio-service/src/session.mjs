// SPDX-License-Identifier: AGPL-3.0-or-later
import { spawn } from 'node:child_process';
import { access, mkdir } from 'node:fs/promises';
import { constants } from 'node:fs';
import { delimiter, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { connect } from './cdp.mjs';
import { connectBridge } from './bridge.mjs';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const endpoint = 'http://127.0.0.1:9223/json/list';
export const profilePath = () => resolve(process.env.CLOVERCUT_STUDIO_PROFILE || fileURLToPath(new URL('../data/browser-profile', import.meta.url)));
export async function browserPath() {
  const names = process.platform === 'win32' ? ['msedge.exe', 'chrome.exe'] : ['chromium', 'chromium-browser', 'google-chrome', 'google-chrome-stable'];
  const candidates = [process.env.CLOVERCUT_CHROMIUM, ...String(process.env.PATH ?? '').split(delimiter).flatMap(dir => names.map(name => join(dir, name)))];
  if (process.platform === 'darwin') candidates.push('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', '/Applications/Chromium.app/Contents/MacOS/Chromium', '/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge');
  if (process.platform === 'win32') for (const dir of [process.env.PROGRAMFILES, process.env['PROGRAMFILES(X86)'], process.env.LOCALAPPDATA].filter(Boolean)) candidates.push(join(dir,'Microsoft/Edge/Application/msedge.exe'),join(dir,'Google/Chrome/Application/chrome.exe'));
  for (const candidate of candidates.filter(Boolean)) { try { await access(candidate, process.platform === 'win32' ? constants.F_OK : constants.X_OK); return candidate; } catch {} }
  throw Error('Install Chromium, Chrome or Edge, or set CLOVERCUT_CHROMIUM to its executable');
}
export async function startBrowser({ visible = false } = {}) {
  const profile = profilePath();
  await mkdir(profile, { recursive: true, mode: 0o700 });
  const args = [`--user-data-dir=${profile}`, '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=9223', '--no-first-run', '--no-default-browser-check'];
  if (!visible) args.push('--headless=new', '--disable-gpu', '--enable-automation');
  args.push('https://studio.youtube.com');
  const child = spawn(await browserPath(), args, { stdio: 'ignore', windowsHide: !visible });
  await new Promise((resolve, reject) => { child.once('spawn', resolve); child.once('error', reject); });
  child.unref();
  return child;
}
async function browserAvailable() {
  try { return (await fetch(endpoint, { signal: AbortSignal.timeout(1000) })).ok; } catch { return false; }
}
async function prepareBridge(options) {
  let c;
  for (let i = 0; i < 60; i++) {
    try { c = await connect({ allowStudioHome: true }); break; } catch { await sleep(500); }
  }
  if (!c) throw Error('Studio sign-in is required; run npm run login');
  try {
    const state = await c.call('Runtime.evaluate', { expression: 'window.__clovercutAudioBridge?.status()', returnByValue: true });
    if (!options.reconnect && ['list_tracks', 'get_tracks'].every(kind => state.result?.value?.some(path => path.endsWith('/' + kind)))) return;
    await c.call('Page.enable');
    const version = await c.call('Browser.getVersion');
    if (version.userAgent.includes('HeadlessChrome/')) {
      // Studio's compatibility screen rejects the headless branding even though
      // this is the installed Chromium engine. Keep its actual version/platform.
      await c.call('Network.setUserAgentOverride', { userAgent: version.userAgent.replace('HeadlessChrome/', 'Chrome/') });
      await c.call('Page.reload');
      await sleep(1000);
    }
    let musicUrl;
    for (let i = 0; i < 60; i++) {
      const response = await c.call('Runtime.evaluate', { expression: `(() => {
        const proceed = [...document.querySelectorAll('a, button, ytcp-button')].find(e => /^(aceder ao youtube studio|continue to youtube studio|go to youtube studio)$/i.test(e.textContent.trim()));
        if (proceed) { proceed.click(); return null; }
        const channel = location.pathname.match(/^\\/channel\\/[^/]+/);
        if (channel) { const target = new URL(location.href); target.pathname = channel[0] + '/music'; return target.href; }
        return null;
      })()`, returnByValue: true });
      musicUrl = response.result?.value;
      if (musicUrl) break;
      await sleep(500);
    }
    if (!musicUrl) throw Error('Studio sign-in is required; run npm run login');
    if (!/\/music(?:[/?#]|$)/.test(c.pageUrl)) {
      await c.call('Page.navigate', { url: musicUrl });
      await sleep(1500);
    }
    await connectBridge(c, options);
  } finally { c.close(); }
}
// One recovery per service, including simultaneous playback requests. Only an
// explicitly requested login opens a visible window; audio requests stay headless.
export function createSessionManager({ available = browserAvailable, launch = startBrowser, prepare = prepareBridge, wait = sleep } = {}) {
  let pending;
  return {
    ensure(options = {}) {
      if (pending) return pending;
      pending = (async () => {
        if (!await available()) {
          const child = await launch({ visible: false });
          let ready = false;
          for (let i = 0; i < 60; i++) {
            if (await available()) { ready = true; break; }
            if (child.exitCode != null) throw Error('The local Studio browser could not start');
            await wait(500);
          }
          if (!ready) throw Error('The local Studio browser did not become ready');
        }
        await prepare(options);
      })().finally(() => { pending = undefined; });
      return pending;
    },
  };
}
const manager = createSessionManager();
export const ensureStudio = options => manager.ensure(options);
export async function openLogin() {
  if (await browserAvailable()) {
    const c = await connect({ allowStudioHome: true, allowLogin: true });
    try {
      const { arguments: args } = await c.call('Browser.getBrowserCommandLine');
      if (!args?.includes('--user-data-dir=' + profilePath()) || !args.includes('--remote-debugging-port=9223')) throw Error('Close the dedicated Studio browser before starting interactive sign-in');
      await c.call('Browser.close');
    } finally { c.close(); }
    for (let i = 0; i < 40 && await browserAvailable(); i++) await sleep(250);
    if (await browserAvailable()) throw Error('The dedicated Studio browser is still closing');
    await sleep(500);
  }
  await startBrowser({ visible: true });
}
