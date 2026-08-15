<script lang="ts">
	import { onMount } from 'svelte';
	import { PACK_STORAGE } from '$lib/game/session';
	import type { FireMode, PackDoc } from '$lib/game/types';

	const fires: FireMode[] = ['shell', 'cone', 'line', 'pulse', 'beam'];

	let pack = $state<PackDoc>({
		slug: 'stock',
		name: 'Stock',
		blurb: 'Default frontier loadout.',
		guns: [],
		strikes: []
	});
	let presets = $state<PackDoc[]>([]);
	let report = $state('Load a preset or edit the numbers. Validate before you deploy.');
	let ok = $state(false);
	let jsonText = $state('');
	let ready = $state(false);

	function syncJson() {
		jsonText = JSON.stringify(pack, null, 2);
	}

	function loadResolved(next: PackDoc) {
		pack = next;
		ok = false;
		syncJson();
	}

	async function wasm() {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		return WasmGame;
	}

	async function applyRaw(raw: string) {
		const WasmGame = await wasm();
		const r = JSON.parse(WasmGame.resolvePack(raw)) as {
			ok: boolean;
			pack?: PackDoc;
			error?: string;
		};
		if (!r.ok || !r.pack) {
			ok = false;
			report = r.error || 'Rejected.';
			return;
		}
		loadResolved(r.pack);
		report = `${r.pack.name} resolved. ${r.pack.guns.length} guns, ${r.pack.strikes.length} strikes.`;
	}

	async function loadPreset(p: PackDoc) {
		await applyRaw(JSON.stringify(p));
	}

	async function validate() {
		const WasmGame = await wasm();
		const r = JSON.parse(WasmGame.validatePack(JSON.stringify(pack))) as {
			ok: boolean;
			error?: string;
			guns?: number;
			stock?: boolean;
		};
		ok = r.ok;
		report = r.ok
			? `Ready. ${r.guns} guns in tray${r.stock ? ' · stock numbers' : ''}.`
			: r.error || 'Rejected.';
		if (r.ok) await applyRaw(JSON.stringify(pack));
	}

	function play() {
		sessionStorage.setItem(PACK_STORAGE, JSON.stringify(pack));
		window.location.href = '/play?pack=1';
	}

	function copyJson() {
		void navigator.clipboard.writeText(JSON.stringify(pack, null, 2));
		report = 'JSON copied.';
	}

	function parseJson() {
		void applyRaw(jsonText);
	}

	onMount(() => {
		void (async () => {
			const WasmGame = await wasm();
			presets = JSON.parse(WasmGame.packPresets()) as PackDoc[];
			const saved = sessionStorage.getItem(PACK_STORAGE);
			if (saved) {
				await applyRaw(saved);
			} else {
				await applyRaw(WasmGame.stockPack());
			}
			ready = true;
		})();
	});
</script>

<main class="workshop">
	<header class="topbar">
		<a class="btn" href="/">Briefing</a>
		<span class="map-chip">Loadout probe</span>
		<div class="top-actions">
			<button type="button" onclick={() => validate()}>Validate</button>
			<button type="button" class="primary" onclick={() => play()} disabled={!ok}>Deploy</button>
			<button type="button" onclick={() => copyJson()}>Copy JSON</button>
			<a class="btn" href="/workshop">Map probe</a>
		</div>
	</header>
	<div class="probe pack-probe">
		<aside class="probe-side">
			<label>Name <input bind:value={pack.name} oninput={() => (ok = false)} /></label>
			<label>Slug <input bind:value={pack.slug} oninput={() => (ok = false)} /></label>
			<label>Blurb <input bind:value={pack.blurb} oninput={() => (ok = false)} /></label>
			<p class="hint">Presets</p>
			<div class="brushes">
				{#each presets as p (p.slug)}
					<button type="button" onclick={() => loadPreset(p)}>{p.name}</button>
				{/each}
			</div>
			<p class="hint">{report}</p>
			<p class="hint">
				Numbers only. Fire modes already exist in the tick. You cannot add an eleventh gun. Disabled
				slots leave the tray.
			</p>
			<label>
				JSON
				<textarea class="pack-json" bind:value={jsonText} rows="8"></textarea>
			</label>
			<button type="button" onclick={() => parseJson()}>Apply JSON</button>
		</aside>
		<div class="probe-stage pack-stage">
			{#if !ready}
				<p class="hint">Linking catalog…</p>
			{:else}
				<table class="pack-table">
					<thead>
						<tr>
							<th>On</th>
							<th>Gun</th>
							<th>Cost</th>
							<th>Range</th>
							<th>Dmg</th>
							<th>Int</th>
							<th>Splash</th>
							<th>Fire</th>
							<th>G</th>
							<th>A</th>
						</tr>
					</thead>
					<tbody>
						{#each pack.guns as g, i (g.id)}
							<tr>
								<td>
									<input
										type="checkbox"
										checked={g.enabled !== false}
										onchange={(e) => {
											pack.guns[i].enabled = e.currentTarget.checked;
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										class="pack-name"
										bind:value={g.name}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										bind:value={g.cost}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.05"
										bind:value={g.range}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.1"
										bind:value={g.damage}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.01"
										bind:value={g.fireInterval}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.05"
										bind:value={g.splash}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<select
										bind:value={g.fire}
										onchange={() => {
											ok = false;
											syncJson();
										}}
									>
										{#each fires as f}
											<option value={f}>{f}</option>
										{/each}
									</select>
								</td>
								<td>
									<input
										type="checkbox"
										checked={!!g.hitsGround}
										onchange={(e) => {
											pack.guns[i].hitsGround = e.currentTarget.checked;
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="checkbox"
										checked={!!g.hitsAir}
										onchange={(e) => {
											pack.guns[i].hitsAir = e.currentTarget.checked;
											ok = false;
											syncJson();
										}}
									/>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<table class="pack-table">
					<thead>
						<tr>
							<th>On</th>
							<th>Strike</th>
							<th>Cost</th>
							<th>Radius</th>
							<th>Dmg</th>
							<th>CD</th>
						</tr>
					</thead>
					<tbody>
						{#each pack.strikes as s, i (s.id)}
							<tr>
								<td>
									<input
										type="checkbox"
										checked={s.enabled !== false}
										onchange={(e) => {
											pack.strikes[i].enabled = e.currentTarget.checked;
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										class="pack-name"
										bind:value={s.name}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										bind:value={s.cost}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.05"
										bind:value={s.radius}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.1"
										bind:value={s.damage}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
								<td>
									<input
										type="number"
										step="0.05"
										bind:value={s.cooldown}
										oninput={() => {
											ok = false;
											syncJson();
										}}
									/>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>
	</div>
</main>
