// SPDX-License-Identifier: AGPL-3.0-or-later
import { openLogin } from './session.mjs';
await openLogin();
console.log('Sign in to YouTube Studio in this dedicated browser, then open Audio Library. Close the window when finished; the service will reuse this local profile in the background.');
