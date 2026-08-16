<script lang="ts">
	import { onMount } from 'svelte';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import MapThumb from '$lib/game/MapThumb.svelte';
	import MenuChrome from '$lib/game/MenuChrome.svelte';
	import type { ChallengeInfo, MapDoc, MissionInfo } from '$lib/game/types';
	import { missionUnlocked, readCampaignCleared } from '$lib/game/types';

	let missions = $state<MissionInfo[]>([]);
	let challenges = $state<ChallengeInfo[]>([]);
	let docs = $state<Record<number, MapDoc>>({});
	let cleared = $state<number[]>([]);

	const complete = $derived(missions.length > 0 && missions.every((m) => cleared.includes(m.id)));
	const held = $derived(missions.filter((m) => cleared.includes(m.id)).length);
	const pct = $derived(missions.length ? Math.round((held / missions.length) * 100) : 0);

	onMount(async () => {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		missions = JSON.parse(WasmGame.campaign()) as MissionInfo[];
		challenges = JSON.parse(WasmGame.challenges()) as ChallengeInfo[];
		cleared = readCampaignCleared();
		const loaded: Record<number, MapDoc> = {};
		const ids = new Set([...missions.map((m) => m.mapId), ...challenges.map((c) => c.mapId)]);
		for (const id of ids) {
			loaded[id] = JSON.parse(WasmGame.theaterDoc(id)) as MapDoc;
		}
		docs = loaded;
	});
</script>

<main class="war-menu campaign-menu">
	<MenuChrome titleMark="Campaign" title="Operations" current="campaign">
		{#snippet lead()}
			<div class="op-progress" role="status">
				<span>{held} / {missions.length || '—'} theaters held</span>
				<div class="op-bar" aria-hidden="true"><i style="width: {pct}%"></i></div>
			</div>
			{#if complete}
				<p class="hazard"><strong>Board clear.</strong> Run it again, or take a challenge seed.</p>
			{/if}
		{/snippet}
	</MenuChrome>
	<section class="campaign-board">
		<div>
			<h2>Missions</h2>
			{#if missions.length === 0}
				<p class="hint">Linking ops…</p>
			{:else}
				<div class="theater-list campaign-grid">
					{#each missions as m (m.id)}
						{@const open = missionUnlocked(m.id, cleared)}
						{@const done = cleared.includes(m.id)}
						{#if open}
							<a
								class="theater"
								class:cleared={done}
								href={`/play?mission=${m.id}`}
							>
								{#if docs[m.mapId]}
									<MapThumb map={docs[m.mapId]} />
								{/if}
								<span class="stamp" class:held={done}>{done ? 'Held' : 'Open'}</span>
								<span class="theater-copy">
									<span class="dossier-id">{String(m.id + 1).padStart(2, '0')}</span>
									<strong>{m.name}</strong>
									<small>{m.mapName} · {m.modifierName} · {m.objective}</small>
								</span>
							</a>
						{:else}
							<div class="theater locked">
								{#if docs[m.mapId]}
									<MapThumb map={docs[m.mapId]} />
								{/if}
								<span class="stamp sealed">Sealed</span>
								<span class="theater-copy">
									<span class="dossier-id">{String(m.id + 1).padStart(2, '0')}</span>
									<strong>{m.name}</strong>
									<small>Hold the previous theater first.</small>
								</span>
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
		<div>
			<h2>Challenges</h2>
			<p class="hint">Public seeds. Verify the hash after you walk off the field.</p>
			<div class="theater-list campaign-grid">
				{#each challenges as c (c.id)}
					<a class="theater" href={`/play?challenge=${c.id}`}>
						{#if docs[c.mapId]}
							<MapThumb map={docs[c.mapId]} />
						{/if}
						<span class="stamp seed">Seed</span>
						<span class="theater-copy">
							<span class="dossier-id">{String(c.id + 1).padStart(2, '0')}</span>
							<strong>{c.name}</strong>
							<small>
								{c.mapName} · {c.modifierName}
								{#if c.holdUntilWave != null}
									· hold {c.holdUntilWave}
								{/if}
							</small>
						</span>
					</a>
				{/each}
			</div>
		</div>
	</section>
	<footer class="menu-foot">
		<SettingsDock compact />
		<p class="hint">Objectives do not freeze the sim. Advance when you are done, or stay.</p>
	</footer>
</main>
