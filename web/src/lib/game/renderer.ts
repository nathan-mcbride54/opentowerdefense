import type { CreepView, MapStatic, Snapshot, TowerView } from './types';
import type { Palette } from './settings';
import { BUILD_BY_ID, clusterCells, drawBarricade, drawCreep as paintCreep, drawRelay, drawTurret, drawWallCell, paintTerrainBitmap } from './sprites';

const BASE = {
	// Lantern Dusk. Ground/rock come from the biome table in sprites.ts; these are the
	// overlay colours the renderer owns: grid, markers, ranges, HP and unit tints.
	void: '#050506',
	soil: '#35422a',
	scrub: '#3f5031',
	scrub2: '#4b5f3a',
	sand: '#6b6242',
	rock: '#7d7a72',
	rockLit: '#adaa9f',
	rockShade: '#24231f',
	// The grid is a placement tool, not a combat tool — quiet until a structure is held.
	grid: 'rgba(240, 169, 76, 0.10)',
	spawn: 'rgba(200, 84, 96, 0.30)',
	spawnEdge: 'rgba(232, 115, 107, 0.55)',
	core: '#ffc46e',
	coreDim: '#6a4a1c',
	wall: '#c9bda6',
	wallEdge: '#7d7364',
	valid: 'rgba(255, 196, 110, 0.6)',
	invalid: 'rgba(232, 115, 107, 0.6)',
	range: 'rgba(240, 169, 76, 0.14)',
	rangeAir: 'rgba(127, 200, 184, 0.16)',
	rangeEdge: 'rgba(240, 169, 76, 0.55)',
	rangeAirEdge: 'rgba(127, 200, 184, 0.6)',
	hpBack: 'rgba(6, 5, 4, 0.62)',
	hpOk: '#7fc8b8',
	hpMid: '#f5c96b',
	hpBad: '#e8736b',
	runner: '#f0954e',
	lorry: '#c8574c',
	bulwark: '#8f6a9c',
	wasp: '#f2cf6a',
	colossus: '#d0567f',
	mite: '#b6d472',
	medic: '#74d8b0',
	shade: '#5d6a8c',
	flicker: '#6fc9ea',
	ac: '#8fa6bc',
	how: '#e0aa5c',
	sky: '#86cbe4',
	inferno: '#ef7a44',
	arc: '#7ad8cc',
	pulse: '#b98ce8',
	helios: '#ffd97a',
	swarm: '#96ccf2',
	siege: '#cf8464',
	tracer: '#ffe6ae',
	shell: '#f5c96b',
	burst: '#ffb257',
	slow: 'rgba(127, 200, 184, 0.55)'
};

type Colors = typeof BASE;

/** Memoized: this object-spreads a ~50-key literal and was called once per frame. */
const paletteCache = new Map<Palette, Colors>();

function paletteColors(palette: Palette): Colors {
	const hit = paletteCache.get(palette);
	if (hit) return hit;
	const built = buildPalette(palette);
	paletteCache.set(palette, built);
	return built;
}

function buildPalette(palette: Palette): Colors {
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
			hpOk: '#6fc9ea',
			hpMid: '#f2cf6a',
			hpBad: '#e878a0',
			spawn: 'rgba(200, 90, 140, 0.35)',
			spawnEdge: 'rgba(230, 110, 160, 0.55)'
		};
	}
	if (palette === 'high') {
		return {
			...BASE,
			void: '#000000',
			soil: '#2b3622',
			core: '#ffd98a',
			runner: '#ffa251',
			wasp: '#ffe89a',
			valid: 'rgba(255, 214, 150, 0.8)',
			invalid: 'rgba(255, 110, 100, 0.8)',
			rangeEdge: 'rgba(255, 200, 130, 0.8)',
			rangeAirEdge: 'rgba(150, 225, 210, 0.85)'
		};
	}
	return BASE;
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

	constructor(canvas: HTMLCanvasElement) {
		const ctx = canvas.getContext('2d');
		if (!ctx) throw new Error('Canvas 2D unavailable');
		this.canvas = canvas;
		this.ctx = ctx;
	}

	setLook(palette: Palette, reducedFx: boolean) {
		this.reducedFx = reducedFx;
		if (palette === this.palette) return;
		this.palette = palette;
		this.cols = paletteColors(palette);
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
		return { x: cx, y: cy };
	}

	render(snap: Snapshot, paused: boolean) {
		const map = this.map;
		if (!map) return;
		const { ctx } = this;
		const { dpr, scale, ox, oy, cssW, cssH } = this.layout;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		// Paint the ground BEFORE the shake, or the translate leaves a few unpainted
		// pixels along two edges showing the previous frame.
		ctx.fillStyle = this.cols.void;
		ctx.fillRect(0, 0, cssW, cssH);

		ctx.save();
		// The shake wraps only the world layer so overlays drawn after `restore()` stay put.
		if (snap.hurtFlash > 0 && !this.reducedFx) {
			const p = snap.hurtFlash;
			ctx.translate(p * 5 * Math.sin(snap.time * 52), p * 3.5 * Math.cos(snap.time * 41));
		}
		ctx.translate(ox, oy);
		ctx.scale(scale, scale);

		if (this.terrain) {
			ctx.imageSmoothingEnabled = false;
			ctx.drawImage(this.terrain, 0, 0, map.w, map.h);
		}

		this.drawSpawns(map, snap.time);
		this.drawGrid(map);
		this.drawCore(snap);
		this.drawWalls(snap.walls);
		this.drawWalk(snap.walkPaths);
		this.drawRange(snap);
		this.drawHoverHand(snap.hover, snap.build, snap.strike);
		this.drawTowers(snap.towers, snap.selected?.id ?? null);
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
			ctx.fillStyle = 'rgba(5, 6, 5, 0.38)';
			ctx.fillRect(0, 0, cssW, cssH);
		}
	}

	private paintTerrain() {
		const map = this.map;
		if (!map) return;
		this.terrain = paintTerrainBitmap(
			{
				w: map.w,
				h: map.h,
				rocks: map.rocks,
				spawns: map.spawns,
				name: map.name,
				id: map.id,
				// Pass the same identity the thumbnail uses, so a workshop map does not
				// render with biome 0 in-match after previewing in a seed-derived biome.
				slug: map.slug,
				seed: map.seed
			},
			2
		);
	}

	private drawGrid(map: MapStatic) {
		const { ctx } = this;
		ctx.save();
		ctx.beginPath();
		ctx.strokeStyle = 'rgba(0, 0, 0, 0.7)';
		ctx.lineWidth = 0.085;
		for (let x = 0; x <= map.w; x++) {
			ctx.moveTo(x, 0);
			ctx.lineTo(x, map.h);
		}
		for (let y = 0; y <= map.h; y++) {
			ctx.moveTo(0, y);
			ctx.lineTo(map.w, y);
		}
		ctx.stroke();
		ctx.beginPath();
		ctx.strokeStyle = this.cols.grid;
		ctx.lineWidth = 0.05;
		for (let x = 0; x <= map.w; x++) {
			ctx.moveTo(x, 0);
			ctx.lineTo(x, map.h);
		}
		for (let y = 0; y <= map.h; y++) {
			ctx.moveTo(0, y);
			ctx.lineTo(map.w, y);
		}
		ctx.stroke();
		ctx.restore();
	}

	private drawSpawns(map: MapStatic, time: number) {
		const { ctx } = this;
		const set = new Set(map.spawns.map(([x, y]) => `${x},${y}`));
		const pulse = 0.45 + 0.2 * Math.sin(time * 3);
		ctx.save();
		ctx.fillStyle = this.cols.spawn;
		ctx.globalAlpha = 0.18;
		for (const [x, y] of map.spawns) ctx.fillRect(x, y, 1, 1);
		ctx.globalAlpha = pulse;
		ctx.strokeStyle = this.cols.spawnEdge;
		ctx.lineWidth = 0.08;
		ctx.beginPath();
		for (const [x, y] of map.spawns) {
			if (!set.has(`${x},${y - 1}`)) {
				ctx.moveTo(x, y);
				ctx.lineTo(x + 1, y);
			}
			if (!set.has(`${x},${y + 1}`)) {
				ctx.moveTo(x, y + 1);
				ctx.lineTo(x + 1, y + 1);
			}
			if (!set.has(`${x - 1},${y}`)) {
				ctx.moveTo(x, y);
				ctx.lineTo(x, y + 1);
			}
			if (!set.has(`${x + 1},${y}`)) {
				ctx.moveTo(x + 1, y);
				ctx.lineTo(x + 1, y + 1);
			}
		}
		ctx.stroke();
		ctx.restore();
	}

	private drawCore(snap: Snapshot) {
		const relays = snap.cores?.length ? snap.cores : [snap.core];
		const rings = this.reducedFx ? 1 : 3;
		const frac = snap.integrity / Math.max(1, snap.integrityMax);
		for (const c of clusterCells(relays)) {
			drawRelay(this.ctx, c.cx, c.cy, {
				frac,
				time: snap.time,
				rings,
				colors: this.cols,
				scale: c.count > 1 ? 1.22 : 1.08
			});
		}
	}

	private drawWalls(walls: [number, number][]) {
		const { ctx } = this;
		const set = new Set(walls.map(([x, y]) => `${x},${y}`));
		// drawWallCell leaves fillStyle on an opaque plank brown; fence it in so the next
		// painter inherits a clean context rather than the last nail it drew.
		ctx.save();
		for (const [x, y] of walls) {
			drawWallCell(ctx, x, y, this.cols, set);
		}
		ctx.restore();
	}

	private drawWalk(paths: [number, number][][] | undefined) {
		if (!paths?.length) return;
		const { ctx } = this;
		ctx.save();
		ctx.strokeStyle = 'rgba(77, 184, 212, 0.42)';
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
		strike: number
	) {
		if (!h) return;
		const { ctx } = this;
		if (h.strike || strike > 0) {
			ctx.beginPath();
			ctx.arc(h.x + 0.5, h.y + 0.5, Math.max(0.4, h.range), 0, Math.PI * 2);
			ctx.fillStyle = h.valid
				? 'rgba(240, 169, 76, 0.18)'
				: 'rgba(220, 70, 60, 0.16)';
			ctx.fill();
			ctx.strokeStyle = h.valid
				? 'rgba(240, 169, 76, 0.7)'
				: this.cols.invalid;
			ctx.lineWidth = 0.06;
			ctx.stroke();
			return;
		}
		if (build === 0) {
			return;
		}
		ctx.fillStyle = h.valid
			? 'rgba(180, 200, 70, 0.16)'
			: 'rgba(220, 70, 60, 0.16)';
		ctx.fillRect(h.x, h.y, 1, 1);
		ctx.strokeStyle = h.valid
			? this.cols.valid
			: this.cols.invalid;
		ctx.lineWidth = 0.09;
		ctx.strokeRect(h.x + 0.04, h.y + 0.04, 0.92, 0.92);
		const kind = BUILD_BY_ID[build];
		if (kind && kind !== 'inspect') {
			ctx.save();
			ctx.translate(h.x + 0.5, h.y + 0.5);
			ctx.globalAlpha = h.valid ? 0.92 : 0.4;
			if (kind === 'barricade') drawBarricade(ctx, this.cols);
			else drawTurret(ctx, kind, { aim: 0, colors: this.cols });
			ctx.restore();
		}
		if (!h.valid && h.reason) this.drawHoverReason(h);
	}

	/**
	 * The engine already ships the right words ("That would cut off the relay"). Without
	 * this the player gets one undifferentiated red square for occupied / unaffordable /
	 * gun-cap / would-seal-the-relay, and only learns which after clicking.
	 */
	private drawHoverReason(h: NonNullable<Snapshot['hover']>) {
		const { ctx } = this;
		const map = this.map;
		ctx.save();
		ctx.font = '0.4px "Barlow Condensed", sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		const padX = 0.14;
		const w = ctx.measureText(h.reason).width + padX * 2;
		// Flip below the cell in the top rows, and keep the label inside the map.
		const above = h.y > 0;
		const cy = h.y + (above ? -0.34 : 1.34);
		const cx = Math.min(Math.max(h.x + 0.5, w / 2), (map?.w ?? h.x + 1) - w / 2);
		ctx.fillStyle = 'rgba(5, 6, 5, 0.9)';
		ctx.fillRect(cx - w / 2, cy - 0.24, w, 0.48);
		ctx.strokeStyle = 'rgba(232, 116, 138, 0.55)';
		ctx.lineWidth = 0.03;
		ctx.strokeRect(cx - w / 2, cy - 0.24, w, 0.48);
		ctx.fillStyle = '#e8748a';
		ctx.fillText(h.reason, cx, cy + 0.02);
		ctx.restore();
	}

	private drawRange(snap: Snapshot) {
		this.paintRangeRing(
			this.rangeFrom(snap.selected, snap.hover, snap.build, snap.strike, snap.relocating)
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
		src: { cx: number; cy: number; range: number; air: boolean } | null
	) {
		if (!src || src.range <= 0) return;
		const { ctx } = this;
		// Both styles must be set here. Filling with whatever the previous draw left on the
		// context painted the range as an opaque barricade brown over the terrain.
		ctx.save();
		ctx.fillStyle = src.air ? this.cols.rangeAir : this.cols.range;
		ctx.strokeStyle = src.air ? this.cols.rangeAirEdge : this.cols.rangeEdge;
		ctx.beginPath();
		ctx.arc(src.cx, src.cy, src.range, 0, Math.PI * 2);
		ctx.fill();
		ctx.setLineDash([0.18, 0.12]);
		ctx.lineWidth = 0.045;
		ctx.stroke();
		ctx.setLineDash([]);
		ctx.restore();
	}

	private drawTowers(towers: TowerView[], selectedId: number | null) {
		for (const t of towers) {
			this.drawTower(t, t.id === selectedId);
		}
	}

	private drawTower(t: TowerView, selected: boolean) {
		const { ctx } = this;
		const x = t.x + 0.5;
		const y = t.y + 0.5;
		ctx.save();
		ctx.translate(x, y);
		if (selected) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.52, 0, Math.PI * 2);
			ctx.strokeStyle = selected ? 'rgba(255, 196, 110, 0.9)' : 'rgba(169, 140, 240, 0.9)';
			ctx.lineWidth = 0.05;
			ctx.stroke();
		}
		if (t.overcharged) {
			ctx.beginPath();
			ctx.arc(0, 0, 0.5, 0, Math.PI * 2);
			ctx.strokeStyle = 'rgba(120, 230, 255, 0.75)';
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
		drawTurret(ctx, t.kind, {
			aim: t.aim,
			tier: t.tier,
			airFocus: t.airFocus,
			colors: this.cols
		});
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
		paintCreep(ctx, c.kind, {
			heading: c.heading,
			radius: c.radius,
			flying: c.flying,
			colors: this.cols
		});
		if (c.slowed) {
			ctx.strokeStyle = this.cols.slow;
			ctx.lineWidth = 0.05;
			ctx.beginPath();
			ctx.arc(0, 0, c.radius + 0.14, 0, Math.PI * 2);
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
				ctx.strokeStyle = 'rgba(120, 230, 255, 0.85)';
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

