import { test } from 'node:test';
import assert from 'node:assert/strict';
import { runInNewContext } from 'node:vm';
import { connectBridge } from '../src/bridge.mjs';

for (const label of ['Play audio track Rain', 'Reproduzir faixa de áudio Chuva', 'Tocar faixa de áudio Chuva', 'Lire la piste audio Pluie', 'Audiotrack abspielen Regen', '音轨播放']) {
  test(`connects the Studio preview independently of its label: ${label}`, async () => {
    const paths = ['/youtubei/v1/creator_music/list_tracks'];
    let clicks = 0, listener, removed = false;
    const button = { click() { clicks++; paths.push('/youtubei/v1/creator_music/get_tracks'); } };
    const document = {
      querySelector(selector) {
        // Model the actual Studio row observed in Chromium. The label changes
        // with the account language; the row element and play ID do not.
        if (selector === 'ytmus-library-row ytcp-icon-button#play') return button;
        const prefixes = [...selector.matchAll(/aria-label\^="([^"]+)"/g)];
        return prefixes.some(([, prefix]) => label.startsWith(prefix)) ? button : null;
      },
      querySelectorAll() { return []; },
      addEventListener(type, handler, options) {
        assert.equal(type, 'play'); assert.equal(options.capture, true);
        assert.equal(options.once, true); listener = handler;
      },
    };
    class Media { pause() { this.paused = true; } }
    const window = { __clovercutAudioBridge: { status: () => paths } };
    const c = { async call(method, params) {
      if (method === 'Runtime.evaluate') return { result: { value: runInNewContext(params.expression, { window, document, HTMLMediaElement: Media }) } };
      if (method === 'Page.addScriptToEvaluateOnNewDocument') return { identifier: 'fixture' };
      if (method === 'Page.removeScriptToEvaluateOnNewDocument') { removed = true; return {}; }
      assert.ok(['Page.enable', 'Page.reload'].includes(method)); return {};
    } };
    await connectBridge(c, { sleep: async () => {} });
    assert.equal(clicks, 1); assert.equal(removed, true);
    const media = new Media(); listener({ target: media });
    assert.equal(media.paused, true, 'bootstrap preview must remain silent');
  });
}
