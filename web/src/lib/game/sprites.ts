import type { BuildKind, CreepKind, MapDoc } from './types';

export type SpriteColors = {
	ac: string;
	how: string;
	sky: string;
	inferno: string;
	arc: string;
	pulse: string;
	helios: string;
	swarm: string;
	siege: string;
	runner: string;
	lorry: string;
	bulwark: string;
	wasp: string;
	colossus: string;
	mite: string;
	medic: string;
	shade: string;
	flicker: string;
	wall: string;
	wallEdge: string;
	core: string;
};

export const SPRITE: SpriteColors = {
	ac: '#8fa6bc',
	how: '#e0aa5c',
	sky: '#86cbe4',
	inferno: '#ef7a44',
	arc: '#7ad8cc',
	pulse: '#b98ce8',
	helios: '#ffd97a',
	swarm: '#96ccf2',
	siege: '#cf8464',
	runner: '#f0954e',
	lorry: '#c8574c',
	bulwark: '#8f6a9c',
	wasp: '#f2cf6a',
	colossus: '#d0567f',
	mite: '#b6d472',
	medic: '#74d8b0',
	shade: '#5d6a8c',
	flicker: '#6fc9ea',
	wall: '#c9bda6',
	wallEdge: '#7d7364',
	core: '#ffc46e'
};

export const BUILD_BY_ID: BuildKind[] = [
	'inspect',
	'barricade',
	'autocannon',
	'howitzer',
	'skystinger',
	'inferno',
	'arcLance',
	'pulseArray',
	'helios',
	'swarmRack',
	'siegeRail'
];

type Ctx = CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D;

/** Derived shades are constants re-derived thousands of times a frame; cache them. */
const mixCache = new Map<string, string>();

function hexRgb(hex: string): [number, number, number] {
	const h = hex[0] === '#' ? hex.slice(1) : hex;
	const n = parseInt(h.length === 3 ? h.replace(/./g, (c) => c + c) : h, 16);
	return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function mixHex(a: string, b: string, t: number): string {
	const key = `${a}|${b}|${t}`;
	const hit = mixCache.get(key);
	if (hit !== undefined) return hit;
	const [ar, ag, ab] = hexRgb(a);
	const [br, bg, bb] = hexRgb(b);
	const to = (v: number) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0');
	const out = `#${to(ar + (br - ar) * t)}${to(ag + (bg - ag) * t)}${to(ab + (bb - ab) * t)}`;
	mixCache.set(key, out);
	return out;
}

function dark(hex: string, t = 0.42): string {
	return mixHex(hex, '#0a0806', t);
}

function lit(hex: string, t = 0.32): string {
	return mixHex(hex, '#fff6d4', t);
}

function poly(ctx: Ctx, pts: [number, number][], fill: string, stroke?: string, lw = 0.05) {
	ctx.beginPath();
	ctx.moveTo(pts[0][0], pts[0][1]);
	for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1]);
	ctx.closePath();
	ctx.fillStyle = fill;
	ctx.fill();
	if (stroke) {
		ctx.strokeStyle = stroke;
		ctx.lineWidth = lw;
		ctx.stroke();
	}
}

function circle(ctx: Ctx, x: number, y: number, r: number, fill: string, stroke?: string, lw = 0.045) {
	ctx.beginPath();
	ctx.arc(x, y, r, 0, Math.PI * 2);
	ctx.fillStyle = fill;
	ctx.fill();
	if (stroke) {
		ctx.strokeStyle = stroke;
		ctx.lineWidth = lw;
		ctx.stroke();
	}
}

function rect(
	ctx: Ctx,
	x: number,
	y: number,
	w: number,
	h: number,
	fill: string,
	stroke?: string,
	lw = 0.045
) {
	ctx.fillStyle = fill;
	ctx.fillRect(x, y, w, h);
	if (stroke) {
		ctx.strokeStyle = stroke;
		ctx.lineWidth = lw;
		ctx.strokeRect(x, y, w, h);
	}
}

function bolt(ctx: Ctx, x: number, y: number) {
	circle(ctx, x, y, 0.055, '#2a261c', '#d8cba8', 0.025);
	circle(ctx, x - 0.012, y - 0.014, 0.02, 'rgba(255,248,220,0.5)');
}

function padHex(ctx: Ctx, r: number, fill: string, stroke = '#1a1610') {
	const pts: [number, number][] = [];
	for (let i = 0; i < 6; i++) {
		const a = (i * Math.PI) / 3 - Math.PI / 6;
		pts.push([Math.cos(a) * r, Math.sin(a) * r]);
	}
	poly(ctx, pts, fill, stroke, 0.045);
}

function shadowBlob(ctx: Ctx, x = 0.04, y = 0.16, rx = 0.4, ry = 0.16) {
	ctx.beginPath();
	ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
	ctx.fillStyle = 'rgba(0,0,0,0.4)';
	ctx.fill();
}

function metalBarrel(ctx: Ctx, x: number, y: number, len: number, thick: number, col = '#d8d2c4') {
	const d = dark(col, 0.48);
	const l = lit(col, 0.42);
	rect(ctx, x, y - thick / 2, len, thick, d, '#14110c', 0.04);
	rect(ctx, x, y - thick / 2 + thick * 0.12, len, thick * 0.5, col);
	rect(ctx, x + 0.02, y - thick / 2 + thick * 0.16, Math.max(0.05, len - 0.1), thick * 0.22, l);
	rect(ctx, x + len * 0.18, y - thick / 2, 0.045, thick, '#3a3428');
	rect(ctx, x + len * 0.5, y - thick / 2, 0.04, thick, '#3a3428');
	rect(ctx, x + len - 0.11, y - thick * 0.48, 0.11, thick * 0.96, '#1c1a16');
	circle(ctx, x + len - 0.02, y, thick * 0.28, '#0c0a08');
}

function sandbag(ctx: Ctx, x: number, y: number, w: number, h: number, col: string) {
	rect(ctx, x + 0.02, y + 0.05, w, h, '#2a2418');
	rect(ctx, x, y, w, h * 0.86, col, '#4a3c28', 0.028);
	rect(ctx, x + 0.05, y + 0.025, w * 0.72, 0.04, 'rgba(255,240,200,0.3)');
	rect(ctx, x + w * 0.4, y + 0.06, 0.035, h * 0.62, 'rgba(70,54,32,0.4)');
	circle(ctx, x + 0.07, y + h * 0.4, 0.03, dark(col, 0.35));
	circle(ctx, x + w - 0.07, y + h * 0.4, 0.03, dark(col, 0.35));
}

function oval(ctx: Ctx, x: number, y: number, rx: number, ry: number, fill: string, stroke?: string, lw = 0.04) {
	ctx.beginPath();
	ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
	ctx.fillStyle = fill;
	ctx.fill();
	if (stroke) {
		ctx.strokeStyle = stroke;
		ctx.lineWidth = lw;
		ctx.stroke();
	}
}

function bevelPlate(ctx: Ctx, x: number, y: number, w: number, h: number, col: string) {
	rect(ctx, x + 0.03, y + 0.05, w, h, 'rgba(0,0,0,0.32)');
	rect(ctx, x, y, w, h, dark(col, 0.34), '#14110c', 0.03);
	rect(ctx, x + 0.03, y + 0.02, w - 0.06, Math.max(0.035, h * 0.22), lit(col, 0.28));
	rect(ctx, x + w - 0.045, y + h * 0.18, 0.022, h * 0.64, dark(col, 0.52));
	rect(ctx, x + 0.04, y + h - 0.045, w - 0.1, 0.025, dark(col, 0.45));
}

function wheel(ctx: Ctx, x: number, y: number, r: number) {
	circle(ctx, x + r * 0.16, y + r * 0.18, r, 'rgba(0,0,0,0.35)');
	circle(ctx, x, y, r, '#16120e', '#4a4034', 0.03);
	circle(ctx, x, y, r * 0.58, '#3a342c');
	circle(ctx, x - r * 0.18, y - r * 0.2, r * 0.16, 'rgba(255,236,200,0.28)');
	circle(ctx, x, y, r * 0.16, '#1a1610');
}

function tread(ctx: Ctx, x: number, y: number, w: number, h: number) {
	rect(ctx, x, y, w, h, '#1a1610', '#4a4034', 0.02);
	const n = 5;
	for (let i = 0; i < n; i++) {
		rect(ctx, x + 0.015, y + 0.025 + (i * h) / n, w - 0.03, (h / n) * 0.42, '#5a5044');
	}
}

function visor(ctx: Ctx, x: number, y: number, w: number, h: number, glass = '#6ad0e0') {
	rect(ctx, x, y, w, h, '#152028', '#0c1014', 0.02);
	rect(ctx, x + 0.02, y + 0.012, w * 0.62, h * 0.42, glass);
	rect(ctx, x + w * 0.1, y + 0.018, w * 0.2, h * 0.22, 'rgba(255,255,255,0.5)');
}

function rocket(ctx: Ctx, x: number, y: number, len: number, thick: number, body = '#5a7380', nose = '#d8eef4') {
	rect(ctx, x, y - thick / 2, len * 0.72, thick, dark(body, 0.12), '#1a2830', 0.02);
	rect(ctx, x + 0.02, y - thick / 2 + 0.015, len * 0.5, thick * 0.28, lit(body, 0.22));
	rect(ctx, x + len * 0.2, y - thick / 2, 0.035, thick, '#e8b84a');
	poly(
		ctx,
		[
			[x + len * 0.72, y - thick / 2],
			[x + len, y],
			[x + len * 0.72, y + thick / 2]
		],
		nose,
		'#1a2830',
		0.02
	);
	poly(ctx, [[x + 0.03, y - thick / 2], [x - 0.05, y - thick * 0.95], [x + 0.12, y - thick / 2]], dark(body, 0.35));
	poly(ctx, [[x + 0.03, y + thick / 2], [x - 0.05, y + thick * 0.95], [x + 0.12, y + thick / 2]], dark(body, 0.35));
}

function fatCannon(ctx: Ctx, x: number, y: number, len: number, thick: number, col: string) {
	poly(
		ctx,
		[
			[x, y - thick * 0.38],
			[x + len * 0.82, y - thick * 0.52],
			[x + len, y - thick * 0.32],
			[x + len, y + thick * 0.32],
			[x + len * 0.82, y + thick * 0.52],
			[x, y + thick * 0.38]
		],
		dark(col, 0.22),
		'#14110c',
		0.03
	);
	poly(
		ctx,
		[
			[x + 0.02, y - thick * 0.24],
			[x + len * 0.78, y - thick * 0.36],
			[x + len * 0.78, y - thick * 0.08],
			[x + 0.02, y - thick * 0.06]
		],
		lit(col, 0.32)
	);
	rect(ctx, x + len * 0.22, y - thick * 0.5, 0.045, thick, '#4a3c24');
	rect(ctx, x + len * 0.48, y - thick * 0.52, 0.04, thick * 1.04, '#4a3c24');
	rect(ctx, x + len - 0.1, y - thick * 0.62, 0.1, thick * 1.24, '#1c1810');
	rect(ctx, x + len - 0.06, y - thick * 0.42, 0.035, thick * 0.84, '#6a5a38');
	circle(ctx, x + len - 0.02, y, thick * 0.2, '#0c0a08');
}

function caution(ctx: Ctx, x: number, y: number, w: number, h: number) {
	rect(ctx, x, y, w, h, '#1a1410');
	rect(ctx, x + w * 0.12, y, w * 0.22, h, '#e8b84a');
	rect(ctx, x + w * 0.52, y, w * 0.22, h, '#e8b84a');
}

function hatch(ctx: Ctx, x: number, y: number, r: number, col: string) {
	circle(ctx, x, y, r, dark(col, 0.15), '#14110c', 0.03);
	circle(ctx, x - r * 0.15, y - r * 0.2, r * 0.4, lit(col, 0.22));
	rect(ctx, x - r * 0.12, y - r * 0.85, r * 0.24, r * 0.45, '#2a2418');
}

export function drawBarricade(ctx: Ctx, colors: SpriteColors = SPRITE) {
	shadowBlob(ctx, 0.02, 0.2, 0.42, 0.15);
	rect(ctx, -0.06, -0.42, 0.08, 0.55, '#6a5a38', '#2a2418', 0.03);
	rect(ctx, -0.04, -0.4, 0.03, 0.5, lit('#6a5a38', 0.2));
	sandbag(ctx, -0.4, 0.1, 0.4, 0.24, colors.wall);
	sandbag(ctx, -0.02, 0.12, 0.4, 0.24, dark(colors.wall, 0.08));
	sandbag(ctx, -0.24, -0.08, 0.42, 0.24, lit(colors.wall, 0.08));
	sandbag(ctx, -0.42, -0.26, 0.4, 0.24, colors.wall);
	sandbag(ctx, 0.0, -0.24, 0.4, 0.24, lit(colors.wall, 0.12));
	rect(ctx, -0.12, -0.36, 0.22, 0.07, '#5a4a30', '#2a2418', 0.02);
	caution(ctx, -0.1, -0.34, 0.18, 0.04);
}

function turretPad(ctx: Ctx, kind: BuildKind, col: string) {
	shadowBlob(ctx, 0.05, 0.2, 0.46, 0.18);
	if (kind === 'pulseArray') {
		rect(ctx, -0.44, -0.44, 0.88, 0.88, '#16130e', '#c9a24a', 0.055);
		rect(ctx, -0.36, -0.36, 0.72, 0.72, dark(col, 0.2), '#1a1610', 0.04);
		rect(ctx, -0.3, -0.3, 0.6, 0.1, lit(col, 0.2));
		caution(ctx, -0.28, 0.26, 0.56, 0.07);
	} else if (kind === 'skystinger' || kind === 'swarmRack') {
		poly(ctx, [[0, -0.48], [0.46, 0.38], [-0.46, 0.38]], '#16130e', '#c9a24a', 0.05);
		poly(ctx, [[0, -0.4], [0.38, 0.32], [-0.38, 0.32]], col, '#1a1610', 0.04);
		poly(ctx, [[0, -0.28], [0.16, 0.04], [-0.16, 0.04]], lit(col, 0.18));
	} else if (kind === 'howitzer' || kind === 'siegeRail') {
		circle(ctx, 0.02, 0.05, 0.46, '#16130e', '#c9a24a', 0.05);
		circle(ctx, 0, 0.02, 0.38, dark(col, 0.15), '#1a1610', 0.04);
		circle(ctx, -0.08, -0.08, 0.16, lit(col, 0.2));
		sandbag(ctx, -0.4, 0.2, 0.3, 0.15, '#c9b892');
		sandbag(ctx, 0.1, 0.22, 0.3, 0.15, '#c9b892');
	} else {
		padHex(ctx, 0.46, '#16130e', '#c9a24a');
		padHex(ctx, 0.36, dark(col, 0.08));
		padHex(ctx, 0.24, lit(col, 0.14), dark(col, 0.32));
		circle(ctx, 0, 0.02, 0.1, dark(col, 0.35), '#1a1610', 0.025);
	}
	bolt(ctx, -0.26, -0.18);
	bolt(ctx, 0.26, -0.16);
	bolt(ctx, 0.22, 0.26);
	bolt(ctx, -0.24, 0.24);
}

function gunColor(kind: BuildKind, airFocus: boolean, colors: SpriteColors) {
	if (kind === 'howitzer') return colors.how;
	if (kind === 'skystinger' || kind === 'swarmRack') return colors.sky;
	if (kind === 'inferno') return colors.inferno;
	if (kind === 'arcLance') return colors.arc;
	if (kind === 'pulseArray') return colors.pulse;
	if (kind === 'helios') return airFocus ? colors.sky : colors.helios;
	if (kind === 'siegeRail') return colors.siege;
	return colors.ac;
}

export function drawTurret(
	ctx: Ctx,
	kind: BuildKind,
	opts: { aim?: number; tier?: number; airFocus?: boolean; colors?: SpriteColors } = {}
) {
	const colors = opts.colors ?? SPRITE;
	const aim = opts.aim ?? 0;
	const tier = opts.tier ?? 0;
	const col = gunColor(kind, !!opts.airFocus, colors);

	if (kind === 'barricade' || kind === 'inspect') {
		if (kind === 'barricade') drawBarricade(ctx, colors);
		else drawInspect(ctx);
		return;
	}

	turretPad(ctx, kind, col);
	ctx.save();
	ctx.rotate(aim);

	if (kind === 'autocannon') {
		bevelPlate(ctx, -0.26, -0.24, 0.44, 0.48, '#4a5860');
		circle(ctx, 0.04, 0, 0.22, '#3a4850', '#14110c', 0.04);
		circle(ctx, 0.02, -0.03, 0.16, '#5a6870');
		circle(ctx, -0.04, -0.08, 0.07, lit('#7a96a8', 0.28));
		hatch(ctx, -0.04, -0.2, 0.08, '#6a7880');
		visor(ctx, -0.04, -0.07, 0.24, 0.11);
		rect(ctx, 0.08, -0.22, 0.12, 0.44, '#2a3438');
		metalBarrel(ctx, 0.14, -0.13, 0.52, 0.15);
		metalBarrel(ctx, 0.14, 0.13, 0.52, 0.15);
		rect(ctx, 0.18, -0.04, 0.1, 0.08, '#2a3438');
		bevelPlate(ctx, -0.36, -0.16, 0.2, 0.32, '#6a5040');
		rect(ctx, -0.32, -0.09, 0.12, 0.05, '#c9a070');
		rect(ctx, -0.32, 0.04, 0.12, 0.05, '#c9a070');
		rect(ctx, -0.24, -0.32, 0.04, 0.14, '#c9a24a');
		circle(ctx, -0.22, -0.34, 0.035, '#e8b84a');
	} else if (kind === 'howitzer') {
		bevelPlate(ctx, -0.24, -0.22, 0.46, 0.44, '#5a4020');
		circle(ctx, 0.06, 0, 0.18, '#3a2a14', '#e0c888', 0.04);
		circle(ctx, 0.02, -0.04, 0.07, '#f0d890');
		fatCannon(ctx, 0.14, 0, 0.54, 0.28, '#e0c888');
		rect(ctx, 0.2, 0.14, 0.32, 0.09, '#3a3220', '#1a1610', 0.02);
		rect(ctx, 0.22, 0.16, 0.28, 0.035, '#c9a060');
		bolt(ctx, -0.08, -0.12);
		bolt(ctx, -0.08, 0.12);
	} else if (kind === 'skystinger') {
		oval(ctx, -0.1, 0, 0.2, 0.2, '#b8d4e0', '#3a5460', 0.035);
		oval(ctx, -0.12, -0.04, 0.1, 0.1, '#eef6fa');
		circle(ctx, -0.1, 0, 0.045, '#1a3038');
		circle(ctx, -0.1, 0, 0.02, '#7ad0e8');
		rect(ctx, -0.14, 0.16, 0.08, 0.12, '#4a6878');
		bevelPlate(ctx, 0.0, -0.24, 0.36, 0.18, '#4a6878');
		bevelPlate(ctx, 0.0, 0.06, 0.36, 0.18, '#4a6878');
		rocket(ctx, 0.04, -0.15, 0.5, 0.11);
		rocket(ctx, 0.04, 0.15, 0.5, 0.11);
		rect(ctx, -0.22, -0.32, 0.04, 0.2, '#c9a24a');
		circle(ctx, -0.2, -0.34, 0.03, '#e8b84a');
	} else if (kind === 'inferno') {
		circle(ctx, -0.14, -0.2, 0.13, dark('#8a3a20', 0.1), '#3a1810', 0.03);
		circle(ctx, -0.14, 0.2, 0.13, dark('#8a3a20', 0.1), '#3a1810', 0.03);
		circle(ctx, -0.18, -0.24, 0.045, '#f0a070');
		circle(ctx, -0.18, 0.16, 0.045, '#f0a070');
		rect(ctx, -0.16, -0.08, 0.06, 0.16, '#3a1810');
		bevelPlate(ctx, -0.12, -0.16, 0.3, 0.32, '#5a2a18');
		caution(ctx, -0.08, 0.08, 0.22, 0.05);
		poly(ctx, [[0.16, -0.18], [0.54, -0.09], [0.54, 0.09], [0.16, 0.18]], '#f0a060', '#5a2810', 0.03);
		poly(ctx, [[0.24, -0.09], [0.48, -0.035], [0.48, 0.035], [0.24, 0.09]], '#ffe0a0');
		rect(ctx, 0.52, -0.11, 0.09, 0.22, '#2a1810');
		circle(ctx, 0.62, 0, 0.06, '#ff9040');
		circle(ctx, 0.62, 0, 0.03, '#ffe8a0');
		ctx.strokeStyle = '#8a4a28';
		ctx.lineWidth = 0.04;
		ctx.beginPath();
		ctx.arc(-0.02, 0.22, 0.16, 0.2, 2.6);
		ctx.stroke();
	} else if (kind === 'arcLance') {
		bevelPlate(ctx, -0.22, -0.14, 0.28, 0.28, '#2a4844');
		rect(ctx, -0.08, -0.04, 0.16, 0.08, '#1a3030');
		for (let i = 0; i < 5; i++) {
			const x = 0.04 + i * 0.085;
			const rr = 0.1 - i * 0.01;
			circle(ctx, x, 0, rr, i % 2 ? '#d8fff8' : '#b8fff4', '#1a4840', 0.02);
			circle(ctx, x - 0.02, -0.025, rr * 0.28, '#f4fffc');
		}
		metalBarrel(ctx, 0.18, 0, 0.42, 0.09, '#d8fff8');
		circle(ctx, 0.62, 0, 0.055, '#e8fff8');
		circle(ctx, 0.62, 0, 0.025, '#fff');
		rect(ctx, 0.16, -0.16, 0.04, 0.32, '#3a6860');
		rect(ctx, 0.32, -0.14, 0.04, 0.28, '#3a6860');
	} else if (kind === 'pulseArray') {
		bevelPlate(ctx, -0.18, -0.18, 0.36, 0.36, '#2a1830');
		circle(ctx, 0, 0, 0.26, dark(col, 0.1), '#1a1018', 0.03);
		circle(ctx, 0, 0, 0.18, col);
		circle(ctx, 0, 0, 0.1, '#f0d8ff');
		circle(ctx, -0.04, -0.05, 0.035, '#fff');
		for (let i = -1; i <= 1; i++) {
			for (let j = -1; j <= 1; j++) {
				if (i === 0 && j === 0) continue;
				circle(ctx, i * 0.12, j * 0.12, 0.022, '#e0b8f8');
			}
		}
		ctx.strokeStyle = 'rgba(240, 210, 255, 0.9)';
		ctx.lineWidth = 0.035;
		ctx.beginPath();
		ctx.arc(0, 0, 0.32, -0.75, 0.75);
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(0, 0, 0.4, -0.5, 0.5);
		ctx.stroke();
	} else if (kind === 'helios') {
		circle(ctx, 0, 0, 0.22, '#2a2610', '#5a4818', 0.035);
		circle(ctx, 0, 0, 0.16, opts.airFocus ? colors.sky : '#fff4a8');
		circle(ctx, 0, 0, 0.1, opts.airFocus ? '#d8f4ff' : '#fffce8');
		circle(ctx, -0.045, -0.045, 0.05, 'rgba(255,255,255,0.75)');
		for (let i = 0; i < 8; i++) {
			const a = (i * Math.PI) / 4;
			rect(ctx, Math.cos(a) * 0.22 - 0.025, Math.sin(a) * 0.22 - 0.025, 0.05, 0.05, '#c9a24a');
		}
		metalBarrel(ctx, 0.14, 0, 0.42, 0.11, opts.airFocus ? '#c8e8f4' : '#f8e890');
		rect(ctx, -0.04, -0.28, 0.08, 0.08, '#5a4818');
	} else if (kind === 'swarmRack') {
		bevelPlate(ctx, -0.2, -0.26, 0.58, 0.52, '#3a5060');
		caution(ctx, -0.16, 0.18, 0.5, 0.06);
		for (let row = 0; row < 3; row++) {
			for (let colI = 0; colI < 2; colI++) {
				const mx = -0.04 + colI * 0.22;
				const my = -0.16 + row * 0.14;
				rocket(ctx, mx, my, 0.34, 0.09);
			}
		}
	} else if (kind === 'siegeRail') {
		bevelPlate(ctx, -0.24, -0.16, 0.36, 0.32, '#4a3028');
		caution(ctx, -0.2, 0.08, 0.28, 0.05);
		rect(ctx, 0.08, -0.07, 0.64, 0.14, '#2a1810', '#3a2418', 0.025);
		rect(ctx, 0.1, -0.035, 0.58, 0.05, '#fff0d0');
		rect(ctx, 0.1, -0.012, 0.58, 0.012, '#ffc070');
		rect(ctx, 0.2, -0.16, 0.08, 0.32, '#2a1810');
		rect(ctx, 0.42, -0.16, 0.08, 0.32, '#2a1810');
		circle(ctx, 0.24, -0.16, 0.05, '#c07050', '#2a1810', 0.02);
		circle(ctx, 0.46, -0.16, 0.05, '#c07050', '#2a1810', 0.02);
		rect(ctx, 0.66, -0.1, 0.1, 0.2, '#1a1010');
		circle(ctx, -0.04, 0, 0.09, '#c07050', '#2a1810', 0.02);
		circle(ctx, -0.06, -0.03, 0.03, '#f0b090');
	}

	ctx.restore();
	if (tier > 0) {
		for (let i = 0; i < tier; i++) {
			poly(
				ctx,
				[
					[-0.1 + i * 0.13, 0.4],
					[-0.02 + i * 0.13, 0.3],
					[0.06 + i * 0.13, 0.4]
				],
				'#f0e8b0',
				'#6a5a20',
				0.02
			);
		}
	}
}

export function drawInspect(ctx: Ctx) {
	shadowBlob(ctx, 0.02, 0.14, 0.3, 0.1);
	circle(ctx, 0, 0, 0.3, 'rgba(232,184,74,0.1)', '#e8b84a', 0.04);
	oval(ctx, -0.12, 0, 0.14, 0.16, '#2a2418', '#c9a24a', 0.035);
	oval(ctx, 0.12, 0, 0.14, 0.16, '#2a2418', '#c9a24a', 0.035);
	oval(ctx, -0.12, 0, 0.09, 0.1, '#1a2830');
	oval(ctx, 0.12, 0, 0.09, 0.1, '#1a2830');
	circle(ctx, -0.12, 0, 0.055, '#e8b84a');
	circle(ctx, 0.12, 0, 0.055, '#e8b84a');
	circle(ctx, -0.14, -0.03, 0.02, '#fff6c8');
	circle(ctx, 0.1, -0.03, 0.02, '#fff6c8');
	rect(ctx, -0.05, -0.05, 0.1, 0.1, '#c9a24a');
	ctx.strokeStyle = '#e8b84a';
	ctx.lineWidth = 0.045;
	ctx.beginPath();
	ctx.moveTo(-0.4, 0);
	ctx.lineTo(-0.28, 0);
	ctx.moveTo(0.28, 0);
	ctx.lineTo(0.4, 0);
	ctx.moveTo(0, -0.4);
	ctx.lineTo(0, -0.3);
	ctx.moveTo(0, 0.3);
	ctx.lineTo(0, 0.4);
	ctx.stroke();
	rect(ctx, -0.07, -0.44, 0.14, 0.09, '#c9a24a', '#3a3010', 0.02);
	rect(ctx, -0.03, -0.42, 0.06, 0.04, '#fff4c0');
}

export function drawStrike(ctx: Ctx, id: number) {
	shadowBlob(ctx, 0.02, 0.16, 0.3, 0.1);
	if (id === 1) {
		poly(ctx, [[-0.22, -0.32], [0.22, -0.32], [0.3, 0.26], [-0.3, 0.26]], '#4a3428', '#2a1810', 0.04);
		poly(ctx, [[-0.18, -0.28], [0.18, -0.28], [0.24, 0.1], [-0.24, 0.1]], '#7a5a3c');
		rect(ctx, -0.16, -0.2, 0.32, 0.02, 'rgba(0,0,0,0.25)');
		rect(ctx, -0.16, -0.08, 0.32, 0.02, 'rgba(0,0,0,0.25)');
		rect(ctx, -0.24, -0.04, 0.48, 0.09, '#e8b84a');
		rect(ctx, -0.2, -0.02, 0.4, 0.04, '#fff4c0');
		rect(ctx, -0.06, -0.4, 0.12, 0.16, '#c9a24a', '#3a3010', 0.02);
		circle(ctx, 0, 0.14, 0.07, '#1a1010');
		circle(ctx, 0, 0.14, 0.03, '#e8b84a');
	} else if (id === 2) {
		circle(ctx, 0, 0, 0.34, '#1a2838', '#7ad0e8', 0.045);
		circle(ctx, 0, 0, 0.22, '#2a4860');
		circle(ctx, 0, 0, 0.13, '#7ad0e8');
		circle(ctx, -0.04, -0.05, 0.045, '#e8fbff');
		ctx.strokeStyle = '#c8f0ff';
		ctx.lineWidth = 0.035;
		ctx.beginPath();
		ctx.arc(0, 0, 0.28, 0, Math.PI * 2);
		ctx.stroke();
		ctx.lineWidth = 0.04;
		for (let i = 0; i < 6; i++) {
			const a = (i * Math.PI) / 3 + 0.3;
			ctx.beginPath();
			ctx.moveTo(Math.cos(a) * 0.16, Math.sin(a) * 0.16);
			ctx.lineTo(Math.cos(a) * 0.42, Math.sin(a) * 0.42);
			ctx.stroke();
		}
	} else {
		circle(ctx, 0, 0, 0.32, 'rgba(232,184,74,0.12)', '#e8b84a', 0.045);
		ctx.strokeStyle = '#e8b84a';
		ctx.lineWidth = 0.035;
		ctx.beginPath();
		ctx.arc(0, 0, 0.22, 0, Math.PI * 2);
		ctx.stroke();
		ctx.lineWidth = 0.04;
		ctx.beginPath();
		ctx.moveTo(-0.38, 0);
		ctx.lineTo(0.38, 0);
		ctx.moveTo(0, -0.38);
		ctx.lineTo(0, 0.38);
		ctx.stroke();
		poly(ctx, [[0, -0.18], [0.14, 0], [0, 0.18], [-0.14, 0]], '#e8b84a', '#3a3010', 0.025);
		circle(ctx, 0, 0, 0.05, '#fff4c0');
	}
}

export function drawCreep(
	ctx: Ctx,
	kind: CreepKind,
	opts: { heading?: number; radius?: number; flying?: boolean; colors?: SpriteColors } = {}
) {
	const colors = opts.colors ?? SPRITE;
	const r = (opts.radius ?? 0.28) * 2.05;
	const heading = opts.heading ?? 0;
	const col =
		kind === 'wasp'
			? colors.wasp
			: kind === 'colossus'
				? colors.colossus
				: kind === 'mite'
					? colors.mite
					: kind === 'medic'
						? colors.medic
						: kind === 'shade'
							? colors.shade
							: kind === 'flicker'
								? colors.flicker
								: kind === 'bulwark'
									? colors.bulwark
									: kind === 'lorry'
										? colors.lorry
										: colors.runner;

	ctx.save();
	ctx.rotate(heading);
	if (!opts.flying) {
		ctx.beginPath();
		ctx.ellipse(0.04, r * 0.62, r * 0.95, r * 0.3, 0, 0, Math.PI * 2);
		ctx.fillStyle = 'rgba(0,0,0,0.32)';
		ctx.fill();
	}

	// A dark backing disc so hostiles separate from noisy ground. One extra fill per
	// creep — deliberately NOT ctx.shadowBlur, which would apply per sub-shape and cost
	// 15-25 shadowed draws per creep per frame.
	ctx.beginPath();
	ctx.ellipse(0, 0, r * 1.18, r * 0.98, 0, 0, Math.PI * 2);
	ctx.fillStyle = 'rgba(10, 8, 16, 0.5)';
	ctx.fill();

	if (kind === 'wasp') {
		poly(
			ctx,
			[
				[-r * 0.15, -r * 1.05],
				[r * 0.35, -r * 0.2],
				[r * 0.2, r * 0.2],
				[-r * 0.15, r * 1.05],
				[-r * 0.7, r * 0.35],
				[-r * 0.7, -r * 0.35]
			],
			'rgba(255,255,220,0.22)'
		);
		poly(
			ctx,
			[[r * 1.5, 0], [r * 0.2, r * 0.42], [-r * 0.55, r * 0.28], [-r * 0.7, 0], [-r * 0.55, -r * 0.28], [r * 0.2, -r * 0.42]],
			dark(col, 0.12),
			'#5a4818',
			0.04
		);
		poly(ctx, [[r * 1.2, 0], [r * 0.35, r * 0.22], [r * 0.35, -r * 0.22]], lit(col, 0.22));
		circle(ctx, r * 0.7, 0, r * 0.22, '#fff3a0');
		circle(ctx, r * 0.64, -r * 0.06, r * 0.08, '#fff');
		rect(ctx, -r * 0.78, -r * 0.12, r * 0.42, r * 0.24, '#2a2410');
		circle(ctx, -r * 0.82, 0, r * 0.1, '#c9a24a');
		poly(ctx, [[-r * 0.2, -r * 0.18], [-r * 1.15, -r * 0.78], [-r * 0.35, -r * 0.02]], 'rgba(255,255,220,0.32)');
		poly(ctx, [[-r * 0.2, r * 0.18], [-r * 1.15, r * 0.78], [-r * 0.35, r * 0.02]], 'rgba(255,255,220,0.32)');
		rect(ctx, r * 0.05, -r * 0.06, r * 0.55, r * 0.04, 'rgba(90,72,24,0.45)');
	} else if (kind === 'colossus') {
		tread(ctx, -r * 1.2, -r * 0.95, r * 0.28, r * 1.9);
		tread(ctx, r * 0.92, -r * 0.95, r * 0.28, r * 1.9);
		rect(ctx, -r * 1.05, -r * 0.82, r * 2.1, r * 1.64, dark(col, 0.22), '#4a2030', 0.05);
		rect(ctx, -r * 0.95, -r * 0.7, r * 1.9, r * 1.4, col);
		rect(ctx, -r * 0.9, -r * 0.66, r * 1.8, r * 0.2, lit(col, 0.16));
		rect(ctx, -r * 0.7, -r * 0.2, r * 0.35, r * 0.08, 'rgba(0,0,0,0.25)');
		rect(ctx, -r * 0.2, -r * 0.2, r * 0.35, r * 0.08, 'rgba(0,0,0,0.25)');
		rect(ctx, -r * 0.28, -r * 0.38, r * 1.05, r * 0.76, '#2a1220');
		circle(ctx, r * 0.42, 0, r * 0.24, '#f0a0c0');
		circle(ctx, r * 0.36, -r * 0.06, r * 0.09, '#ffe0f0');
		rect(ctx, r * 0.55, -r * 0.08, r * 0.55, r * 0.16, '#c9a0b0');
		rect(ctx, -r * 0.82, r * 0.68, r * 0.48, r * 0.28, '#3a1824');
		rect(ctx, r * 0.35, r * 0.68, r * 0.48, r * 0.28, '#3a1824');
		bolt(ctx, -r * 0.7, -r * 0.5);
		bolt(ctx, r * 0.55, -r * 0.5);
	} else if (kind === 'bulwark') {
		tread(ctx, -r * 1.18, -r * 0.78, r * 0.26, r * 1.56);
		tread(ctx, r * 0.92, -r * 0.78, r * 0.26, r * 1.56);
		rect(ctx, -r * 1.02, -r * 0.7, r * 2.04, r * 1.4, dark(col, 0.2), '#3a2018', 0.045);
		rect(ctx, -r * 0.92, -r * 0.58, r * 1.84, r * 1.16, col);
		rect(ctx, -r * 0.86, -r * 0.52, r * 1.72, r * 0.16, lit(col, 0.14));
		caution(ctx, -r * 0.4, r * 0.28, r * 0.8, r * 0.1);
		circle(ctx, r * 0.08, -r * 0.02, r * 0.38, '#5a3028', '#2a1810', 0.03);
		circle(ctx, r * 0.04, -r * 0.08, r * 0.14, lit(col, 0.1));
		rect(ctx, r * 0.16, -r * 0.1, r * 0.78, r * 0.2, '#c9a090', '#2a1810', 0.02);
		circle(ctx, r * 0.12, 0, r * 0.1, '#2a1810');
	} else if (kind === 'lorry') {
		// LONG WITH A NOTCH. Extended body plus a visible gap near x=0 reads as
		// "truck and trailer" at any size — the one shape no other hostile has.
		bevelPlate(ctx, -r * 1.36, -r * 0.6, r * 1.12, r * 1.2, dark(col, 0.18));
		rect(ctx, -r * 1.28, -r * 0.5, r * 0.96, r * 0.22, lit(col, 0.16));
		// The trailer gap.
		rect(ctx, -r * 0.24, -r * 0.2, r * 0.2, r * 0.4, '#1a1420');
		bevelPlate(ctx, -r * 0.04, -r * 0.66, r * 1.3, r * 1.32, col);
		rect(ctx, r * 0.06, -r * 0.56, r * 1.1, r * 0.22, lit(col, 0.18));
		visor(ctx, r * 0.72, -r * 0.34, r * 0.46, r * 0.68);
		rect(ctx, r * 1.28, -r * 0.16, r * 0.1, r * 0.12, '#f5d98a');
		rect(ctx, r * 1.28, r * 0.06, r * 0.1, r * 0.12, '#f5d98a');
		wheel(ctx, -r * 0.95, r * 0.62, r * 0.19);
		wheel(ctx, r * 0.28, r * 0.66, r * 0.19);
		wheel(ctx, r * 0.92, r * 0.64, r * 0.17);
	} else if (kind === 'mite') {
		poly(ctx, [[r * 1.42, 0], [-r * 0.5, r * 0.95], [-r * 0.15, 0], [-r * 0.5, -r * 0.95]], col, '#3a4820', 0.03);
		poly(ctx, [[r * 0.95, 0], [r * 0.2, r * 0.32], [r * 0.2, -r * 0.32]], lit(col, 0.22));
		poly(ctx, [[-r * 0.15, 0], [-r * 0.7, r * 0.35], [-r * 0.55, 0], [-r * 0.7, -r * 0.35]], dark(col, 0.2));
		ctx.strokeStyle = '#3a4820';
		ctx.lineWidth = 0.045;
		ctx.beginPath();
		ctx.moveTo(-r * 0.05, r * 0.22);
		ctx.lineTo(-r * 0.95, r * 0.82);
		ctx.moveTo(-r * 0.05, -r * 0.22);
		ctx.lineTo(-r * 0.95, -r * 0.82);
		ctx.moveTo(r * 0.22, r * 0.18);
		ctx.lineTo(-r * 0.32, r * 0.72);
		ctx.moveTo(r * 0.22, -r * 0.18);
		ctx.lineTo(-r * 0.32, -r * 0.72);
		ctx.moveTo(r * 0.4, r * 0.12);
		ctx.lineTo(r * 0.05, r * 0.7);
		ctx.moveTo(r * 0.4, -r * 0.12);
		ctx.lineTo(r * 0.05, -r * 0.7);
		ctx.stroke();
		circle(ctx, r * 0.55, 0, r * 0.2, '#2a3218');
		circle(ctx, r * 0.6, -r * 0.04, r * 0.07, '#d8f0a0');
		poly(ctx, [[r * 1.15, -r * 0.08], [r * 1.5, -r * 0.18], [r * 1.15, 0]], '#3a4820');
		poly(ctx, [[r * 1.15, 0.08], [r * 1.5, 0.18], [r * 1.15, 0]], '#3a4820');
	} else if (kind === 'medic') {
		// ROUND. No wheels, no box — a soft dome so it cannot be mistaken for the
		// runner (pointed) or the lorry (long, notched) at 18px.
		oval(ctx, 0, 0, r * 1.02, r * 0.88, col, dark(col, 0.45), 0.05);
		oval(ctx, -r * 0.18, -r * 0.22, r * 0.62, r * 0.44, lit(col, 0.2));
		rect(ctx, -r * 0.18, -r * 0.72, r * 0.36, r * 1.44, '#f4f8f4');
		rect(ctx, -r * 0.72, -r * 0.18, r * 1.44, r * 0.36, '#f4f8f4');
		circle(ctx, 0, 0, r * 0.2, '#e86a6a');
		circle(ctx, 0, 0, r * 0.1, '#fff0f0');
		// A soft halo, so the unit you must kill first announces itself.
		ctx.globalAlpha = 0.28;
		circle(ctx, 0, 0, r * 1.24, 'rgba(116, 216, 176, 0.5)');
		ctx.globalAlpha = 1;
	} else if (kind === 'shade') {
		ctx.globalAlpha = 0.42;
		poly(ctx, [[r * 0.9, 0], [-r * 0.7, r * 0.9], [-r * 0.35, 0], [-r * 0.7, -r * 0.9]], 'rgba(180,190,220,0.35)');
		ctx.globalAlpha = 0.62;
		poly(
			ctx,
			[[r * 1.22, 0], [-r * 0.48, r * 0.78], [-r * 0.12, 0], [-r * 0.48, -r * 0.78]],
			col,
			'rgba(210,216,230,0.8)',
			0.045
		);
		poly(ctx, [[r * 0.58, 0], [-r * 0.12, r * 0.32], [-r * 0.12, -r * 0.32]], 'rgba(230,236,255,0.4)');
		circle(ctx, r * 0.32, -r * 0.12, r * 0.1, 'rgba(230,236,255,0.8)');
		circle(ctx, r * 0.32, r * 0.12, r * 0.1, 'rgba(230,236,255,0.8)');
		circle(ctx, r * 0.34, -r * 0.12, r * 0.035, '#fff');
		circle(ctx, r * 0.34, r * 0.12, r * 0.035, '#fff');
		ctx.globalAlpha = 1;
	} else if (kind === 'flicker') {
		poly(
			ctx,
			[
				[r * 1.4, 0],
				[r * 0.25, r * 0.28],
				[-r * 0.18, r * 0.1],
				[-r * 1.12, r * 0.6],
				[-r * 0.4, 0],
				[-r * 1.12, -r * 0.6],
				[-r * 0.18, -r * 0.1],
				[r * 0.25, -r * 0.28]
			],
			col,
			'#2a6070',
			0.035
		);
		poly(ctx, [[r * 0.78, 0], [r * 0.12, r * 0.14], [r * 0.12, -r * 0.14]], lit(col, 0.28));
		circle(ctx, r * 0.22, 0, r * 0.13, '#e8fbff');
		circle(ctx, r * 0.2, -r * 0.03, r * 0.05, '#fff');
		circle(ctx, -r * 0.55, 0, r * 0.08, '#fff4a0');
		rect(ctx, -r * 0.85, -r * 0.04, r * 0.28, r * 0.08, 'rgba(255,244,160,0.45)');
	} else {
		// POINTED (runner). A forward wedge nose and a single wheel pair, so the
		// fastest hostile also reads as the sharpest shape.
		poly(
			ctx,
			[
				[r * 1.32, 0],
				[r * 0.42, r * 0.6],
				[-r * 0.86, r * 0.56],
				[-r * 0.86, -r * 0.56],
				[r * 0.42, -r * 0.6]
			],
			col,
			dark(col, 0.5),
			0.05
		);
		poly(ctx, [[r * 1.18, 0], [r * 0.5, r * 0.3], [r * 0.5, -r * 0.3]], lit(col, 0.26));
		visor(ctx, r * 0.06, -r * 0.28, r * 0.46, r * 0.56);
		rect(ctx, r * 1.0, -r * 0.1, r * 0.1, r * 0.2, '#f5d98a');
		wheel(ctx, -r * 0.42, r * 0.6, r * 0.2);
		rect(ctx, -r * 0.8, -r * 0.62, r * 0.06, r * 0.26, '#f5c96b');
	}

	ctx.restore();
}

export function drawWallCell(
	ctx: Ctx,
	x: number,
	y: number,
	colors: SpriteColors = SPRITE,
	walls?: Set<string>
) {
	const L = !!walls?.has(`${x - 1},${y}`);
	const R = !!walls?.has(`${x + 1},${y}`);
	const U = !!walls?.has(`${x},${y - 1}`);
	const D = !!walls?.has(`${x},${y + 1}`);
	const x0 = L ? x : x + 0.1;
	const x1 = R ? x + 1 : x + 0.9;
	const y0 = U ? y : y + 0.08;
	const y1 = D ? y + 1 : y + 0.9;
	const w = x1 - x0;
	const h = y1 - y0;
	ctx.fillStyle = 'rgba(0,0,0,0.36)';
	ctx.fillRect(x0 + 0.05, y0 + 0.12, w, h);
	ctx.fillStyle = dark(colors.wallEdge, 0.12);
	ctx.fillRect(x0, y0 + 0.12, w, h - 0.1);
	ctx.fillStyle = ((x * 13 + y * 7) & 1) === 0 ? colors.wall : lit(colors.wall, 0.06);
	ctx.fillRect(x0, y0, w, h - 0.16);
	ctx.fillStyle = lit(colors.wall, 0.22);
	ctx.fillRect(x0 + 0.05, y0 + 0.03, w - 0.1, 0.07);
	ctx.fillStyle = 'rgba(70, 54, 32, 0.42)';
	const mid = y0 + h * 0.34;
	ctx.fillRect(x0 + 0.06, mid, w - 0.12, 0.05);
	ctx.fillRect(x0 + 0.06, mid + 0.18, w - 0.12, 0.05);
	ctx.fillRect(x0 + 0.06, mid + 0.36, w - 0.12, 0.05);
	ctx.fillStyle = 'rgba(255, 244, 210, 0.14)';
	ctx.fillRect(x0 + 0.08, y0 + 0.14, 0.05, h - 0.3);
	ctx.fillStyle = '#3a3224';
	const nails = [0.18, 0.5, 0.82];
	for (const t of nails) {
		ctx.fillRect(x0 + w * t - 0.03, mid + 0.01, 0.055, 0.055);
		ctx.fillRect(x0 + w * t - 0.03, mid + 0.19, 0.055, 0.055);
	}
	if (!U) {
		ctx.fillStyle = '#4a4030';
		ctx.fillRect(x0 + 0.12, y0 + 0.08, 0.08, 0.08);
		ctx.fillRect(x1 - 0.22, y0 + 0.08, 0.08, 0.08);
		caution(ctx, x0 + w * 0.22, y0 + 0.02, w * 0.56, 0.055);
	}
	if (!L) {
		ctx.fillStyle = dark(colors.wallEdge, 0.28);
		ctx.fillRect(x0 - 0.05, y0 + 0.06, 0.1, h - 0.14);
		ctx.fillStyle = lit(colors.wallEdge, 0.1);
		ctx.fillRect(x0 - 0.02, y0 + 0.08, 0.03, h - 0.18);
	}
	if (!R) {
		ctx.fillStyle = dark(colors.wallEdge, 0.28);
		ctx.fillRect(x1 - 0.05, y0 + 0.06, 0.1, h - 0.14);
	}
}

export function clusterCells(cells: [number, number][]): { cx: number; cy: number; count: number }[] {
	const set = new Set(cells.map(([x, y]) => `${x},${y}`));
	const seen = new Set<string>();
	const out: { cx: number; cy: number; count: number }[] = [];
	for (const [x, y] of cells) {
		const k = `${x},${y}`;
		if (seen.has(k)) continue;
		const stack = [[x, y]];
		seen.add(k);
		let sx = 0;
		let sy = 0;
		let n = 0;
		while (stack.length) {
			const [cx, cy] = stack.pop()!;
			sx += cx;
			sy += cy;
			n += 1;
			for (const [dx, dy] of [
				[1, 0],
				[-1, 0],
				[0, 1],
				[0, -1]
			] as const) {
				const nx = cx + dx;
				const ny = cy + dy;
				const nk = `${nx},${ny}`;
				if (set.has(nk) && !seen.has(nk)) {
					seen.add(nk);
					stack.push([nx, ny]);
				}
			}
		}
		out.push({ cx: sx / n + 0.5, cy: sy / n + 0.5, count: n });
	}
	return out;
}

export function drawRelay(
	ctx: Ctx,
	cx: number,
	cy: number,
	opts: { frac?: number; time?: number; rings?: number; colors?: SpriteColors; scale?: number } = {}
) {
	const colors = opts.colors ?? SPRITE;
	const frac = opts.frac ?? 1;
	const time = opts.time ?? 0;
	const rings = opts.rings ?? 2;
	const s = opts.scale ?? 1;
	ctx.save();
	ctx.translate(cx, cy);
	ctx.scale(s, s);
	// A small halo at the lantern itself rather than a pool washing the whole tile —
	// the relay should look lit, not floodlit.
	const lamp = 0.5 + 0.5 * frac;
	for (let i = 3; i >= 1; i--) {
		ctx.beginPath();
		ctx.arc(0, -0.52, 0.13 + i * 0.11, 0, Math.PI * 2);
		ctx.fillStyle = `rgba(255, 196, 110, ${0.05 * lamp})`;
		ctx.fill();
	}
	for (let i = rings; i >= 1; i--) {
		ctx.beginPath();
		ctx.arc(0, 0.06, 0.52 + i * 0.26 + 0.03 * Math.sin(time * 2 + i), 0, Math.PI * 2);
		ctx.strokeStyle = `rgba(255, 196, 110, ${(0.1 / i) * lamp})`;
		ctx.lineWidth = 0.03;
		ctx.stroke();
	}
	ctx.beginPath();
	ctx.ellipse(0.04, 0.44, 0.6, 0.2, 0, 0, Math.PI * 2);
	ctx.fillStyle = 'rgba(0,0,0,0.42)';
	ctx.fill();
	sandbag(ctx, -0.58, 0.2, 0.36, 0.2, '#b9ae9a');
	sandbag(ctx, -0.18, 0.26, 0.36, 0.2, '#a89c86');
	sandbag(ctx, 0.22, 0.2, 0.36, 0.2, '#b9ae9a');
	bevelPlate(ctx, -0.5, -0.16, 1.0, 0.62, '#4a4434');
	caution(ctx, -0.34, 0.26, 0.68, 0.08);
	const roof = frac > 0.35 ? colors.core : '#8a6a28';
	poly(ctx, [[-0.62, -0.12], [0, -0.7], [0.62, -0.12]], dark(roof, 0.32), '#2a2410', 0.05);
	poly(ctx, [[-0.48, -0.14], [0, -0.58], [0.48, -0.14]], roof);
	rect(ctx, -0.02, -0.62, 0.04, 0.46, dark(roof, 0.4));
	rect(ctx, -0.5, -0.2, 1.0, 0.1, '#2a2418');
	bevelPlate(ctx, -0.2, -0.62, 0.4, 0.28, '#3a3428');
	rect(ctx, -0.14, -0.56, 0.12, 0.12, '#ffc46e');
	rect(ctx, 0.02, -0.56, 0.12, 0.12, '#ffc46e');
	rect(ctx, -0.12, -0.54, 0.07, 0.05, '#fff6dc');
	rect(ctx, 0.04, -0.54, 0.07, 0.05, '#fff6dc');
	poly(ctx, [[-0.22, -0.62], [0, -0.8], [0.22, -0.62]], roof, '#2a2410', 0.035);
	rect(ctx, -0.12, 0.0, 0.24, 0.36, '#14110c');
	rect(ctx, -0.08, 0.04, 0.16, 0.14, '#3a3428');
	circle(ctx, 0.06, 0.18, 0.03, '#e8b84a');
	rect(ctx, -0.34, -0.06, 0.16, 0.14, '#e8b84a');
	rect(ctx, 0.18, -0.06, 0.16, 0.14, '#e8b84a');
	rect(ctx, -0.32, -0.04, 0.1, 0.06, '#fff4c0');
	rect(ctx, 0.2, -0.04, 0.1, 0.06, '#fff4c0');
	rect(ctx, -0.03, -0.98, 0.06, 0.22, colors.core);
	oval(ctx, 0.1, -1.02, 0.13, 0.09, '#e8d080', '#2a2410', 0.03);
	oval(ctx, 0.1, -1.02, 0.05, 0.035, '#1a1610');
	circle(ctx, 0, -1.02, 0.07, colors.core, '#2a2410', 0.03);
	circle(ctx, 0, -1.02, 0.03, frac > 0.2 ? '#fff4c0' : '#5a4818');
	bolt(ctx, -0.42, -0.06);
	bolt(ctx, 0.42, -0.06);
	bolt(ctx, -0.42, 0.16);
	bolt(ctx, 0.42, 0.16);
	ctx.restore();
}

export function hash(x: number, y: number): number {
	let n = Math.imul(x, 374761393) + Math.imul(y, 668265263);
	n = Math.imul(n ^ (n >>> 13), 1274126177);
	return ((n ^ (n >>> 16)) >>> 0) / 4294967296;
}

export const TERRAIN_SUB = 16;

export type GroundCols = {
	soil: string;
	scrub: string;
	scrub2: string;
	sand: string;
	rock: string;
	rockLit: string;
	rockShade: string;
	grass: string;
	dirt: string;
	pebble: string;
	moss: string;
	crater: string;
	bush: string;
	bushLit: string;
	spawn: string;
	spawnLit: string;
};

/** Shared boundary rim for every biome's rock — one silhouette colour across all maps. */
const ROCK_RIM = '#14120f';

const BIOME_SLUGS = [
	'kilo',
	'redoubt',
	'dust',
	'split',
	'enclave',
	'twin',
	'trigate',
	'oxbow',
	'mossfold',
	'labyrinth'
];

/**
 * Ground palettes. Each theater keeps its own identity — meadow, flagstone, ochre
 * desert, slate, teal, brown earth, wetland, forest — so the set does not read as ten
 * variations on one time of day. Only Labyrinth is a true night map, with Twin Cores
 * as the one cool teal; the rest are daylit earth, just deep enough to sit under a
 * near-black UI.
 *
 * Rock stays one warm-neutral grey family across all ten. That is deliberate: "you
 * cannot build here" should be a single learnable colour, and a desaturated grey has
 * strong contrast against both the green and the brown grounds. `rockShade` plus the
 * shared ROCK_RIM give stone a hard edge so it reads as an object at ~25px per tile
 * rather than a stain in the ground texture.
 */
const BIOMES: GroundCols[] = [
	// 0 · Kilo Outpost — open green meadow
	{
		soil: '#35422a', scrub: '#3f5031', scrub2: '#465937', sand: '#60583b',
		rock: '#98958b', rockLit: '#bbb8ac', rockShade: '#24231f',
		grass: '#516d38', dirt: '#4a3f2c', pebble: '#918d82', moss: '#3f5c34',
		crater: '#262c1e', bush: '#22301a', bushLit: '#3c5630',
		spawn: '#6b3228', spawnLit: '#94463a'
	},
	// 1 · Redoubt — flagstone courtyard, grey-brown
	{
		soil: '#3d3a34', scrub: '#47443c', scrub2: '#4f4b42', sand: '#635b4b',
		rock: '#9b978e', rockLit: '#bdbaae', rockShade: '#26241f',
		grass: '#464c35', dirt: '#4a4238', pebble: '#938f84', moss: '#45503a',
		crater: '#2a2823', bush: '#2c3026', bushLit: '#454a38',
		spawn: '#6d342c', spawnLit: '#96483c'
	},
	// 2 · Dust Cut — warm ochre desert
	{
		soil: '#4b3b24', scrub: '#55442a', scrub2: '#5b4a30', sand: '#6c5836',
		rock: '#a29b8e', rockLit: '#c3beb0', rockShade: '#2c2820',
		grass: '#504727', dirt: '#42341f', pebble: '#6c685f', moss: '#4a4527',
		crater: '#302516', bush: '#2b2915', bushLit: '#433f22',
		spawn: '#7d3c26', spawnLit: '#a55036'
	},
	// 3 · Split Relay — cold slate spine
	{
		soil: '#383b3c', scrub: '#424646', scrub2: '#494d4d', sand: '#595a55',
		rock: '#979c9c', rockLit: '#babebd', rockShade: '#232525',
		grass: '#434f3b', dirt: '#423f38', pebble: '#909392', moss: '#3f5148',
		crater: '#262828', bush: '#253029', bushLit: '#3d4d40',
		spawn: '#68322e', spawnLit: '#91463e'
	},
	// 4 · Enclave — sheltered, lush green yard
	{
		soil: '#2f4026', scrub: '#384d2c', scrub2: '#405633', sand: '#5c5636',
		rock: '#959489', rockLit: '#b8b5a8', rockShade: '#20221c',
		grass: '#466431', dirt: '#443c28', pebble: '#8e8b80', moss: '#375c30',
		crater: '#202a1a', bush: '#1c2c16', bushLit: '#35522a',
		spawn: '#67322a', spawnLit: '#90463a'
	},
	// 5 · Twin Cores — cool teal, the one blue-green map
	{
		soil: '#26403f', scrub: '#2e4c4a', scrub2: '#345552', sand: '#455a56',
		rock: '#929c9a', rockLit: '#b5bebb', rockShade: '#1c2423',
		grass: '#2c594d', dirt: '#35423f', pebble: '#8b9391', moss: '#2b6053',
		crater: '#1b2a29', bush: '#1a3230', bushLit: '#2f5a52',
		spawn: '#5f3436', spawnLit: '#884849'
	},
	// 6 · Tri-Gate — neutral brown earth
	{
		soil: '#48392c', scrub: '#544434', scrub2: '#5c4b3a', sand: '#705e42',
		rock: '#9d978c', rockLit: '#bfbaad', rockShade: '#29241e',
		grass: '#555530', dirt: '#4e3e2c', pebble: '#948f84', moss: '#4e5232',
		crater: '#2f2620', bush: '#302a1e', bushLit: '#4c4430',
		spawn: '#723528', spawnLit: '#9b4938'
	},
	// 7 · Oxbow — river bend, wet green banks
	{
		soil: '#2c4436', scrub: '#35523f', scrub2: '#3c5c47', sand: '#515f46',
		rock: '#959c95', rockLit: '#b8beb5', rockShade: '#1e2420',
		grass: '#376446', dirt: '#3c4438', pebble: '#8d938c', moss: '#2f6248',
		crater: '#1e2c24', bush: '#1c3226', bushLit: '#33583e',
		spawn: '#63342f', spawnLit: '#8c4840'
	},
	// 8 · Mossfold — deep forest, the darkest green
	{
		soil: '#2a3a26', scrub: '#32462c', scrub2: '#385032', sand: '#515134',
		rock: '#909689', rockLit: '#b3b9ab', rockShade: '#1d211b',
		grass: '#395e2f', dirt: '#3a3a2a', pebble: '#8a8f84', moss: '#33602e',
		crater: '#1d2619', bush: '#182814', bushLit: '#2e5026',
		spawn: '#613128', spawnLit: '#8a4538'
	},
	// 9 · Labyrinth — the night map, violet under starlight
	{
		soil: '#2e2a40', scrub: '#37324c', scrub2: '#3e3955', sand: '#4e485e',
		rock: '#988fb0', rockLit: '#bbb1cf', rockShade: '#1c1826',
		grass: '#413b60', dirt: '#38324a', pebble: '#918aa2', moss: '#3c4560',
		crater: '#221e30', bush: '#241f38', bushLit: '#3c3558',
		spawn: '#653048', spawnLit: '#8e4458'
	}
];

export function biomeCols(slug?: string, id?: number, seed?: number): GroundCols {
	if (typeof id === 'number' && id >= 0 && id < BIOMES.length) return BIOMES[id];
	if (slug) {
		const i = BIOME_SLUGS.indexOf(slug);
		if (i >= 0) return BIOMES[i];
	}
	return BIOMES[Math.abs(seed ?? 0) % BIOMES.length];
}

export function visualSeed(spec: {
	w: number;
	h: number;
	rocks: [number, number][];
	seed?: number;
	name?: string;
	id?: number;
}): number {
	if (spec.seed) return spec.seed >>> 0;
	let s = (spec.w * 73856093) ^ (spec.h * 19349663) ^ ((spec.id ?? 0) * 83492791);
	for (const [x, y] of spec.rocks) {
		s = Math.imul(s ^ (x * 0x45d9f3b + y * 0x27d4eb2d), 2246822519);
	}
	if (spec.name) {
		for (let i = 0; i < spec.name.length; i++) s = Math.imul(s ^ spec.name.charCodeAt(i), 2654435761);
	}
	return s >>> 0;
}

function parseHex(hex: string): [number, number, number] {
	const h = hex[0] === '#' ? hex.slice(1) : hex;
	const n = parseInt(h.length === 3 ? h.replace(/./g, (c) => c + c) : h, 16);
	return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function shadeHex(hex: string, amt: number): string {
	const [r, g, b] = parseHex(hex);
	const to = (v: number) => Math.max(0, Math.min(255, Math.round(v * amt))).toString(16).padStart(2, '0');
	return `#${to(r)}${to(g)}${to(b)}`;
}

export function groundTexel(subX: number, subY: number, cols: GroundCols, seed = 0): string {
	const sub = TERRAIN_SUB;
	const cellX = Math.floor(subX / sub);
	const cellY = Math.floor(subY / sub);
	const sx = ((subX % sub) + sub) % sub;
	const sy = ((subY % sub) + sub) % sub;
	const u = (sx + 0.5) / sub;
	const v = (sy + 0.5) / sub;
	const mac = hash(Math.floor(subX / 8) + seed, Math.floor(subY / 8));
	const mid = hash(Math.floor(subX / 3) + (seed & 1023), Math.floor(subY / 3));
	const fine = hash(subX + (seed & 255), subY + ((seed >>> 8) & 255));
	const speck = hash(subX + 41, subY + 73 + (seed & 63));

	const craterRoll = hash(cellX * 17 + seed, cellY * 23 + 91);
	if (craterRoll > 0.93) {
		const ccx = 0.35 + hash(cellX, 3) * 0.3;
		const ccy = 0.35 + hash(cellY, 4) * 0.3;
		const dx = u - ccx;
		const dy = v - ccy;
		const rad = 0.18 + hash(cellX, cellY) * 0.14;
		const d2 = dx * dx + dy * dy;
		if (d2 < rad * rad) {
			if (d2 > rad * rad * 0.5) return cols.dirt;
			return speck > 0.55 ? cols.crater : cols.dirt;
		}
	}

	const bush = hash(cellX + seed * 3, cellY + 99);
	if (bush > 0.9) {
		const bx = 0.28 + hash(cellX, 1) * 0.44;
		const by = 0.34 + hash(cellY, 2) * 0.38;
		const ddx = u - bx;
		const ddy = v - by;
		const br = 0.15 + hash(cellX, cellY) * 0.08;
		if (ddx * ddx + ddy * ddy * 1.2 < br * br) {
			if (v < by - 0.015 && fine > 0.35) return cols.bushLit;
			return cols.bush;
		}
	}

	if (fine > 0.82 && mid > 0.4 && mac < 0.64 && speck > 0.45) {
		return cols.grass;
	}

	const rut = hash(Math.floor(subY / 6), seed + 7);
	if (rut > 0.86) {
		const wave = (subX + subY * 0.18) % 28;
		if (wave > 12.1 && wave < 14.05 && fine > 0.22) return cols.dirt;
	}

	if (speck > 0.972) return cols.pebble;
	if (speck < 0.012) return cols.sand;

	const n = mac * 0.16 + mid * 0.34 + fine * 0.5;
	if (n > 0.84) return cols.sand;
	if (n > 0.6) return cols.scrub2;
	if (n > 0.34) return cols.scrub;
	return cols.soil;
}

export function rockTexel(
	cellX: number,
	cellY: number,
	sx: number,
	sy: number,
	rocks: Set<string>,
	cols: GroundCols,
	seed = 0
): string | null {
	const sub = TERRAIN_SUB;
	const u = (sx + 0.5) / sub;
	const v = (sy + 0.5) / sub;
	const h0 = hash(cellX + seed, cellY);
	const h1 = hash(cellX, cellY + seed + 17);
	const h2 = hash(cellX * 3, cellY * 7 + seed);
	const nL = rocks.has(`${cellX - 1},${cellY}`);
	const nR = rocks.has(`${cellX + 1},${cellY}`);
	const nU = rocks.has(`${cellX},${cellY - 1}`);
	const nD = rocks.has(`${cellX},${cellY + 1}`);
	const blobs = [
		{ cx: 0.38 + h0 * 0.16, cy: 0.46 + h1 * 0.1, rx: 0.34 + h0 * 0.1, ry: 0.28 + h1 * 0.08 },
		{ cx: 0.62 + h1 * 0.12, cy: 0.58 + h2 * 0.08, rx: 0.26 + h2 * 0.08, ry: 0.24 + h0 * 0.06 }
	];
	if (h2 > 0.42) blobs.push({ cx: 0.5 + (h0 - 0.5) * 0.22, cy: 0.36 + h1 * 0.08, rx: 0.22, ry: 0.18 });

	const n = hash(cellX * sub + sx + seed, cellY * sub + sy);
	const n2 = hash(cellX * sub + sx * 3 + seed, cellY * sub + sy * 5);
	let hit = false;
	let minD = 9;
	for (const b of blobs) {
		let { cx, cy, rx, ry } = b;
		if (nL) {
			cx -= 0.14;
			rx += 0.2;
		}
		if (nR) {
			cx += 0.14;
			rx += 0.2;
		}
		if (nU) {
			cy -= 0.12;
			ry += 0.18;
		}
		if (nD) {
			cy += 0.12;
			ry += 0.18;
		}
		const d = ((u - cx) / rx) ** 2 + ((v - cy) / ry) ** 2 + (n - 0.5) * 0.2;
		if (d < 1) {
			hit = true;
			minD = Math.min(minD, d);
		}
	}
	if (!hit) {
		if ((nL && u < 0.3) || (nR && u > 0.7) || (nU && v < 0.3) || (nD && v > 0.7)) {
			hit = n < 0.78;
		}
	}
	if (!hit) return null;

	const crack = hash(cellX * 11 + Math.floor(sx / 2), cellY * 13 + seed);
	if (Math.abs(u + v * 0.35 - (0.35 + crack * 0.4)) < 0.022 && n2 > 0.55 && v > 0.18 && v < 0.86)
		return cols.rockShade;
	if (n > 0.97) return cols.moss;
	const light = (0.58 - v) * 0.95 + (0.42 - u) * 0.38 + (n2 - 0.5) * 0.16;
	// A hard near-black rim on the blob boundary. At ~25px per tile this is what makes
	// stone read as a raised object instead of a smudge in the ground texture, and it
	// costs one comparison in a loop that already has minD.
	if (minD > 0.9) return ROCK_RIM;
	if (minD > 0.76) return cols.rockShade;
	if (light > 0.2) return cols.rockLit;
	if (light < -0.1) return cols.rockShade;
	return cols.rock;
}

function rockCover(
	cellX: number,
	cellY: number,
	sx: number,
	sy: number,
	rocks: Set<string>,
	cols: GroundCols,
	seed: number
): boolean {
	const sub = TERRAIN_SUB;
	let cx = cellX;
	let cy = cellY;
	let x = sx;
	let y = sy;
	while (x < 0) {
		cx -= 1;
		x += sub;
	}
	while (x >= sub) {
		cx += 1;
		x -= sub;
	}
	while (y < 0) {
		cy -= 1;
		y += sub;
	}
	while (y >= sub) {
		cy += 1;
		y -= sub;
	}
	if (!rocks.has(`${cx},${cy}`)) return false;
	return rockTexel(cx, cy, x, y, rocks, cols, seed) != null;
}

function spawnTexel(subX: number, subY: number, cols: GroundCols, seed: number): string {
	const fine = hash(subX + 5, subY + seed);
	if (fine > 0.72) return cols.spawnLit;
	if (fine < 0.2) return cols.dirt;
	return cols.spawn;
}

export function terrainHex(
	cellX: number,
	cellY: number,
	sx: number,
	sy: number,
	rocks: Set<string>,
	spawns: Set<string> | undefined,
	cols: GroundCols,
	seed: number
): string {
	if (rocks.has(`${cellX},${cellY}`)) {
		const rock = rockTexel(cellX, cellY, sx, sy, rocks, cols, seed);
		if (rock) return applyCellGrid(rock, sx, sy);
	}
	const shadow =
		rockCover(cellX, cellY, sx - 1, sy - 1, rocks, cols, seed) ||
		rockCover(cellX, cellY, sx, sy - 1, rocks, cols, seed);
	const subX = cellX * TERRAIN_SUB + sx;
	const subY = cellY * TERRAIN_SUB + sy;
	const hex = spawns?.has(`${cellX},${cellY}`)
		? spawnTexel(subX, subY, cols, seed)
		: groundTexel(subX, subY, cols, seed);
	return shadow ? applyCellGrid(shadeHex(hex, 0.62), sx, sy) : applyCellGrid(hex, sx, sy);
}

function applyCellGrid(hex: string, sx: number, sy: number): string {
	const sub = TERRAIN_SUB;
	if (sx === 0 || sy === 0) return shadeHex(hex, 0.22);
	if (sx === 1 || sy === 1) return shadeHex(hex, 0.4);
	if (sx === 2 || sy === 2) return shadeHex(hex, 0.58);
	if (sx >= sub - 1 || sy >= sub - 1) return shadeHex(hex, 0.72);
	if ((sx === 3 && sy > 2 && sy < sub - 1) || (sy === 3 && sx > 2 && sx < sub - 1)) return shadeHex(hex, 1.12);
	return hex;
}

export type TerrainSpec = {
	w: number;
	h: number;
	rocks: [number, number][];
	spawns?: [number, number][];
	seed?: number;
	slug?: string;
	name?: string;
	id?: number;
	cols?: GroundCols;
};

const terrainCache = new Map<string, OffscreenCanvas | HTMLCanvasElement>();

export function paintTerrainBitmap(spec: TerrainSpec, texel = 2): OffscreenCanvas | HTMLCanvasElement {
	const cols = spec.cols ?? biomeCols(spec.slug, spec.id, spec.seed);
	const seed = visualSeed(spec);
	// Content-addressed, not length-addressed: keying on counts alone meant moving a rock
	// in the workshop (same count) returned the previous bitmap and the preview went stale.
	let cellHash = 0;
	for (const [cx, cy] of spec.rocks) cellHash = (cellHash * 31 + cx * 73856093 + cy * 19349663) | 0;
	for (const [cx, cy] of spec.spawns ?? [])
		cellHash = (cellHash * 31 + cx * 83492791 + cy * 22801763) | 0;
	const key = `v7:${seed}:${spec.w}x${spec.h}:${texel}:${spec.slug ?? spec.id ?? ''}:${spec.rocks.length}:${spec.spawns?.length ?? 0}:${cellHash >>> 0}`;
	const hit = terrainCache.get(key);
	if (hit) return hit;
	const sub = TERRAIN_SUB;
	const cell = sub * texel;
	const w = spec.w * cell;
	const h = spec.h * cell;
	const g =
		typeof OffscreenCanvas !== 'undefined'
			? new OffscreenCanvas(w, h)
			: Object.assign(document.createElement('canvas'), { width: w, height: h });
	const c = g.getContext('2d');
	if (!c) return g;
	const img = c.createImageData(w, h);
	const data = img.data;
	const rocks = new Set(spec.rocks.map(([x, y]) => `${x},${y}`));
	const spawns = spec.spawns ? new Set(spec.spawns.map(([x, y]) => `${x},${y}`)) : undefined;
	const cache = new Map<string, [number, number, number]>();
	const rgb = (hex: string) => {
		let v = cache.get(hex);
		if (!v) {
			v = parseHex(hex);
			cache.set(hex, v);
		}
		return v;
	};
	for (let y = 0; y < spec.h; y++) {
		for (let x = 0; x < spec.w; x++) {
			for (let sy = 0; sy < sub; sy++) {
				for (let sx = 0; sx < sub; sx++) {
					const [r, gv, b] = rgb(terrainHex(x, y, sx, sy, rocks, spawns, cols, seed));
					const px0 = x * cell + sx * texel;
					const py0 = y * cell + sy * texel;
					for (let ty = 0; ty < texel; ty++) {
						for (let tx = 0; tx < texel; tx++) {
							const i = ((py0 + ty) * w + px0 + tx) * 4;
							data[i] = r;
							data[i + 1] = gv;
							data[i + 2] = b;
							data[i + 3] = 255;
						}
					}
				}
			}
		}
	}
	c.putImageData(img, 0, 0);
	if (terrainCache.size > 20) {
		const first = terrainCache.keys().next().value;
		if (first) terrainCache.delete(first);
	}
	terrainCache.set(key, g);
	return g;
}

export type MapThumbSrc = {
	w: number;
	h: number;
	rocks: [number, number][];
	cores?: [number, number][];
	core?: [number, number][];
	spawns: [number, number][];
	slug?: string;
	seed?: number;
	name?: string;
	id?: number;
};

export function paintMapThumb(canvas: HTMLCanvasElement, map: MapThumbSrc, large = false) {
	const cssW = Math.max(1, canvas.clientWidth || canvas.width);
	const cssH = Math.max(1, canvas.clientHeight || canvas.height);
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	canvas.width = Math.floor(cssW * dpr);
	canvas.height = Math.floor(cssH * dpr);
	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	const scale = Math.min(cssW / map.w, cssH / map.h);
	const ox = (cssW - map.w * scale) / 2;
	const oy = (cssH - map.h * scale) / 2;
	ctx.fillStyle = '#050605';
	ctx.fillRect(0, 0, cssW, cssH);
	ctx.imageSmoothingEnabled = false;
	ctx.save();
	ctx.translate(ox, oy);
	ctx.scale(scale, scale);
	const bmp = paintTerrainBitmap(map, large ? 2 : 1);
	ctx.drawImage(bmp, 0, 0, map.w, map.h);
	ctx.fillStyle = 'rgba(168, 48, 36, 0.18)';
	for (const [x, y] of map.spawns) ctx.fillRect(x, y, 1, 1);
	const cores = map.cores ?? map.core ?? [];
	for (const c of clusterCells(cores)) {
		drawRelay(ctx, c.cx, c.cy, { scale: c.count > 1 ? 1.12 : 0.92, rings: 0 });
	}
	ctx.restore();
}

export type ShopSlot = { kind: 'inspect' } | { kind: 'build'; id: number } | { kind: 'strike'; id: number };

export function paintShopIcon(canvas: HTMLCanvasElement, slot: ShopSlot) {
	const size = 128;
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	canvas.width = Math.floor(size * dpr);
	canvas.height = Math.floor(size * dpr);
	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	// No plate at all: the icon sits straight on the tile so the tile's own border is
	// the only frame. Drawing a well here gave every item a second, inner border.
	ctx.save();
	const inset = 6;
	const inner = size - inset * 2;
	ctx.beginPath();
	ctx.rect(inset, inset, inner, inner);
	ctx.clip();
	ctx.translate(size / 2, size / 2);
	if (slot.kind === 'inspect') {
		ctx.scale(inner / 1.2, inner / 1.2);
		drawInspect(ctx);
	} else if (slot.kind === 'strike') {
		ctx.scale(inner / 1.2, inner / 1.2);
		drawStrike(ctx, slot.id);
	} else {
		ctx.scale(inner / 1.48, inner / 1.48);
		const build = BUILD_BY_ID[slot.id] ?? 'autocannon';
		drawTurret(ctx, build, { aim: 0 });
	}
	ctx.restore();
}

export function paintCreepIcon(canvas: HTMLCanvasElement, kind: CreepKind) {
	const size = 96;
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	canvas.width = Math.floor(size * dpr);
	canvas.height = Math.floor(size * dpr);
	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	// Transparent: .wave-unit already supplies the frame.
	ctx.save();
	ctx.translate(size / 2, size / 2 + 6);
	ctx.scale(size / 1.18, size / 1.18);
	drawCreep(ctx, kind, {
		heading: -0.55,
		flying: kind === 'wasp' || kind === 'flicker'
	});
	ctx.restore();
}

export function asMapThumb(doc: MapDoc): MapThumbSrc {
	return {
		w: doc.w,
		h: doc.h,
		rocks: doc.rocks,
		cores: doc.cores,
		spawns: doc.spawns,
		slug: doc.slug,
		seed: doc.seed,
		name: doc.name
	};
}
