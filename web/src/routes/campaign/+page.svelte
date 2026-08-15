<script lang="ts">
	import { onMount } from 'svelte';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import type { ChallengeInfo, MissionInfo } from '$lib/game/types';
	import { missionUnlocked, readCampaignCleared } from '$lib/game/types';

	let missions = $state<MissionInfo[]>([]);
	let challenges = $state<ChallengeInfo[]>([]);
	let cleared = $state<number[]>([]);
	let coop = $state(false);

	const complete = $derived(missions.length > 0 && missions.every((m) => cleared.includes(m.id)));
	const coopQ = $derived(coop ? '&coop=1' : '');

	onMount(async () => {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		missions = JSON.parse(WasmGame.campaign()) as MissionInfo[];
		challenges = JSON.parse(WasmGame.challenges()) as ChallengeInfo[];
		cleared = readCampaignCleared();
	});
</script>

<main class="brief">
	<section class="brief-hero">
		<div>
			<p class="kicker">Frontier command · Campaign</p>
			<h1>Operations board</h1>
			<p class="lede">
				Eight scripted theaters. Hold the listed wave, then keep going if you want the endless walk.
				Clear a mission to unlock the next. Challenges are known seeds — same orders, same fight.
			</p>
			{#if complete}
				<p class="hazard"><strong>Board clear.</strong> Run it again, or take a challenge seed.</p>
			{/if}
			<div class="actions">
				<a class="btn" href="/">Briefing</a>
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
			<p class="hint">Objectives do not freeze the sim. Advance when you are done, or stay.</p>
		</div>
	</section>
	<aside class="brief-side">
		<div>
			<h2>Missions</h2>
			{#if missions.length === 0}
				<p class="hint">Linking ops…</p>
			{:else}
				<div class="theater-list">
					{#each missions as m (m.id)}
						{@const open = missionUnlocked(m.id, cleared)}
						{@const done = cleared.includes(m.id)}
						{#if open}
							<a
								class="theater"
								class:cleared={done}
								href={`/play?mission=${m.id}${coopQ}`}
							>
								<strong>{m.id + 1}. {m.name}</strong>
								<span>{m.briefing}</span>
								<small>{m.mapName} · {m.modifierName} · {m.objective}</small>
							</a>
						{:else}
							<div class="theater locked">
								<strong>{m.id + 1}. {m.name}</strong>
								<span>Hold the previous theater first.</span>
								<small>{m.mapName} · {m.modifierName}</small>
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
		<div>
			<h2>Challenges</h2>
			<div class="theater-list">
				{#each challenges as c (c.id)}
					<a class="theater" href={`/play?challenge=${c.id}${coopQ}`}>
						<strong>{c.name}</strong>
						<span>{c.blurb}</span>
						<small>
							{c.mapName} · {c.modifierName}
							{#if c.holdUntilWave != null}
								· hold {c.holdUntilWave}
							{/if}
						</small>
					</a>
				{/each}
			</div>
		</div>
	</aside>
</main>
