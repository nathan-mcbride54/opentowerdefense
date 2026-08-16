/**
 * Generates web/static/favicon.ico from the same lantern design as favicon.svg.
 *
 *   npm run favicon
 *
 * Written by hand rather than pulled from a package: the shape is a handful of
 * primitives, and this keeps the repo dependency-free. Emits 16/32/48px entries as
 * 32-bit BGRA DIBs, which every Windows shell and legacy browser understands.
 */
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const OUT = join(dirname(fileURLToPath(import.meta.url)), '..', 'static', 'favicon.ico');
const SIZES = [16, 32, 48];

const hex = (h) => [
	parseInt(h.slice(1, 3), 16),
	parseInt(h.slice(3, 5), 16),
	parseInt(h.slice(5, 7), 16)
];

const BG = hex('#0a0a0b');
const CAP = hex('#c47f2c');
const RIM = hex('#8a6a34');
const LIT = hex('#ffc46e');
const DEEP = hex('#c47f2c');
const CORE = hex('#fff6dc');
const GLOW = hex('#f0a94c');
const HOOK = hex('#8a7a5c');

const mix = (a, b, t) => a.map((v, i) => Math.round(v + (b[i] - v) * t));

/** Render the lantern at `size`, returning RGBA in row-major top-down order. */
function render(size) {
	const px = new Uint8Array(size * size * 4);
	const u = (v) => v * size; // design is authored in 0..1
	const put = (x, y, rgb, a = 1) => {
		if (x < 0 || y < 0 || x >= size || y >= size) return;
		const i = (y * size + x) * 4;
		const prev = [px[i], px[i + 1], px[i + 2]];
		const out = mix(prev, rgb, a);
		px[i] = out[0];
		px[i + 1] = out[1];
		px[i + 2] = out[2];
		px[i + 3] = 255;
	};

	for (let y = 0; y < size; y++) {
		for (let x = 0; x < size; x++) {
			const i = (y * size + x) * 4;
			px[i] = BG[0];
			px[i + 1] = BG[1];
			px[i + 2] = BG[2];
			px[i + 3] = 255;
		}
	}

	// Warm halo behind the lamp.
	const cx = u(0.5);
	const cy = u(0.47);
	const rad = u(0.5);
	for (let y = 0; y < size; y++) {
		for (let x = 0; x < size; x++) {
			const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy) / rad;
			if (d < 1) put(x, y, GLOW, 0.42 * (1 - d) ** 1.6);
		}
	}

	const rect = (x0, y0, x1, y1, rgb, a = 1) => {
		for (let y = Math.round(u(y0)); y < Math.round(u(y1)); y++)
			for (let x = Math.round(u(x0)); x < Math.round(u(x1)); x++) put(x, y, rgb, a);
	};

	// Hook and cap.
	rect(0.47, 0.06, 0.53, 0.17, HOOK);
	for (let y = Math.round(u(0.15)); y < Math.round(u(0.31)); y++) {
		const t = (y - u(0.15)) / (u(0.31) - u(0.15));
		const half = u(0.04 + 0.19 * t);
		for (let x = Math.round(cx - half); x < Math.round(cx + half); x++) put(x, y, CAP);
	}
	rect(0.26, 0.31, 0.74, 0.36, RIM);

	// Housing, tapering slightly toward the base.
	const top = u(0.36);
	const bot = u(0.79);
	for (let y = Math.round(top); y < Math.round(bot); y++) {
		const t = (y - top) / (bot - top);
		const half = u(0.175 - 0.03 * t);
		for (let x = Math.round(cx - half); x < Math.round(cx + half); x++) {
			put(x, y, mix(LIT, DEEP, t));
		}
	}

	// Flame core.
	const fy = u(0.55);
	const rx = u(0.092);
	const ry = u(0.115);
	for (let y = 0; y < size; y++) {
		for (let x = 0; x < size; x++) {
			const dx = (x + 0.5 - cx) / rx;
			const dy = (y + 0.5 - fy) / ry;
			const d = dx * dx + dy * dy;
			if (d <= 1) put(x, y, CORE, d > 0.55 ? 0.75 : 1);
		}
	}

	// Corner ribs so it reads as a lamp rather than a blob, then the base.
	if (size >= 32) {
		rect(0.38, 0.36, 0.41, 0.79, RIM, 0.6);
		rect(0.59, 0.36, 0.62, 0.79, RIM, 0.6);
	}
	rect(0.33, 0.79, 0.67, 0.87, RIM);
	return px;
}

/** One ICO entry: BITMAPINFOHEADER + bottom-up BGRA + 1bpp AND mask. */
function dib(size, rgba) {
	const header = Buffer.alloc(40);
	header.writeUInt32LE(40, 0);
	header.writeInt32LE(size, 4);
	header.writeInt32LE(size * 2, 8); // XOR + AND
	header.writeUInt16LE(1, 12);
	header.writeUInt16LE(32, 14);
	header.writeUInt32LE(size * size * 4, 20);

	const xor = Buffer.alloc(size * size * 4);
	for (let y = 0; y < size; y++) {
		const src = size - 1 - y; // ICO stores rows bottom-up
		for (let x = 0; x < size; x++) {
			const s = (src * size + x) * 4;
			const d = (y * size + x) * 4;
			xor[d] = rgba[s + 2];
			xor[d + 1] = rgba[s + 1];
			xor[d + 2] = rgba[s];
			xor[d + 3] = rgba[s + 3];
		}
	}
	// Fully opaque, so the mask is all zeroes — but the rows must still be 4-byte aligned.
	const maskRow = Math.ceil(size / 32) * 4;
	return Buffer.concat([header, xor, Buffer.alloc(maskRow * size)]);
}

const images = SIZES.map((s) => dib(s, render(s)));
const dir = Buffer.alloc(6 + 16 * SIZES.length);
dir.writeUInt16LE(0, 0);
dir.writeUInt16LE(1, 2);
dir.writeUInt16LE(SIZES.length, 4);
let offset = dir.length;
SIZES.forEach((s, i) => {
	const at = 6 + i * 16;
	dir.writeUInt8(s === 256 ? 0 : s, at);
	dir.writeUInt8(s === 256 ? 0 : s, at + 1);
	dir.writeUInt16LE(1, at + 4);
	dir.writeUInt16LE(32, at + 6);
	dir.writeUInt32LE(images[i].length, at + 8);
	dir.writeUInt32LE(offset, at + 12);
	offset += images[i].length;
});

writeFileSync(OUT, Buffer.concat([dir, ...images]));
console.log(`favicon.ico: ${SIZES.join('/')}px, ${Buffer.concat([dir, ...images]).length} bytes`);
