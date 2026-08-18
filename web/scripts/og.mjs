/**
 * Renders web/static/og.png (1200x630) from scripts/og.html using whatever
 * headless Chromium is on the machine. This is the image chat apps and social
 * feeds show when the site URL is pasted.
 *
 *   npm run og
 *
 * No npm dependency: Chrome/Edge's built-in --screenshot is enough for a static card.
 */
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const SRC = join(here, 'og.html');
const OUT = join(here, '..', 'static', 'og.png');

const candidates = [
	process.env.CHROME,
	'C:/Program Files/Google/Chrome/Application/chrome.exe',
	'C:/Program Files (x86)/Google/Chrome/Application/chrome.exe',
	'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
	'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
	'/usr/bin/google-chrome',
	'/usr/bin/chromium',
	'/usr/bin/chromium-browser',
	'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
].filter(Boolean);

const browser = candidates.find((p) => existsSync(p));
if (!browser) {
	console.error('No Chrome/Chromium/Edge found. Set CHROME=/path/to/browser and retry.');
	process.exit(1);
}

// A throwaway profile keeps this from touching (or being blocked by) a running browser.
const profile = mkdtempSync(join(tmpdir(), 'otd-og-'));
try {
	execFileSync(
		browser,
		[
			'--headless=new',
			'--disable-gpu',
			'--hide-scrollbars',
			'--force-device-scale-factor=1',
			`--user-data-dir=${profile}`,
			'--window-size=1200,630',
			// Give the web fonts a moment to arrive before the capture.
			'--virtual-time-budget=6000',
			`--screenshot=${OUT}`,
			pathToFileURL(SRC).href
		],
		{ stdio: 'pipe' }
	);
} finally {
	rmSync(profile, { recursive: true, force: true });
}
console.log(`wrote ${OUT}`);
