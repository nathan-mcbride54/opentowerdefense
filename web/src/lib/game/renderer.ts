import type { CreepView, MapStatic, Snapshot, TowerView } from './types';
import type { Palette } from './settings';

const BASE = {
	void: '#0c100b',
	soil: '#1c2218',
	scrub: '#2a3322',
	scrub2: '#323b28',
	sand: '#3a372c',
	rock: '#4c483c',
	rockLit: '#6a6556',
	rockShade: '#2e2c26',
	grid: 'rgba(196, 210, 160, 0.07)',
	gridHot: 'rgba(214, 228, 170, 0.18)',
	spawn: 'rgba(168, 62, 48, 0.35)',
	spawnEdge: 'rgba(220, 90, 70, 0.55)',
	core: '#4ee0d8',
	coreDim: '#1a5c58',
	wall: '#c9b892',
	wallEdge: '#8a7d5c',
	valid: 'rgba(90, 210, 160, 0.55)',
	invalid: 'rgba(220, 70, 60, 0.55)',
	range: 'rgba(90, 210, 200, 0.22)',
	rangeAir: 'rgba(120, 190, 230, 0.22)',
	hpBack: 'rgba(0,0,0,0.55)',
	hpOk: '#6fd08a',
	hpMid: '#d4c05a',
	hpBad: '#d45a48',
	runner: '#e07a4a',
	lorry: '#c45c3a',
	bulwark: '#8a4a38',
	wasp: '#e8c24a',
	colossus: '#c45a88',
	mite: '#b8d46a',
	medic: '#7ee0a8',
	shade: '#6a7088',
	flicker: '#7ad0e8',
	ac: '#7a96a8',
	how: '#d4a24a',
	sky: '#7ec4e0',
	inferno: '#e07038',
	arc: '#7ad0c8',
	pulse: '#b07ae0',
	helios: '#f0e080',
	swarm: '#8ec8f0',
	siege: '#c07050',
	tracer: '#f0e0a0',
	shell: '#f0c060',
	burst: '#f0a040',
	slow: 'rgba(120, 190, 255, 0.55)'
};

type Colors = typeof BASE;

function paletteColors(palette: Palette): Colors {
	if (palette === 'safe') {
		return {
			...BASE,
			runner: '#f4a36a',
			lorry: '#d88858',
			bulwark: '#b07050',
			wasp: '#f0e070',
			colossus: '#d080c0',
			mite: '#c8e878',
			medic: '#90e8b8',
			shade: '#8890a8',
			flicker: '#90d8f0',
			hpOk: '#7ec8e8',
			hpMid: '#e8d070',
			hpBad: '#e878a0',
			spawn: 'rgba(200, 90, 140, 0.35)',
			spawnEdge: 'rgba(230, 110, 160, 0.55)'
		};
	}
	if (palette === 'high') {
		return {
			...BASE,
			void: '#050605',
			soil: '#141810',
			core: '#7ffff6',
			runner: '#ff8a4a',
			wasp: '#ffe25a',
			gridHot: 'rgba(255, 255, 200, 0.28)',
			valid: 'rgba(80, 255, 180, 0.7)',
			invalid: 'rgba(255, 70, 70, 0.7)'
		};
	}
	return BASE;
}

function hash(x: number, y: number): number {
	let n = Math.imul(x, 374761393) + Math.imul(y, 668265263);
	n = Math.imul(n ^ (n >>> 13), 1274126177);
	return ((n ^ (n >>> 16)) >>> 0) / 4294967296;
}

export class BattlefieldRenderer {
	private canvas: HTMLCanvasElement;
	private ctx: CanvasRenderingContext2D;
	private map: MapStatic | null = null;
	private terrain: OffscreenCanvas | HTMLCanvasElement | null = null;
	private layout = { dpr: 1, scale: 32, ox: 0, oy: 0, cssW: 0, cssH: 0 };
	private cols = paletteColors('default');
	private reducedFx = false;
	private palette: Palette = 'default';
	private fit = 32;
	private zoom = 1;
	private panX = 0;
	private panY = 0;
	private mini = { x: 0, y: 0, w: 0, h: 0 };

	constructor(canvas: HTMLCanvasElement) {
		const ctx = canvas.getContext('2d');
		if (!ctx) throw new Error('Canvas 2D unavailable');
		this.canvas = canvas;
		this.ctx = ctx;
	}

	setLook(palette: Palette, reducedFx: boolean) {
		const paletteChanged = palette !== this.palette;
		this.palette = palette;
		this.cols = paletteColors(palette);
		this.reducedFx = reducedFx;
		if (paletteChanged && this.map) {
			this.terrain = null;
			this.paintTerrain();
		}
	}

	setMap(map: MapStatic) {
		this.map = map;
		this.terrain = null;
		this.zoom = 1;
		this.panX = 0;
		this.panY = 0;
		this.resize();
	}

	resize() {
		const map = this.map;
		if (!map) return;
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		const cssW = Math.max(1, this.canvas.clientWidth);
		const cssH = Math.max(1, this.canvas.clientHeight);
		this.canvas.width = Math.floor(cssW * dpr);
		this.canvas.height = Math.floor(cssH * dpr);
		this.fit = Math.min(cssW / map.w, cssH / map.h);
		this.layout.cssW = cssW;
		this.layout.cssH = cssH;
		this.layout.dpr = dpr;
		this.clampView();
		this.syncLayout();
		this.paintTerrain();
	}

	resetView() {
		this.zoom = 1;
		this.panX = 0;
		this.panY = 0;
		this.syncLayout();
	}

	panBy(dx: number, dy: number) {
		this.panX += dx;
		this.panY += dy;
		this.clampView();
		this.syncLayout();
	}

	zoomAt(clientX: number, clientY: number, factor: number) {
		const map = this.map;
		if (!map) return;
		const rect = this.canvas.getBoundingClientRect();
		const px = clientX - rect.left;
		const py = clientY - rect.top;
		const { scale, ox, oy, cssW, cssH } = this.layout;
		const wx = (px - ox) / scale;
		const wy = (py - oy) / scale;
		this.zoom = Math.min(6, Math.max(1, this.zoom * factor));
		const newScale = this.fit * this.zoom;
		this.panX = px - wx * newScale - (cssW - map.w * newScale) / 2;
		this.panY = py - wy * newScale - (cssH - map.h * newScale) / 2;
		this.clampView();
		this.syncLayout();
	}

	private clampView() {
		const map = this.map;
		if (!map) return;
		this.zoom = Math.min(6, Math.max(1, this.zoom));
		if (this.zoom <= 1.02) {
			this.zoom = 1;
			this.panX = 0;
			this.panY = 0;
			return;
		}
		const { cssW, cssH } = this.layout;
		const scale = this.fit * this.zoom;
		const extraX = Math.max(0, (map.w * scale - cssW) / 2 + 48);
		const extraY = Math.max(0, (map.h * scale - cssH) / 2 + 48);
		this.panX = Math.min(extraX, Math.max(-extraX, this.panX));
		this.panY = Math.min(extraY, Math.max(-extraY, this.panY));
	}

	private syncLayout() {
		const map = this.map;
		if (!map) return;
		const { dpr, cssW, cssH } = this.layout;
		const scale = this.fit * this.zoom;
		this.layout = {
			dpr,
			scale,
			ox: (cssW - map.w * scale) / 2 + this.panX,
			oy: (cssH - map.h * scale) / 2 + this.panY,
			cssW,
			cssH
		};
	}

	cellAt(clientX: number, clientY: number): { x: number; y: number } | null {
		const map = this.map;
		if (!map) return null;
		const rect = this.canvas.getBoundingClientRect();
		const { scale, ox, oy } = this.layout;
		const x = (clientX - rect.left - ox) / scale;
		const y = (clientY - rect.top - oy) / scale;
		const cx = Math.floor(x);
		const cy = Math.floor(y);
		if (cx < 0 || cy < 0 || cx >= map.w || cy >= map.h) return null;
		if (this.overMinimap(clientX, clientY)) return null;
		return { x: cx, y: cy };
	}

	overMinimap(clientX: number, clientY: number): boolean {
		return this.minimapCell(clientX, clientY) != null;
	}

	minimapCell(clientX: number, clientY: number): { x: number; y: number } | null {
		const map = this.map;
		if (!map || this.mini.w <= 0) return null;
		const rect = this.canvas.getBoundingClientRect();
		const px = clientX - rect.left;
		const py = clientY - rect.top;
		const { x, y, w, h } = this.mini;
		if (px < x || py < y || px > x + w || py > y + h) return null;
		const cx = Math.max(0, Math.min(map.w - 1, Math.floor(((px - x) / w) * map.w)));
		const cy = Math.max(0, Math.min(map.h - 1, Math.floor(((py - y) / h) * map.h)));
		return { x: cx, y: cy };
	}

	lookAt(cx: number, cy: number) {
		const map = this.map;
		if (!map) return;
		this.zoom = Math.max(this.zoom, 2.2);
		const { cssW, cssH } = this.layout;
		const scale = this.fit * this.zoom;
		this.panX = cssW / 2 - (cx + 0.5) * scale - (cssW - map.w * scale) / 2;
		this.panY = cssH / 2 - (cy + 0.5) * scale - (cssH - map.h * scale) / 2;
		this.clampView();
		this.syncLayout();
	}

	render(snap: Snapshot, paused: boolean) {
		const map = this.map;
		if (!map) return;
		const { ctx } = this;
		const { dpr, scale, ox, oy, cssW, cssH } = this.layout;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		if (snap.hurtFlash > 0 && !this.reducedFx) {
			const p = snap.hurtFlash;
			ctx.translate(p * 5 * Math.sin(snap.time * 52), p * 3.5 * Math.cos(snap.time * 41));
		}
		ctx.fillStyle = this.cols.void;
		ctx.fillRect(0, 0, cssW, cssH);

		ctx.save();
		ctx.translate(ox, oy);
		ctx.scale(scale, scale);

		if (this.terrain) {
			ctx.imageSmoothingEnabled = false;
			ctx.drawImage(this.terrain, 0, 0, map.w, map.h);
		}

		this.drawSpawns(map, snap.time);
		this.drawCore(snap);
		if (snap.build > 0 || snap.build2 > 0 || snap.strike > 0 || snap.strike2 > 0) {
			this.drawGrid(map);
		}
		this.drawWalls(snap.walls);
		this.drawWalk(snap.walkPaths);
		this.drawRange(snap);
		this.drawHoverHand(snap.hover, snap.build, snap.strike, false);
		this.drawHoverHand(snap.hover2, snap.build2, snap.strike2, true);
		this.drawTowers(snap.towers, snap.selected?.id ?? null, snap.selected2?.id ?? null);
		for (const c of snap.creeps) {
			if (!c.flying) this.drawCreep(c);
		}
		this.drawProjectiles(snap);
		this.drawBeams(snap);
		for (const c of snap.creeps) {
			if (c.flying) this.drawCreep(c);
		}
		this.drawFx(snap);
		for (const c of snap.creeps) this.drawHp(c);

		ctx.restore();

		if (snap.hurtFlash > 0) {
			ctx.fillStyle = `rgba(160, 24, 18, ${0.22 * snap.hurtFlash})`;
			ctx.fillRect(0, 0, cssW, cssH);
		}
		if (paused && !snap.defeated) {
			ctx.fillStyle = 'rgba(8, 10, 8, 0.28)';
			ctx.fillRect(0, 0, cssW, cssH);
		}
		this.drawMinimap(snap);
	}

	private paintTerrain() {
		const map = this.map;
		if (!map) return;
		const cell = 24;
		const w = map.w * cell;
		const h = map.h * cell;
		const g =
			typeof OffscreenCanvas !== 'undefined'
				? new OffscreenCanvas(w, h)
				: Object.assign(document.createElement('canvas'), { width: w, height: h });
		const c = g.getContext('2d');
		if (!c) return;
		c.fillStyle = this.cols.soil;
		c.fillRect(0, 0, w, h);
		for (let y = 0; y < map.h; y++) {
			for (let x = 0; x < map.w; x++) {
				const n = hash(x, y);
				c.fillStyle = n > 0.62 ? this.cols.scrub2 : n > 0.28 ? this.cols.scrub : this.cols.sand;
				c.fillRect(x * cell, y * cell, cell, cell);
				if (n > 0.84) {
					c.fillStyle = 'rgba(0,0,0,0.08)';
					c.fillRect(x * cell + 4, y * cell + 6, 6, 4);
				}
			}
		}
		const rockSet = new Set(map.rocks.map(([x, y]) => `${x},${y}`));
		for (const [x, y] of map.rocks) {
			this.paintRock(c, x, y, cell, rockSet);
		}
		this.terrain = g;
	}

	private drawMinimap(snap: Snapshot) {
		const map = this.map;
		if (!map) return;
		const { ctx } = this;
		const { cssW, cssH, scale, ox, oy } = this.layout;
		const maxW = Math.min(156, cssW * 0.28);
		const maxH = Math.min(110, cssH * 0.28);
		const cell = Math.min(maxW / map.w, maxH / map.h);
		const w = map.w * cell;
		const h = map.h * cell;
		const x = 10;
		const y = cssH - h - 12;
		this.mini = { x, y, w, h };
		ctx.save();
		ctx.fillStyle = 'rgba(8, 12, 8, 0.82)';
		ctx.fillRect(x - 3, y - 3, w + 6, h + 6);
		ctx.strokeStyle = 'rgba(196, 210, 160, 0.28)';
		ctx.lineWidth = 1;
		ctx.strokeRect(x - 3, y - 3, w + 6, h + 6);
		ctx.fillStyle = this.cols.soil;
		ctx.fillRect(x, y, w, h);
		ctx.fillStyle = this.cols.rock;
		for (const [rx, ry] of map.rocks) {
			ctx.fillRect(x + rx * cell, y + ry * cell, cell, cell);
		}
		ctx.fillStyle = this.cols.wall;
		for (const [wx, wy] of snap.walls) {
			ctx.fillRect(x + wx * cell, y + wy * cell, cell, cell);
		}
		ctx.fillStyle = this.cols.spawnEdge;
		for (const [sx, sy] of map.spawns) {
			ctx.fillRect(x + sx * cell, y + sy * cell, cell, cell);
		}
		ctx.fillStyle = this.cols.core;
		for (const [cx, cy] of map.core) {
			ctx.fillRect(x + cx * cell, y + cy * cell, cell, cell);
		}
		for (const c of snap.creeps) {
			ctx.fillStyle =
				c.kind === 'shade'
					? this.cols.shade
					: c.kind === 'flicker'
						? this.cols.flicker
						: c.flying
							? this.cols.wasp
							: this.cols.runner;
			ctx.fillRect(x + c.x * cell - 1, y + c.y * cell - 1, 2, 2);
		}
		const viewX = -ox / scale;
		const viewY = -oy / scale;
		const viewW = cssW / scale;
		const viewH = cssH / scale;
		ctx.strokeStyle = 'rgba(78, 224, 216, 0.85)';
		ctx.lineWidth = 1;
		ctx.strokeRect(x + viewX * cell, y + viewY * cell, viewW * cell, viewH * cell);
		ctx.restore();
	}

	private paintRock(
		c: OffscreenCanvasRenderingContext2D | CanvasRenderingContext2D,
		x: number,
		y: number,
		cell: number,
		rocks: Set<string>
	) {
		const px = x * cell;
		const py = y * cell;
		c.fillStyle = this.cols.rockShade;
		c.fillRect(px, py, cell, cell);
		c.fillStyle = this.cols.rock;
		c.beginPath();
		c.moveTo(px + 3, py + cell - 2);
		c.lineTo(px + 5, py + 6);
		c.lineTo(px + cell - 4, py + 4);
		c.lineTo(px + cell - 2, py + cell - 3);
		c.closePath();
		c.fill();
		c.fillStyle = this.cols.rockLit;
		c.fillRect(px + 6, py + 5, cell * 0.35, 3);
		if (!rocks.has(`${x},${y - 1}`)) {
			c.fillStyle = 'rgba(0,0,0,0.25)';
			c.fillRect(px, py + cell - 3, cell, 3);
		}
	}

	private drawGrid(map: MapStatic) {
		const { ctx } = this;
		ctx.beginPath();
		ctx.strokeStyle = this.cols.gridHot;
		ctx.lineWidth = 0.03;
		for (let x = 0; x <= map.w; x++) {
			ctx.moveTo(x, 0);
			ctx.lineTo(x, map.h);
		}
		for (let y = 0; y <= map.h; y++) {
			ctx.moveTo(0, y);
			ctx.lineTo(map.w, y);
		}
		ctx.stroke();
	}

	private drawSpawns(map: MapStatic, time: number) {
		const { ctx } = this;
		const pulse = 0.35 + 0.15 * Math.sin(time * 3);
		ctx.fillStyle = this.cols.spawn;
		for (const [x, y] of map.spawns) {
			ctx.fillRect(x, y, 1, 1);
		}
		ctx.strokeStyle = this.cols.spawnEdge;
		ctx.globalAlpha = pulse;
		ctx.lineWidth = 0.06;
		for (const [x, y] of map.spawns) {
			ctx.strokeRect(x + 0.12, y + 0.12, 0.76, 0.76);
		}
		ctx.globalAlpha = 1;
	}

	private drawCore(snap: Snapshot) {
		const relays = snap.cores?.length ? snap.cores : [snap.core];
		const rings = this.reducedFx ? 1 : 3;
		for (const [cx, cy] of relays) {
			this.paintRelay(snap, cx, cy, rings);
		}
	}

	private paintRelay(snap: Snapshot, cx: number, cy: number, rings: number) {
		const { ctx } = this;
		const frac = snap.integrity / Math.max(1, snap.integrityMax);
		ctx.save();
		ctx.translate(cx, cy);
		for (let i = rings; i >= 1; i--) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.55 + i * 0.28 + 0.04 * Math.sin(snap.time * 2 + i), 0, Math.PI * 2);
			ctx.strokeStyle = `rgba(78, 224, 216, ${0.18 / i})`;
			ctx.lineWidth = 0.05;
			ctx.stroke();
		}
		ctx.rotate(Math.PI / 4);
		ctx.fillStyle = this.cols.coreDim;
		ctx.fillRect(-0.55, -0.55, 1.1, 1.1);
		ctx.fillStyle = this.cols.core;
		ctx.globalAlpha = 0.35 + 0.5 * frac;
		ctx.fillRect(-0.38, -0.38, 0.76, 0.76);
		ctx.globalAlpha = 1;
		ctx.restore();
		ctx.fillStyle = this.cols.core;
		ctx.fillRect(cx - 0.06, cy - 0.85, 0.12, 0.45);
		ctx.beginPath();
		ctx.arc(cx, cy - 0.92, 0.1, 0, Math.PI * 2);
		ctx.fill();
	}

	private drawWalls(walls: [number, number][]) {
		const { ctx } = this;
		for (const [x, y] of walls) {
			ctx.fillStyle = this.cols.wallEdge;
			ctx.fillRect(x + 0.08, y + 0.12, 0.84, 0.8);
			ctx.fillStyle = this.cols.wall;
			ctx.fillRect(x + 0.1, y + 0.08, 0.8, 0.72);
			ctx.fillStyle = 'rgba(0,0,0,0.18)';
			ctx.fillRect(x + 0.1, y + 0.42, 0.8, 0.08);
			ctx.fillRect(x + 0.48, y + 0.08, 0.06, 0.72);
		}
	}

	private drawWalk(paths: [number, number][][] | undefined) {
		if (!paths?.length) return;
		const { ctx } = this;
		ctx.save();
		ctx.strokeStyle = 'rgba(90, 210, 200, 0.32)';
		ctx.lineWidth = 0.09;
		ctx.setLineDash([0.22, 0.14]);
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';
		for (const path of paths) {
			if (path.length < 2) continue;
			ctx.beginPath();
			ctx.moveTo(path[0][0] + 0.5, path[0][1] + 0.5);
			for (let i = 1; i < path.length; i++) {
				ctx.lineTo(path[i][0] + 0.5, path[i][1] + 0.5);
			}
			ctx.stroke();
		}
		ctx.setLineDash([]);
		ctx.restore();
	}

	private drawHoverHand(
		h: Snapshot['hover'],
		build: number,
		strike: number,
		p2: boolean
	) {
		if (!h) return;
		const { ctx } = this;
		if (h.strike || strike > 0) {
			ctx.beginPath();
			ctx.arc(h.x + 0.5, h.y + 0.5, Math.max(0.4, h.range), 0, Math.PI * 2);
			ctx.fillStyle = h.valid
				? p2
					? 'rgba(232, 194, 74, 0.2)'
					: 'rgba(240, 160, 70, 0.18)'
				: 'rgba(220, 70, 60, 0.16)';
			ctx.fill();
			ctx.strokeStyle = h.valid
				? p2
					? 'rgba(232, 194, 74, 0.85)'
					: 'rgba(240, 160, 70, 0.7)'
				: this.cols.invalid;
			ctx.lineWidth = 0.06;
			ctx.stroke();
			return;
		}
		if (build === 0) {
			if (p2) {
				ctx.fillStyle = 'rgba(232, 194, 74, 0.12)';
				ctx.fillRect(h.x, h.y, 1, 1);
				ctx.strokeStyle = 'rgba(232, 194, 74, 0.9)';
				ctx.lineWidth = 0.08;
				ctx.strokeRect(h.x + 0.08, h.y + 0.08, 0.84, 0.84);
			}
			return;
		}
		ctx.fillStyle = h.valid
			? p2
				? 'rgba(232, 194, 74, 0.24)'
				: 'rgba(90, 210, 160, 0.22)'
			: 'rgba(220, 70, 60, 0.22)';
		ctx.fillRect(h.x, h.y, 1, 1);
		ctx.strokeStyle = h.valid
			? p2
				? 'rgba(232, 194, 74, 0.9)'
				: this.cols.valid
			: this.cols.invalid;
		ctx.lineWidth = 0.07;
		ctx.strokeRect(h.x + 0.06, h.y + 0.06, 0.88, 0.88);
	}

	private drawRange(snap: Snapshot) {
		this.paintRangeRing(
			this.rangeFrom(snap.selected, snap.hover, snap.build, snap.strike, snap.relocating),
			false
		);
		this.paintRangeRing(
			this.rangeFrom(snap.selected2, snap.hover2, snap.build2, snap.strike2, snap.relocating2),
			true
		);
	}

	private rangeFrom(
		sel: Snapshot['selected'],
		hover: Snapshot['hover'],
		build: number,
		strike: number,
		relocating: boolean
	): { cx: number; cy: number; range: number; air: boolean } | null {
		if (relocating && hover && hover.range > 0) {
			return {
				cx: hover.x + 0.5,
				cy: hover.y + 0.5,
				range: hover.range,
				air: hover.hitsAir && !hover.hitsGround
			};
		}
		if (sel && sel.range > 0) {
			return {
				cx: sel.x + 0.5,
				cy: sel.y + 0.5,
				range: sel.range,
				air: sel.hitsAir && !sel.hitsGround
			};
		}
		if (hover && (build > 0 || strike > 0) && hover.range > 0) {
			return {
				cx: hover.x + 0.5,
				cy: hover.y + 0.5,
				range: hover.range,
				air: hover.hitsAir && !hover.hitsGround
			};
		}
		return null;
	}

	private paintRangeRing(
		src: { cx: number; cy: number; range: number; air: boolean } | null,
		p2: boolean
	) {
		if (!src || src.range <= 0) return;
		const { ctx } = this;
		ctx.beginPath();
		ctx.arc(src.cx, src.cy, src.range, 0, Math.PI * 2);
		if (p2) {
			ctx.fillStyle = src.air ? 'rgba(240, 180, 70, 0.14)' : 'rgba(232, 194, 74, 0.16)';
			ctx.strokeStyle = 'rgba(232, 194, 74, 0.75)';
		} else {
			ctx.fillStyle = src.air ? this.cols.rangeAir : this.cols.range;
			ctx.strokeStyle = src.air ? 'rgba(126, 196, 224, 0.7)' : 'rgba(90, 210, 200, 0.7)';
		}
		ctx.fill();
		ctx.setLineDash([0.18, 0.12]);
		ctx.lineWidth = 0.045;
		ctx.stroke();
		ctx.setLineDash([]);
	}

	private drawTowers(towers: TowerView[], selectedId: number | null, selected2Id: number | null) {
		for (const t of towers) {
			this.drawTower(t, t.id === selectedId, t.id === selected2Id && t.id !== selectedId);
		}
	}

	private drawTower(t: TowerView, selected: boolean, selected2: boolean) {
		const { ctx } = this;
		const x = t.x + 0.5;
		const y = t.y + 0.5;
		ctx.save();
		ctx.translate(x, y);
		if (selected || selected2) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.52, 0, Math.PI * 2);
			ctx.strokeStyle = selected ? 'rgba(240, 230, 180, 0.8)' : 'rgba(232, 194, 74, 0.9)';
			ctx.lineWidth = 0.05;
			ctx.stroke();
		}
		if (t.overcharged) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.5, 0, Math.PI * 2);
			ctx.strokeStyle = 'rgba(255, 196, 80, 0.75)';
			ctx.lineWidth = 0.055;
			ctx.stroke();
		}
		if (t.stunned) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.48, 0, Math.PI * 2);
			ctx.strokeStyle = 'rgba(196, 72, 88, 0.9)';
			ctx.setLineDash([0.08, 0.07]);
			ctx.lineWidth = 0.05;
			ctx.stroke();
			ctx.setLineDash([]);
		}
		const pad =
			t.kind === 'howitzer'
				? this.cols.how
				: t.kind === 'skystinger' || t.kind === 'swarmRack'
					? this.cols.sky
					: t.kind === 'inferno'
						? this.cols.inferno
						: t.kind === 'arcLance'
							? this.cols.arc
							: t.kind === 'pulseArray'
								? this.cols.pulse
								: t.kind === 'helios'
									? t.airFocus
										? this.cols.sky
										: this.cols.helios
									: t.kind === 'siegeRail'
										? this.cols.siege
										: this.cols.ac;
		ctx.fillStyle = '#1a1e1a';
		ctx.beginPath();
		ctx.arc(0, 0, 0.42, 0, Math.PI * 2);
		ctx.fill();
		ctx.fillStyle = pad;
		if (t.kind === 'skystinger' || t.kind === 'swarmRack') {
			ctx.beginPath();
			ctx.moveTo(0, -0.38);
			ctx.lineTo(0.36, 0.28);
			ctx.lineTo(-0.36, 0.28);
			ctx.closePath();
			ctx.fill();
		} else if (t.kind === 'pulseArray') {
			ctx.fillRect(-0.3, -0.3, 0.6, 0.6);
		} else if (t.kind === 'inferno') {
			ctx.beginPath();
			ctx.moveTo(0, -0.34);
			ctx.lineTo(0.32, 0.3);
			ctx.lineTo(-0.32, 0.3);
			ctx.closePath();
			ctx.fill();
		} else if (t.kind === 'howitzer' || t.kind === 'siegeRail') {
			ctx.beginPath();
			ctx.arc(0, 0, 0.34, 0, Math.PI * 2);
			ctx.fill();
		} else {
			for (let i = 0; i < 6; i++) {
				const a = (i * Math.PI) / 3 - Math.PI / 6;
				if (i === 0) ctx.beginPath();
				const cmd = i === 0 ? ctx.moveTo.bind(ctx) : ctx.lineTo.bind(ctx);
				cmd(Math.cos(a) * 0.34, Math.sin(a) * 0.34);
			}
			ctx.closePath();
			ctx.fill();
		}
		ctx.rotate(t.aim);
		ctx.fillStyle = '#d8d2c4';
		if (t.kind === 'howitzer' || t.kind === 'siegeRail') {
			ctx.fillRect(0.05, -0.1, t.kind === 'siegeRail' ? 0.56 : 0.48, 0.2);
			ctx.fillStyle = '#2a2a26';
			ctx.fillRect(t.kind === 'siegeRail' ? 0.56 : 0.48, -0.07, 0.08, 0.14);
		} else if (t.kind === 'skystinger' || t.kind === 'swarmRack') {
			ctx.fillRect(0.02, -0.16, 0.42, 0.08);
			ctx.fillRect(0.02, 0.08, 0.42, 0.08);
			if (t.kind === 'swarmRack') ctx.fillRect(0.08, -0.04, 0.36, 0.08);
		} else if (t.kind === 'helios' || t.kind === 'arcLance') {
			ctx.fillRect(0.04, -0.06, 0.5, 0.12);
		} else if (t.kind === 'inferno') {
			ctx.fillRect(0.02, -0.12, 0.4, 0.24);
		} else {
			ctx.fillRect(0.04, -0.14, 0.4, 0.07);
			ctx.fillRect(0.04, 0.07, 0.4, 0.07);
		}
		if (t.tier > 0) {
			ctx.rotate(-t.aim);
			ctx.fillStyle = '#f0e8b0';
			for (let i = 0; i < t.tier; i++) {
				ctx.fillRect(-0.06 + i * 0.1, 0.36, 0.07, 0.07);
			}
		}
		ctx.restore();
	}

	private drawCreep(c: CreepView) {
		const { ctx } = this;
		ctx.save();
		if (c.flying) {
			ctx.fillStyle = 'rgba(0,0,0,0.28)';
			ctx.beginPath();
			ctx.ellipse(c.x + 0.08, c.y + 0.28, c.radius * 1.1, c.radius * 0.45, 0, 0, Math.PI * 2);
			ctx.fill();
			ctx.translate(c.x, c.y - 0.22);
		} else {
			ctx.translate(c.x, c.y);
		}
		ctx.rotate(c.heading);
		const col =
			c.kind === 'wasp'
				? this.cols.wasp
				: c.kind === 'colossus'
					? this.cols.colossus
					: c.kind === 'mite'
						? this.cols.mite
						: c.kind === 'medic'
							? this.cols.medic
							: c.kind === 'shade'
								? this.cols.shade
								: c.kind === 'flicker'
									? this.cols.flicker
							: c.kind === 'bulwark'
							? this.cols.bulwark
							: c.kind === 'lorry'
								? this.cols.lorry
								: this.cols.runner;
		ctx.fillStyle = col;
		const r = c.radius;
		if (c.kind === 'wasp') {
			ctx.beginPath();
			ctx.moveTo(r * 1.4, 0);
			ctx.lineTo(-r, r);
			ctx.lineTo(-r * 0.4, 0);
			ctx.lineTo(-r, -r);
			ctx.closePath();
			ctx.fill();
		} else if (c.kind === 'colossus') {
			ctx.fillRect(-r, -r * 0.85, r * 2, r * 1.7);
			ctx.fillStyle = '#2a1a22';
			ctx.fillRect(-r * 0.3, -r * 0.4, r * 1.1, r * 0.8);
		} else if (c.kind === 'bulwark') {
			ctx.fillRect(-r, -r * 0.75, r * 1.9, r * 1.5);
			ctx.fillStyle = '#3a241c';
			ctx.beginPath();
			ctx.arc(r * 0.15, 0, r * 0.35, 0, Math.PI * 2);
			ctx.fill();
		} else if (c.kind === 'lorry') {
			roundRect(ctx, -r * 1.1, -r * 0.7, r * 2.2, r * 1.4, 0.08);
			ctx.fill();
		} else if (c.kind === 'mite') {
			ctx.beginPath();
			ctx.moveTo(r * 1.35, 0);
			ctx.lineTo(-r * 0.7, r);
			ctx.lineTo(-r * 0.2, 0);
			ctx.lineTo(-r * 0.7, -r);
			ctx.closePath();
			ctx.fill();
		} else if (c.kind === 'medic') {
			ctx.fillRect(-r * 0.32, -r * 1.05, r * 0.64, r * 2.1);
			ctx.fillRect(-r * 1.05, -r * 0.32, r * 2.1, r * 0.64);
		} else if (c.kind === 'shade') {
			ctx.globalAlpha = 0.5;
			ctx.beginPath();
			ctx.moveTo(r * 1.15, 0);
			ctx.lineTo(-r * 0.55, r * 0.72);
			ctx.lineTo(-r * 0.18, 0);
			ctx.lineTo(-r * 0.55, -r * 0.72);
			ctx.closePath();
			ctx.fill();
			ctx.strokeStyle = 'rgba(210, 216, 230, 0.4)';
			ctx.lineWidth = 0.04;
			ctx.stroke();
		} else if (c.kind === 'flicker') {
			ctx.beginPath();
			ctx.moveTo(r * 1.25, 0);
			ctx.lineTo(r * 0.1, r * 0.35);
			ctx.lineTo(-r * 0.15, r * 0.08);
			ctx.lineTo(-r, r * 0.55);
			ctx.lineTo(-r * 0.35, 0);
			ctx.lineTo(-r, -r * 0.55);
			ctx.lineTo(-r * 0.15, -r * 0.08);
			ctx.lineTo(r * 0.1, -r * 0.35);
			ctx.closePath();
			ctx.fill();
		} else {
			ctx.beginPath();
			ctx.moveTo(r * 1.2, 0);
			ctx.lineTo(-r, r * 0.8);
			ctx.lineTo(-r, -r * 0.8);
			ctx.closePath();
			ctx.fill();
		}
		if (c.slowed) {
			ctx.rotate(-c.heading);
			ctx.strokeStyle = this.cols.slow;
			ctx.lineWidth = 0.05;
			ctx.beginPath();
			ctx.arc(0, 0, r + 0.14, 0, Math.PI * 2);
			ctx.stroke();
		}
		ctx.restore();
	}

	private drawHp(c: CreepView) {
		if (c.hp >= c.hpMax * 0.995) return;
		const { ctx } = this;
		const w = Math.max(0.55, c.radius * 2.2);
		const y = c.y - (c.flying ? 0.55 : c.radius) - 0.22;
		const frac = Math.max(0, c.hp / c.hpMax);
		ctx.fillStyle = this.cols.hpBack;
		ctx.fillRect(c.x - w / 2, y, w, 0.08);
		ctx.fillStyle = frac > 0.55 ? this.cols.hpOk : frac > 0.28 ? this.cols.hpMid : this.cols.hpBad;
		ctx.fillRect(c.x - w / 2, y, w * frac, 0.08);
	}

	private drawProjectiles(snap: Snapshot) {
		const { ctx } = this;
		for (const p of snap.projectiles) {
			const ang = Math.atan2(p.vy, p.vx);
			ctx.save();
			ctx.translate(p.x, p.y);
			ctx.rotate(ang);
			if (p.kind === 'howitzer') {
				ctx.fillStyle = this.cols.shell;
				ctx.fillRect(-0.12, -0.07, 0.24, 0.14);
			} else if (p.kind === 'siegeRail') {
				ctx.fillStyle = this.cols.siege;
				ctx.fillRect(-0.18, -0.05, 0.36, 0.1);
			} else if (p.kind === 'skystinger' || p.kind === 'swarmRack') {
				ctx.fillStyle = this.cols.sky;
				ctx.beginPath();
				ctx.moveTo(0.14, 0);
				ctx.lineTo(-0.1, 0.06);
				ctx.lineTo(-0.1, -0.06);
				ctx.fill();
			} else {
				ctx.strokeStyle = this.cols.tracer;
				ctx.lineWidth = 0.06;
				ctx.beginPath();
				ctx.moveTo(-0.22, 0);
				ctx.lineTo(0.12, 0);
				ctx.stroke();
			}
			ctx.restore();
		}
	}

	private drawFx(snap: Snapshot) {
		const { ctx } = this;
		for (const f of snap.fx) {
			if (this.reducedFx && (f.kind === 'muzzle' || f.kind === 'cash' || f.kind === 'place')) {
				continue;
			}
			ctx.save();
			ctx.translate(f.x, f.y);
			ctx.globalAlpha = Math.max(0, f.life);
			if (f.kind === 'burst' || f.kind === 'kill' || f.kind === 'satchel') {
				ctx.beginPath();
				ctx.arc(0, 0, (1 - f.life) * 0.7 * f.mag + 0.12, 0, Math.PI * 2);
				ctx.strokeStyle = f.kind === 'satchel' ? '#f0a060' : this.cols.burst;
				ctx.lineWidth = 0.07;
				ctx.stroke();
			} else if (f.kind === 'orbital') {
				ctx.beginPath();
				ctx.arc(0, 0, f.mag * (0.35 + 0.65 * (1 - f.life)), 0, Math.PI * 2);
				ctx.strokeStyle = 'rgba(255, 180, 80, 0.85)';
				ctx.lineWidth = 0.1;
				ctx.stroke();
			} else if (f.kind === 'overload' || f.kind === 'pulse') {
				ctx.beginPath();
				ctx.arc(0, 0, f.mag * (0.4 + 0.6 * (1 - f.life)), 0, Math.PI * 2);
				ctx.strokeStyle = f.kind === 'pulse' ? this.cols.pulse : this.cols.slow;
				ctx.lineWidth = 0.07;
				ctx.stroke();
			} else if (f.kind === 'blink') {
				ctx.beginPath();
				ctx.arc(0, 0, 0.18 + (1 - f.life) * 0.55, 0, Math.PI * 2);
				ctx.strokeStyle = this.cols.flicker;
				ctx.lineWidth = 0.06;
				ctx.setLineDash([0.08, 0.06]);
				ctx.stroke();
				ctx.setLineDash([]);
			} else if (f.kind === 'roar') {
				ctx.beginPath();
				ctx.arc(0, 0, f.mag * (0.35 + 0.65 * (1 - f.life)), 0, Math.PI * 2);
				ctx.strokeStyle = 'rgba(196, 72, 88, 0.85)';
				ctx.lineWidth = 0.1;
				ctx.stroke();
			} else if (f.kind === 'overcharge') {
				ctx.beginPath();
				ctx.arc(0, 0, 0.2 + (1 - f.life) * 0.45, 0, Math.PI * 2);
				ctx.strokeStyle = 'rgba(255, 196, 80, 0.85)';
				ctx.lineWidth = 0.07;
				ctx.stroke();
			} else if (f.kind === 'cone') {
				ctx.rotate(f.heading);
				ctx.fillStyle = 'rgba(224, 112, 56, 0.28)';
				ctx.beginPath();
				ctx.moveTo(0, 0);
				ctx.arc(0, 0, f.mag, -0.58, 0.58);
				ctx.closePath();
				ctx.fill();
			} else if (f.kind === 'muzzle') {
				ctx.fillStyle = '#fff4c8';
				ctx.beginPath();
				ctx.arc(0, 0, 0.12 * f.mag, 0, Math.PI * 2);
				ctx.fill();
			} else if (f.kind === 'leak') {
				ctx.strokeStyle = '#ff6a55';
				ctx.lineWidth = 0.08;
				ctx.beginPath();
				ctx.arc(0, 0, 0.4 + (1 - f.life) * 0.5, 0, Math.PI * 2);
				ctx.stroke();
			} else if (f.kind === 'cash') {
				ctx.fillStyle = '#e8d48a';
				ctx.font = '0.32px IBM Plex Sans, sans-serif';
				ctx.textAlign = 'center';
				ctx.fillText(`+${Math.round(f.mag)}`, 0, -0.2 * (1 - f.life));
			} else if (f.kind === 'place' || f.kind === 'upgrade' || f.kind === 'sell') {
				ctx.strokeStyle = f.kind === 'sell' ? '#d4a0a0' : this.cols.core;
				ctx.lineWidth = 0.05;
				ctx.strokeRect(-0.45, -0.45, 0.9, 0.9);
			} else {
				ctx.fillStyle = this.cols.tracer;
				ctx.beginPath();
				ctx.arc(0, 0, 0.08, 0, Math.PI * 2);
				ctx.fill();
			}
			ctx.restore();
		}
	}

	private drawBeams(snap: Snapshot) {
		const { ctx } = this;
		for (const b of snap.beams) {
			ctx.save();
			ctx.globalAlpha = 0.35 + 0.55 * b.life;
			ctx.strokeStyle = b.kind === 'helios' ? this.cols.helios : this.cols.arc;
			ctx.lineWidth = b.kind === 'helios' ? 0.09 : 0.07;
			ctx.beginPath();
			ctx.moveTo(b.x0, b.y0);
			ctx.lineTo(b.x1, b.y1);
			ctx.stroke();
			ctx.restore();
		}
	}
}

function roundRect(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	w: number,
	h: number,
	r: number
) {
	ctx.beginPath();
	ctx.moveTo(x + r, y);
	ctx.lineTo(x + w - r, y);
	ctx.quadraticCurveTo(x + w, y, x + w, y + r);
	ctx.lineTo(x + w, y + h - r);
	ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
	ctx.lineTo(x + r, y + h);
	ctx.quadraticCurveTo(x, y + h, x, y + h - r);
	ctx.lineTo(x, y + r);
	ctx.quadraticCurveTo(x, y, x + r, y);
	ctx.closePath();
}
