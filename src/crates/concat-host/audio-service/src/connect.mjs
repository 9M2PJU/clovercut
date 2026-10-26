// SPDX-License-Identifier: AGPL-3.0-or-later
import { ensureStudio } from './session.mjs';
await ensureStudio({ reconnect: process.argv.includes('--reconnect') });
console.log('Studio connected. The local service can restore the session in the background after the visible window closes.');
