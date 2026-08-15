<script lang="ts">
	import { onMount } from 'svelte';
	import { WORKSHOP_STORAGE } from '$lib/game/session';
	import type { MapDoc, TheaterInfo } from '$lib/game/types';

	type Brush = 'empty' | 'rock' | 'spawn' | 'core';

	let canvas: HTMLCanvasElement | undefined = $state();
	let theaters = $state<TheaterInfo[]>([]);
	let name = $state('Workshop Yard');
	let slug = $state('workshop');
	let blurb = $state('Custom theater.');
	let hazard = $state('Untested. Validate before you trust it.');
	let w = $state(32);
	let h = $state(20);
	let seed = $state(1);
	let brush = $state<Brush>('rock');
	let cells = $state<Brush[][]>([]);
	let report = $state<string>('Paint a relay and an ingress, then validate.');
	let ok = $state(false);
	let painting = false;

	const colors: Record<Brush, string> = {
		empty: '#2a3322',
		rock: '#6a6556',
		spawn: '#c45c3a',
		core: '#4ee0d8'
	};

	function blank(nw: number, nh: number, keep?: Brush[][]) {
		const next: Brush[][] = [];
		for (let y = 0; y < nh; y++) {
			next[y] = [];
			for (let x = 0; x < nw; x++) {
				next[y][x] = keep?.[y]?.[x] ?? 'empty';
			}
		}
		return next;
	}

	function doc(): MapDoc {
		const cores: [number, number][] = [];
		const spawns: [number, number][] = [];
		const rocks: [number, number][] = [];
		for (let y = 0; y < h; y++) {
			for (let x = 0; x < w; x++) {
				const t = cells[y]?.[x] ?? 'empty';
				if (t === 'core') cores.push([x, y]);
				else if (t === 'spawn') spawns.push([x, y]);
				else if (t === 'rock') rocks.push([x, y]);
			}
		}
		return { slug, name, blurb, hazard, w, h, seed, cores, spawns, rocks };
	}

	function loadDoc(d: MapDoc) {
		name = d.name;
		slug = d.slug || 'workshop';
		blurb = d.blurb || '';
		hazard = d.hazard || '';
		w = d.w;
		h = d.h;
		seed = d.seed || 1;
		const next = blank(d.w, d.h);
		for (const [x, y] of d.cores) if (next[y]) next[y][x] = 'core';
		for (const [x, y] of d.spawns) if (next[y]) next[y][x] = 'spawn';
		for (const [x, y] of d.rocks) if (next[y]) next[y][x] = 'rock';
		cells = next;
		ok = false;
		paint();
	}

	function paint() {
		const c = canvas;
		if (!c) return;
		const ctx = c.getContext('2d');
		if (!ctx) return;
		const cssW = Math.max(1, c.clientWidth);
		const cssH = Math.max(1, c.clientHeight);
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		c.width = Math.floor(cssW * dpr);
		c.height = Math.floor(cssH * dpr);
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		const scale = Math.min(cssW / w, cssH / h);
		const ox = (cssW - w * scale) / 2;
		const oy = (cssH - h * scale) / 2;
		ctx.fillStyle = '#0c100b';
		ctx.fillRect(0, 0, cssW, cssH);
		ctx.translate(ox, oy);
		ctx.scale(scale, scale);
		for (let y = 0; y < h; y++) {
			for (let x = 0; x < w; x++) {
				ctx.fillStyle = colors[cells[y]?.[x] ?? 'empty'];
				ctx.fillRect(x, y, 1, 1);
			}
		}
		ctx.strokeStyle = 'rgba(196, 210, 160, 0.12)';
		ctx.lineWidth = 0.03;
		ctx.beginPath();
		for (let x = 0; x <= w; x++) {
			ctx.moveTo(x, 0);
			ctx.lineTo(x, h);
		}
		for (let y = 0; y <= h; y++) {
			ctx.moveTo(0, y);
			ctx.lineTo(w, y);
		}
		ctx.stroke();
	}

	function cellAt(ev: PointerEvent) {
		if (!canvas) return null;
		const rect = canvas.getBoundingClientRect();
		const scale = Math.min(rect.width / w, rect.height / h);
		const ox = (rect.width - w * scale) / 2;
		const oy = (rect.height - h * scale) / 2;
		const x = Math.floor((ev.clientX - rect.left - ox) / scale);
		const y = Math.floor((ev.clientY - rect.top - oy) / scale);
		if (x < 0 || y < 0 || x >= w || y >= h) return null;
		return { x, y };
	}

	function stamp(ev: PointerEvent) {
		const cell = cellAt(ev);
		if (!cell) return;
		const next = cells.map((row) => row.slice());
		next[cell.y][cell.x] = brush;
		cells = next;
		ok = false;
		paint();
	}

	async function validate() {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		const r = JSON.parse(WasmGame.validateMap(JSON.stringify(doc()))) as {
			ok: boolean;
			error?: string;
			name?: string;
			cores?: number;
			spawns?: number;
		};
		ok = r.ok;
		report = r.ok
			? `Open. ${r.cores} core cells, ${r.spawns} spawns. Safe to deploy.`
			: r.error || 'Rejected.';
	}

	function play() {
		sessionStorage.setItem(WORKSHOP_STORAGE, JSON.stringify(doc()));
		window.location.href = '/play?workshop=1';
	}

	function copyJson() {
		void navigator.clipboard.writeText(JSON.stringify(doc(), null, 2));
		report = 'JSON copied.';
	}

	onMount(() => {
		cells = blank(w, h);
		const ro = new ResizeObserver(() => paint());
		if (canvas) ro.observe(canvas);
		void (async () => {
			const { default: init, WasmGame } = await import('$lib/wasm/otd');
			await init();
			theaters = JSON.parse(WasmGame.theaters()) as TheaterInfo[];
			const saved = sessionStorage.getItem(WORKSHOP_STORAGE);
			if (saved) {
				try {
					loadDoc(JSON.parse(saved) as MapDoc);
				} catch {
					paint();
				}
			} else {
				paint();
			}
		})();
		return () => ro.disconnect();
	});

	async function loadTheater(id: number) {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		const d = JSON.parse(WasmGame.theaterDoc(id)) as MapDoc;
		if (d.w) loadDoc(d);
	}

	function resizeGrid() {
		w = Math.min(64, Math.max(8, w));
		h = Math.min(48, Math.max(8, h));
		cells = blank(w, h, cells);
		ok = false;
		paint();
	}
</script>

<main class="workshop">
	<header class="topbar">
		<a class="btn" href="/">Briefing</a>
		<span class="map-chip">Map probe</span>
		<div class="top-actions">
			<button type="button" onclick={() => validate()}>Validate</button>
			<button type="button" class="primary" onclick={() => play()} disabled={!ok}>Deploy</button>
			<button type="button" onclick={() => copyJson()}>Copy JSON</button>
			<a class="btn" href="/pack">Loadout</a>
		</div>
	</header>
	<div class="probe">
		<aside class="probe-side">
			<label>Name <input bind:value={name} /></label>
			<label>Slug <input bind:value={slug} /></label>
			<label>Seed <input type="number" bind:value={seed} /></label>
			<div class="row">
				<label>W <input type="number" min="8" max="64" bind:value={w} onchange={resizeGrid} /></label>
				<label>H <input type="number" min="8" max="48" bind:value={h} onchange={resizeGrid} /></label>
			</div>
			<p class="hint">Brush</p>
			<div class="brushes">
				{#each ['empty', 'rock', 'spawn', 'core'] as b}
					<button type="button" class:active={brush === b} onclick={() => (brush = b as Brush)}>
						{b}
					</button>
				{/each}
			</div>
			<p class="hint">Start from a theater</p>
			<div class="brushes">
				{#each theaters as t}
					<button type="button" onclick={() => loadTheater(t.id)}>{t.name}</button>
				{/each}
			</div>
			<p class="hint">{report}</p>
			<p class="hint">
				Cores, spawns, and rocks cannot overlap. Every spawn must walk to a relay. Simulation still
				owns the rule — this page only paints.
			</p>
		</aside>
		<div class="probe-stage">
			<canvas
				bind:this={canvas}
				onpointerdown={(e) => {
					painting = true;
					canvas?.setPointerCapture(e.pointerId);
					stamp(e);
				}}
				onpointermove={(e) => {
					if (painting) stamp(e);
				}}
				onpointerup={() => (painting = false)}
				onpointercancel={() => (painting = false)}
			></canvas>
		</div>
	</div>
</main>
