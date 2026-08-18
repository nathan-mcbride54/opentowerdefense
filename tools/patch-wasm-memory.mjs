import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// wasm-bindgen caches a Uint8Array on the wasm heap. After Memory.grow the old
// ArrayBuffer is replaced, but the cache only refreshes when byteLength === 0.
// In this Chromium that is not always true, so the next snapshot() reads a stale
// view and throws "memory access out of bounds". Workshop maps hit it first:
// fromMapJson + mapStatic + a parallel campaign() list grow the heap between
// the first HUD snapshot and the animation loop. Built-in theaters often fit
// in the initial pages and never grow.
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const path = join(root, 'web/src/lib/wasm/otd.js');
const src = readFileSync(path, 'utf8');
const next = src.replace(
	/function getUint8ArrayMemory0\(\) \{\r?\n\s*if \(cachedUint8ArrayMemory0 === null \|\| cachedUint8ArrayMemory0\.byteLength === 0\) \{/,
	'function getUint8ArrayMemory0() {\n    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.buffer !== wasm.memory.buffer) {'
);
if (next === src) {
	if (src.includes('cachedUint8ArrayMemory0.buffer !== wasm.memory.buffer')) {
		console.log('wasm memory view already patched');
		process.exit(0);
	}
	console.error('getUint8ArrayMemory0 shape changed; update tools/patch-wasm-memory.mjs');
	process.exit(1);
}
writeFileSync(path, next);
console.log('patched wasm memory view cache');
