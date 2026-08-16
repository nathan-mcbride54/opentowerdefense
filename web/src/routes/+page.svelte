<script lang="ts">
	import { onMount } from 'svelte';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import MapThumb from '$lib/game/MapThumb.svelte';
	import MenuChrome from '$lib/game/MenuChrome.svelte';
	import type { DailyPick, MapDoc, ModifierInfo, TheaterInfo } from '$lib/game/types';
	import { readBestWave, utcDay } from '$lib/game/types';

	const OPS_PICK = 'otd-ops-pick';
	const keys = [
		['1–9 / 0', 'Select theater'],
		['Enter', 'Play selected'],
		['1–9 / 0', 'Build (in field)'],
		['Q W E', 'Strikes'],
		['T / C / V / G / B', 'Target / Helios / Repair / Move / Overcharge'],
		['U / X', 'Upgrade / sell'],
		['N', 'Call wave'],
		['Space / F / Esc', 'Pause / speed'],
		['M', 'Mute'],
		['Home', 'Reset view']
	];

	let theaters = $state<TheaterInfo[]>([]);
	let modifiers = $state<ModifierInfo[]>([]);
	let docs = $state<Record<number, MapDoc>>({});
	let daily = $state<DailyPick | null>(null);
	let mapId = $state(0);
	let modId = $state(0);
	let bests = $state<Record<string, number>>({});
	let ready = $state(false);
	let now = $state(Date.now());

	const selectedTheater = $derived(theaters.find((t) => t.id === mapId));
	const selectedMod = $derived(modifiers.find((m) => m.id === modId));
	const selectedDoc = $derived(docs[mapId]);
	const playHref = $derived(`/play?map=${mapId}&mod=${modId}`);
	const dailyHref = $derived(
		daily ? `/play?day=${daily.utcDay}` : `/play?day=${utcDay()}`
	);
	const selectedBest = $derived(bests[`${mapId}-${modId}`] ?? 0);
	const dailyLeft = $derived.by(() => {
		const end = (utcDay(now) + 1) * 86_400_000;
		const ms = Math.max(0, end - now);
		const h = Math.floor(ms / 3_600_000);
		const m = Math.floor((ms % 3_600_000) / 60_000);
		return `${h}h ${String(m).padStart(2, '0')}m`;
	});

	function persist() {
		if (!ready) return;
		try {
			localStorage.setItem(OPS_PICK, JSON.stringify({ mapId, modId }));
		} catch {
			/* ignore */
		}
	}

	onMount(() => {
		const clock = setInterval(() => (now = Date.now()), 15_000);
		const onKey = (e: KeyboardEvent) => {
			// target can be document/window, which have no closest().
			const el = e.target instanceof Element ? e.target : null;
			if (el && (el.closest('input, textarea, select, [contenteditable]') || el.closest('[data-rebind]')))
				return;
			if (e.key >= '1' && e.key <= '9') {
				const t = theaters[Number(e.key) - 1];
				if (t) mapId = t.id;
			} else if (e.key === '0') {
				const t = theaters[9];
				if (t) mapId = t.id;
			}
			// Enter must not hijack keyboard activation of a focused control — otherwise
			// tabbing to Settings and pressing Enter deploys into a match instead.
			if (
				e.key === 'Enter' &&
				!e.repeat &&
				selectedTheater &&
				!(el instanceof Element && el.closest('button, a, summary, [role="button"], details'))
			) {
				window.location.href = `/play?map=${mapId}&mod=${modId}`;
			}
		};
		window.addEventListener('keydown', onKey);
		void (async () => {
			const { default: init, WasmGame } = await import('$lib/wasm/otd');
			await init();
			theaters = JSON.parse(WasmGame.theaters()) as TheaterInfo[];
			modifiers = JSON.parse(WasmGame.modifiers()) as ModifierInfo[];
			daily = JSON.parse(WasmGame.daily(utcDay())) as DailyPick;
			const loaded: Record<number, MapDoc> = {};
			for (const t of theaters) {
				loaded[t.id] = JSON.parse(WasmGame.theaterDoc(t.id)) as MapDoc;
			}
			docs = loaded;
			const next: Record<string, number> = {};
			for (const t of theaters) {
				for (const m of modifiers) {
					next[`${t.id}-${m.id}`] = readBestWave(t.id, m.id);
				}
			}
			bests = next;
			try {
				const raw = localStorage.getItem(OPS_PICK);
				if (raw) {
					const pick = JSON.parse(raw) as { mapId?: number; modId?: number };
					if (theaters.some((t) => t.id === pick.mapId)) mapId = pick.mapId as number;
					if (modifiers.some((m) => m.id === pick.modId)) modId = pick.modId as number;
				}
			} catch {
				/* ignore */
			}
			ready = true;
		})();
		return () => {
			clearInterval(clock);
			window.removeEventListener('keydown', onKey);
		};
	});

	$effect(() => {
		mapId;
		modId;
		persist();
	});
</script>

<main class="war-menu">
	<MenuChrome titleMark="Open" title="Tower Defense" current="maps">
		{#snippet actions()}
			{#if daily}
				<a class="btn daily" href={dailyHref}>Daily</a>
			{/if}
		{/snippet}
	</MenuChrome>
	<section class="map-stage">
		{#if selectedDoc && selectedTheater}
			<div class="map-hero map-stage-hero">
				<MapThumb map={selectedDoc} large />
				<div class="map-hero-hud">
					<div>
						<p class="kicker">Theater {String(selectedTheater.id + 1).padStart(2, '0')}</p>
						<h2>{selectedTheater.name}</h2>
						<p>{selectedTheater.hazard}</p>
						<ul class="map-stats">
							<li><b>{selectedDoc.w}×{selectedDoc.h}</b> grid</li>
							<li><b>{selectedDoc.cores.length}</b> relay</li>
							<li><b>{selectedDoc.spawns.length}</b> ingress</li>
							<li><b>{selectedDoc.rocks.length}</b> rock</li>
							{#if selectedMod}
								<li><b>{selectedMod.name}</b></li>
							{/if}
							{#if selectedBest}
								<li><b>wave {selectedBest}</b> best</li>
							{/if}
						</ul>
					</div>
					<div class="map-hero-actions">
						<a class="btn primary deploy" href={playHref}>Play</a>
					</div>
				</div>
			</div>
		{:else}
			<p class="hint">Linking theaters…</p>
		{/if}
	</section>
	<aside class="menu-side">
		<div>
			<h2>Select map</h2>
			<p class="hint side-hint">Keys 1–9 and 0 pick a theater. Enter deploys.</p>
			{#if theaters.length === 0}
				<p class="hint">Linking theaters…</p>
			{:else}
				<div class="theater-list">
					{#each theaters as t, i (t.id)}
						<button
							type="button"
							class="theater"
							class:selected={t.id === mapId}
							onclick={() => (mapId = t.id)}
						>
							{#if docs[t.id]}
								<MapThumb map={docs[t.id]} />
							{/if}
							{#if bests[`${t.id}-${modId}`]}
								<span class="best-chip">W{bests[`${t.id}-${modId}`]}</span>
							{/if}
							<span class="theater-copy">
								<!-- One number per card, reading 1–10 in list order. The key that
								     selects it is 0 for the tenth, so that lives in the tooltip
								     rather than as a second, conflicting number on the card. -->
								<span class="theater-no" title="Press {i === 9 ? 0 : i + 1}">
									{String(i + 1).padStart(2, '0')}
								</span>
								<strong>{t.name}</strong>
								{#if docs[t.id]}
									<small>{docs[t.id].w}×{docs[t.id].h}</small>
								{/if}
							</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>
		<div>
			<h2>Game mode</h2>
			<div class="mod-list">
				{#each modifiers as m (m.id)}
					<button
						type="button"
						class="mod"
						class:selected={m.id === modId}
						onclick={() => (modId = m.id)}
						title={m.hazard}
					>
						{m.name}
					</button>
				{/each}
			</div>
			{#if selectedMod}
				<p class="hint">{selectedMod.blurb}</p>
			{/if}
		</div>
		{#if daily}
			<div class="daily-card">
				<p class="kicker">Today’s assignment</p>
				<strong>{daily.mapName} · {daily.modifierName}</strong>
				<span>{daily.mapHazard} {daily.modifierHazard}</span>
				<span class="daily-clock">Rotates in {dailyLeft}</span>
				<a class="btn daily" href={dailyHref}>Run daily</a>
			</div>
		{/if}
	</aside>
	<footer class="menu-foot">
		<SettingsDock compact />
		<details class="orders">
			<summary>Standing orders / hotkeys</summary>
			<div class="orders-body">
				<ol>
					<li>Barricades and turrets both block ground traffic. Stretch the walk.</li>
					<li>Twin Cores: air still hunts the nearest relay. You cannot abandon the sky.</li>
					<li>Fixed scrap and gun caps are vows. Sell-back is the only refund.</li>
					<li>Workshop maps are JSON. Validate before you deploy. Copy a replay from pause.</li>
					<li>Campaign missions unlock in order. Challenges are public seeds with a verify hash.</li>
					<li>The director names the next wave. Swarm wants splash. Split wants mites dead in the fold.</li>
					<li>Medics heal the column. Interest pays on leftover scrap when the field clears.</li>
					<li>Shades ignore guns without Det. Pulse, Arc, Helios, and strikes can see them.</li>
					<li>The Walk meter is the maze. Move a structure with G. Flickers hop the path.</li>
				</ol>
				<div class="keys">
					{#each keys as [k, v]}
						<kbd>{k}</kbd>
						<span>{v}</span>
					{/each}
				</div>
			</div>
		</details>
		<p class="hint">MIT licensed.</p>
	</footer>
</main>
