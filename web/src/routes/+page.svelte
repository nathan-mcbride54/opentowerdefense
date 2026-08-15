<script lang="ts">
	import { onMount } from 'svelte';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import type { DailyPick, ModifierInfo, TheaterInfo } from '$lib/game/types';
	import { readBestWave, utcDay } from '$lib/game/types';

	const keys = [
		['1–9 / 0', 'Build'],
		['Q W E', 'Strikes'],
		['T / C / V / G / B', 'Target / Helios / Repair / Move / Overcharge'],
		['U / X', 'Upgrade / sell'],
		['N', 'Call wave'],
		['Space / Esc', 'Pause'],
		['M', 'Mute'],
		['Home', 'Reset view']
	];

	let theaters = $state<TheaterInfo[]>([]);
	let modifiers = $state<ModifierInfo[]>([]);
	let daily = $state<DailyPick | null>(null);
	let mapId = $state(0);
	let modId = $state(0);
	let coop = $state(false);
	let bests = $state<Record<string, number>>({});

	const selectedTheater = $derived(theaters.find((t) => t.id === mapId));
	const selectedMod = $derived(modifiers.find((m) => m.id === modId));
	const playHref = $derived(`/play?map=${mapId}&mod=${modId}${coop ? '&coop=1' : ''}`);
	const dailyHref = $derived(
		daily ? `/play?day=${daily.utcDay}${coop ? '&coop=1' : ''}` : `/play?day=${utcDay()}${coop ? '&coop=1' : ''}`
	);
	const selectedBest = $derived(bests[`${mapId}-${modId}`] ?? 0);

	onMount(async () => {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		theaters = JSON.parse(WasmGame.theaters()) as TheaterInfo[];
		modifiers = JSON.parse(WasmGame.modifiers()) as ModifierInfo[];
		daily = JSON.parse(WasmGame.daily(utcDay())) as DailyPick;
		const next: Record<string, number> = {};
		for (const t of theaters) {
			for (const m of modifiers) {
				next[`${t.id}-${m.id}`] = readBestWave(t.id, m.id);
			}
		}
		bests = next;
	});
</script>

<main class="brief">
	<section class="brief-hero">
		<div>
			<p class="kicker">Frontier command · 1.0</p>
			<h1>Open Tower Defense</h1>
			<p class="lede">
				Eight theaters, an eight-mission campaign, known-seed challenges, and catalog packs. Paint a
				map or retune the guns. Local co-op shares one maze and one scrap pile. Waves tell you what
				is coming. Leftover scrap pays interest. Drag to pan, pinch or scroll to zoom. Rebind every
				P1 key. The maze is still yours.
			</p>
			{#if selectedTheater && selectedMod}
				<p class="hazard">
					<strong>{selectedTheater.name}</strong>
					· {selectedTheater.hazard}
					<br />
					<strong>{selectedMod.name}</strong>
					· {selectedMod.hazard}
					{#if selectedBest}
						· best wave {selectedBest}
					{/if}
				</p>
			{/if}
			<div class="actions">
				<a class="btn primary" href={playHref}>Deploy</a>
				<a class="btn" href="/campaign">Campaign</a>
				{#if daily}
					<a class="btn" href={dailyHref}>Today · {daily.mapName}</a>
				{/if}
				<a class="btn" href="/workshop">Map probe</a>
				<a class="btn" href="/pack">Loadout</a>
				<a class="btn" href="/replay">Replay</a>
				<label class="coop-opt">
					<input type="checkbox" bind:checked={coop} />
					Co-op (local)
				</label>
			</div>
		</div>
		<div class="hero-foot">
			<SettingsDock compact />
			<p class="hint">MIT licensed. Simulation in Rust/Wasm. Command post in SvelteKit.</p>
		</div>
	</section>
	<aside class="brief-side">
		<div>
			<h2>Theaters</h2>
			{#if theaters.length === 0}
				<p class="hint">Linking theaters…</p>
			{:else}
				<div class="theater-list">
					{#each theaters as t (t.id)}
						<button
							type="button"
							class="theater"
							class:selected={t.id === mapId}
							onclick={() => (mapId = t.id)}
						>
							<strong>{t.name}</strong>
							<span>{t.blurb}</span>
							<small>{t.hazard}</small>
						</button>
					{/each}
				</div>
			{/if}
		</div>
		<div>
			<h2>Modifiers</h2>
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
				<a class="btn" href={dailyHref}>Run daily</a>
			</div>
		{/if}
		<div>
			<h2>Standing orders</h2>
			<ol>
				<li>Barricades and turrets both block ground traffic. Stretch the walk.</li>
				<li>Twin Cores: air still hunts the nearest relay. You cannot abandon the sky.</li>
				<li>Fixed scrap and gun caps are vows. Sell-back is the only refund.</li>
				<li>Workshop maps are JSON. Validate before you deploy. Copy a replay from pause.</li>
				<li>Campaign missions unlock in order. Challenges are public seeds with a verify hash.</li>
				<li>Loadout packs retune the catalog. The tick does not change.</li>
				<li>Local co-op: one sim, shared scrap. P2 is arrows + Enter and a second keymap.</li>
				<li>Copy a replay from pause. Verify or watch it on the replay desk.</li>
				<li>The director names the next wave. Swarm wants splash. Split wants mites dead in the fold.</li>
				<li>Medics heal the column. Interest pays on leftover scrap when the field clears.</li>
				<li>Shades ignore guns without Det. Pulse, Arc, Helios, and strikes can see them. Drag to paint walls.</li>
				<li>The Walk meter is the maze. Move a structure with G. Flickers hop the path — length is not a win by itself.</li>
			</ol>
		</div>
		<div>
			<h2>Hotkeys</h2>
			<div class="keys">
				{#each keys as [k, v]}
					<kbd>{k}</kbd>
					<span>{v}</span>
				{/each}
			</div>
		</div>
	</aside>
</main>
