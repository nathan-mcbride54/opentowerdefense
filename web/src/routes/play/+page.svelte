<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { createSession, PACK_STORAGE, REPLAY_STORAGE, WORKSHOP_STORAGE, type Session, type SessionOpts } from '$lib/game/session';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import { formatKey, P2_KEYS, type ActionId } from '$lib/game/keys';
	import { loadSettings, subscribeSettings } from '$lib/game/settings';
	import {
		markCampaignCleared,
		readBestWave,
		type CatalogItem,
		type KindCount,
		type SelectedInfo,
		type Snapshot,
		type Speed,
		type StrikeItem
	} from '$lib/game/types';

	let canvas: HTMLCanvasElement | undefined = $state();
	let session: Session | null = $state(null);
	let error = $state<string | null>(null);
	let snap = $state<Snapshot | null>(null);
	let catalog = $state<CatalogItem[]>([]);
	let strikeItems = $state<StrikeItem[]>([]);
	let paused = $state(false);
	let speed = $state<Speed>(1);
	let best = $state(0);
	let keys = $state(loadSettings().keys);
	let heldOpen = $state(false);
	let heldRecorded = false;
	let coop = $state(false);
	let watch = $state(false);

	const params = page.url.searchParams;
	const mapId = Number(params.get('map') ?? '0') || 0;
	const modId = Number(params.get('mod') ?? '0') || 0;
	const dayRaw = params.get('day');
	const utcDay = dayRaw != null && dayRaw !== '' ? Number(dayRaw) : null;
	const workshop = params.get('workshop') === '1';
	const missionRaw = params.get('mission');
	const challengeRaw = params.get('challenge');
	const seedRaw = params.get('seed');
	const missionId = missionRaw != null && missionRaw !== '' ? Number(missionRaw) : null;
	const challengeId = challengeRaw != null && challengeRaw !== '' ? Number(challengeRaw) : null;
	const seedHex = seedRaw && seedRaw !== '' ? seedRaw : null;
	const usePack = params.get('pack') === '1';
	const coopParam = params.get('coop') === '1';
	const replayParam = params.get('replay') === '1';
	const retryHref = (() => {
		if (replayParam) return '/play?replay=1';
		const extra = `${usePack ? '&pack=1' : ''}${coopParam ? '&coop=1' : ''}`;
		if (workshop) return `/play?workshop=1${extra}`;
		if (missionId != null && !Number.isNaN(missionId)) return `/play?mission=${missionId}${coopParam ? '&coop=1' : ''}`;
		if (challengeId != null && !Number.isNaN(challengeId))
			return `/play?challenge=${challengeId}${coopParam ? '&coop=1' : ''}`;
		if (utcDay != null && !Number.isNaN(utcDay)) return `/play?day=${utcDay}${extra}`;
		if (seedHex) return `/play?map=${mapId}&mod=${modId}&seed=${encodeURIComponent(seedHex)}${extra}`;
		return `/play?map=${mapId}&mod=${modId}${extra}`;
	})();
	const nextMissionHref = $derived(
		snap?.missionId != null && snap.missionId < 7
			? `/play?mission=${snap.missionId + 1}${coopParam ? '&coop=1' : ''}`
			: '/campaign'
	);
	const campaignDone = $derived(snap?.missionId === 7 && snap.objectiveCleared);

	function bindLabel(id: ActionId) {
		return formatKey(keys[id]);
	}

	function p2Bind(id: ActionId) {
		return formatKey(P2_KEYS[id]);
	}

	function buildBind(id: number): ActionId {
		return (id === 10 ? 'build10' : `build${id}`) as ActionId;
	}

	function copyReplay() {
		if (!session) return;
		void navigator.clipboard.writeText(session.replayJson());
	}

	function formatParts(parts: KindCount[] | undefined) {
		if (!parts?.length) return '';
		return parts.map((p) => `${p.count} ${p.name}`).join(' · ');
	}

	onMount(() => {
		if (!canvas) return;
		const unsub = subscribeSettings(() => {
			keys = loadSettings().keys;
		});
		best = readBestWave(mapId, modId);
		let active: Session | null = null;
		let primed = false;
		const mapJson = workshop ? sessionStorage.getItem(WORKSHOP_STORAGE) : null;
		const packJson = usePack ? sessionStorage.getItem(PACK_STORAGE) : null;
		const replayJson = replayParam ? sessionStorage.getItem(REPLAY_STORAGE) : null;
		if (workshop && !mapJson) {
			error = 'No workshop map in this tab. Open the map probe first.';
			return () => unsub();
		}
		if (usePack && !packJson) {
			error = 'No loadout in this tab. Open the pack probe first.';
			return () => unsub();
		}
		if (replayParam && !replayJson) {
			error = 'No replay in this tab. Open the replay desk first.';
			return () => unsub();
		}
		if (missionRaw != null && missionRaw !== '') {
			if (missionId == null || Number.isNaN(missionId) || !Number.isInteger(missionId) || missionId < 0) {
				error = 'Unknown mission.';
				return () => unsub();
			}
		}
		if (challengeRaw != null && challengeRaw !== '') {
			if (
				challengeId == null ||
				Number.isNaN(challengeId) ||
				!Number.isInteger(challengeId) ||
				challengeId < 0
			) {
				error = 'Unknown challenge.';
				return () => unsub();
			}
		}
		const withPack = (opts: SessionOpts) => {
			const next = { ...opts, coop: coopParam && !replayParam, replayJson: replayJson ?? undefined };
			return packJson && missionId == null && challengeId == null && !replayParam
				? { ...next, packJson }
				: next;
		};
		createSession(
			canvas,
			(next, extras) => {
				snap = next;
				catalog = extras.catalog;
				strikeItems = extras.strikes;
				paused = extras.paused;
				speed = extras.speed;
				coop = extras.coop;
				watch = extras.watch;
				if (!primed) {
					primed = true;
					if (!extras.watch) best = readBestWave(next.mapId, next.modifierId ?? 0);
				}
				if (next.defeated && !extras.watch)
					best = Math.max(best, readBestWave(next.mapId, next.modifierId ?? 0));
				if (next.objectiveCleared && !heldRecorded && !extras.watch) {
					heldRecorded = true;
					if (next.missionId != null && next.challengeId == null) {
						markCampaignCleared(next.missionId);
					}
					heldOpen = true;
					queueMicrotask(() => {
						if (active && !active.paused) active.togglePause();
					});
				}
			},
			withPack(
				workshop && mapJson
					? { mapJson, modifierId: modId }
					: missionId != null && !Number.isNaN(missionId)
						? { missionId }
						: challengeId != null && !Number.isNaN(challengeId)
							? { challengeId }
							: utcDay != null && !Number.isNaN(utcDay)
								? { utcDay }
								: seedHex
									? { mapId, modifierId: modId, seedHex }
									: { mapId, modifierId: modId }
			)
		)
			.then((s) => {
				active = s;
				session = s;
			})
			.catch((e: unknown) => {
				error = e instanceof Error ? e.message : 'Failed to link simulation';
			});
		return () => {
			unsub();
			active?.destroy();
		};
	});

	function dps(sel: SelectedInfo) {
		if (sel.fireInterval <= 0) return 0;
		return sel.damage / sel.fireInterval;
	}

	function targets(item: { hitsGround: boolean; hitsAir: boolean; detects?: boolean }) {
		let label = 'Block';
		if (item.hitsGround && item.hitsAir) label = 'G/A';
		else if (item.hitsAir) label = 'Air';
		else if (item.hitsGround) label = 'Gnd';
		if (item.detects) label += ' · Det';
		return label;
	}

	function strikeHud(id: number) {
		return snap?.strikes.find((s) => s.id === id);
	}

	const atGunCap = $derived(
		snap?.turretCap != null && snap.turretCount >= snap.turretCap
	);
	const matchLabel = $derived(
		snap
			? [
					snap.missionName ??
						(snap.modifierName && snap.modifierName !== 'Standard'
							? `${snap.mapName} · ${snap.modifierName}`
							: snap.mapName),
					snap.packName
				]
					.filter(Boolean)
					.join(' · ')
			: '…'
	);

	function keepGoing() {
		heldOpen = false;
		if (session?.paused) session.togglePause();
	}
</script>

{#snippet inspectPanel(sel: SelectedInfo, player: number, extraClass: string, p2: boolean)}
	<aside class="inspect {extraClass}">
		{#if p2}
			<p class="kicker">Commander 2</p>
		{/if}
		<h3>{sel.name}</h3>
		<p class="hint">{sel.tierName}</p>
		<dl>
			<dt>Tier</dt>
			<dd>{sel.tier}/{sel.maxTier}</dd>
			<dt>Range</dt>
			<dd>{sel.range.toFixed(2)}</dd>
			<dt>DPS</dt>
			<dd>{dps(sel).toFixed(1)}</dd>
			<dt>Fire</dt>
			<dd>{sel.fire}</dd>
			<dt>Targets</dt>
			<dd>{targets(sel)}</dd>
			<dt>Aim</dt>
			<dd>{sel.targetingLabel}</dd>
			<dt>Kills</dt>
			<dd>{sel.kills}</dd>
			<dt>Invested</dt>
			<dd>{sel.invested}</dd>
		</dl>
		<button type="button" onclick={() => session?.cycleTargeting(player)}>
			Target {p2 ? p2Bind('target') : bindLabel('target')} · {sel.targetingLabel}
		</button>
		<button
			type="button"
			onclick={() => session?.lift(player)}
			disabled={(snap?.credits ?? 0) < 6}
			class:active={p2 ? snap?.relocating2 : snap?.relocating}
		>
			{p2 ? (snap?.relocating2 ? 'Cancel move' : 'Move') : snap?.relocating ? 'Cancel move' : 'Move'}
			{p2 ? p2Bind('move') : bindLabel('move')} · 6
		</button>
		{#if sel.range > 0}
			<button
				type="button"
				onclick={() => session?.overcharge(player)}
				disabled={(snap?.credits ?? 0) < 40}
			>
				Overcharge {p2 ? p2Bind('overcharge') : bindLabel('overcharge')} · 40
			</button>
		{/if}
		{#if sel.canConvert}
			<button
				type="button"
				onclick={() => session?.convert(player)}
				disabled={(sel.convertCost ?? 1) > (snap?.credits ?? 0)}
			>
				Air-tune {p2 ? p2Bind('convert') : bindLabel('convert')} · {sel.convertCost}
			</button>
		{/if}
		<button
			type="button"
			onclick={() => session?.upgrade(player)}
			disabled={sel.upgradeCost == null || (sel.upgradeCost ?? 0) > (snap?.credits ?? 0)}
		>
			Upgrade {p2 ? p2Bind('upgrade') : bindLabel('upgrade')}
			{#if sel.upgradeCost != null}
				· {sel.upgradeCost}
			{/if}
		</button>
		<button type="button" onclick={() => session?.sell(player)}>
			Sell {p2 ? p2Bind('sell') : bindLabel('sell')} · {sel.sellValue}
		</button>
	</aside>
{/snippet}

{#if error}
	<div class="boot-error">{error}</div>
{:else}
	<div class="play">
		<header class="topbar">
			<div class="meters">
				<div class="meter integrity">
					<b>{snap?.integrity ?? '—'}</b>
					<span>Relay</span>
				</div>
				<div class="meter credits">
					<b>{snap?.credits ?? '—'}</b>
					<span>Scrap</span>
				</div>
				{#if snap && snap.interestBps > 0}
					<div class="meter">
						<b>
							{#if snap.interestPaid > 0}+{snap.interestPaid}{:else}{(snap.interestBps / 100).toFixed(0)}%{/if}
						</b>
						<span>Interest</span>
					</div>
				{/if}
				<div class="meter wave">
					<b>{snap?.wave ?? '—'}</b>
					<span>
						{#if snap?.status === 'fortify'}
							Fortify {snap.nextWaveIn.toFixed(1)}s
						{:else if snap?.status === 'incoming'}
							{snap.creepsAlive + snap.creepsRemaining} inbound
						{:else}
							Wave
						{/if}
					</span>
				</div>
				<div class="meter">
					<b>{snap?.kills ?? 0}</b>
					<span>Kills</span>
				</div>
				{#if snap}
					<div class="meter">
						<b>
							{#if snap.hover?.walkAfter != null && snap.hover.walkAfter !== snap.walk}
								{snap.walk}→{snap.hover.walkAfter}
							{:else}
								{snap.walk}
							{/if}
						</b>
						<span>Walk</span>
					</div>
				{/if}
				<div class="meter">
					<b>{best || '—'}</b>
					<span>Best</span>
				</div>
				{#if snap?.turretCap != null}
					<div class="meter">
						<b>{snap.turretCount}/{snap.turretCap}</b>
						<span>Guns</span>
					</div>
				{/if}
				{#if snap?.objectiveWave != null}
					<div class="meter">
						<b>{snap.objectiveCleared ? 'Held' : snap.objectiveWave}</b>
						<span>{snap.objectiveCleared ? 'Objective' : 'Hold'}</span>
					</div>
				{/if}
				{#if snap?.waveIntel}
					<div class="meter">
						<b>{snap.waveIntel.script}</b>
						<span>{snap.status === 'fortify' ? 'Next' : 'Now'}</span>
					</div>
				{/if}
			</div>
			<div class="top-actions">
				<span class="map-chip">{matchLabel}</span>
				{#if watch}
					<span class="map-chip coop">Watching replay</span>
				{/if}
				{#if coop}
					<span class="map-chip coop">Co-op · shared scrap</span>
				{/if}
				{#if !watch}
					<button type="button" onclick={() => session?.callWave()} disabled={!snap?.canCallWave}>
						Call wave {bindLabel('call')}
					</button>
					<button
						type="button"
						onclick={() => session?.repair()}
						disabled={
							!snap ||
							snap.defeated ||
							snap.integrity >= snap.integrityMax ||
							snap.credits < 35
						}
					>
						Repair {bindLabel('repair')} · 35
					</button>
				{/if}
				<button type="button" onclick={() => session?.togglePause()}>
					{paused ? 'Resume' : 'Pause'} {bindLabel('pause')}
				</button>
				<button type="button" onclick={() => session?.cycleSpeed()}>{speed}× {bindLabel('speed')}</button>
				<button type="button" onclick={() => session?.resetView()}>View</button>
				<SettingsDock />
				<a class="btn" href="/campaign">Ops</a>
				<a class="btn" href="/">Briefing</a>
			</div>
		</header>

		<div class="stage">
			<canvas bind:this={canvas}></canvas>
			{#if !session && !error}
				<div class="loading">Linking simulation</div>
			{/if}
			{#if snap?.banner}
				<div class="banner">{snap.banner}</div>
			{/if}
			{#if snap?.message}
				<div class="toast">{snap.message}</div>
			{/if}
			{#if snap?.selected}
				{@render inspectPanel(snap.selected, 0, coop ? 'p1-coop' : '', false)}
			{/if}
			{#if coop && snap?.selected2}
				{@render inspectPanel(snap.selected2, 1, 'p2', true)}
			{/if}
			{#if paused && !snap?.defeated && !heldOpen}
				<div class="defeat pause-overlay">
					<div class="defeat-card">
						<p class="kicker">Halted</p>
						<h2>Command pause</h2>
						<p class="hint">
							{snap?.missionName ?? snap?.mapName} · {snap?.modifierName} · wave {snap?.wave ?? 1}
							{#if snap?.seedHex}
								· {snap.seedHex}
							{/if}
						</p>
						<div class="actions">
							<button class="primary" type="button" onclick={() => session?.togglePause()}>
								Resume
							</button>
							<button type="button" onclick={() => copyReplay()}>Copy replay</button>
							<a class="btn" href="/campaign">Ops</a>
							<a class="btn" href="/">Briefing</a>
						</div>
					</div>
				</div>
			{/if}
			{#if heldOpen && snap?.objectiveCleared && !snap?.defeated}
				<div class="defeat held-overlay">
					<div class="defeat-card">
						<p class="kicker">Objective</p>
						<h2>{campaignDone ? 'Campaign complete' : 'Objective held'}</h2>
						<p class="hint">
							{snap.missionName ?? snap.mapName} through wave {snap.objectiveWave}. The field is still
							open if you want more.
						</p>
						<div class="actions">
							{#if snap.missionId != null}
								<a class="btn primary" href={nextMissionHref} data-sveltekit-reload>
									{campaignDone ? 'Ops board' : 'Advance'}
								</a>
							{:else}
								<a class="btn primary" href="/campaign">Ops board</a>
							{/if}
							<button type="button" onclick={() => keepGoing()}>Keep going</button>
							<button type="button" onclick={() => copyReplay()}>Copy replay</button>
						</div>
					</div>
				</div>
			{/if}
			{#if snap?.defeated}
				<div class="defeat">
					<div class="defeat-card">
						<p class="kicker">Relay down</p>
						<h2>Lost on wave {snap.wave}</h2>
						<p class="hint">
							{snap.kills} kills · {snap.leaks} leaks · best {best}
						</p>
						{#if snap.after}
							<p class="after">
								Spent {snap.after.spent} scrap
								{#if snap.after.killKinds.length}
									· {formatParts(snap.after.killKinds)}
								{/if}
								{#if snap.after.leakKinds.length}
									<br />Leaks · {formatParts(snap.after.leakKinds)}
								{/if}
								{#if snap.after.guns.length}
									<br />
									{snap.after.guns
										.slice(0, 4)
										.map((g) => `${g.name} ${g.kills}`)
										.join(' · ')}
								{/if}
							</p>
						{/if}
						<div class="actions">
							<a class="btn primary" href={retryHref} data-sveltekit-reload>Try again</a>
							<button type="button" onclick={() => copyReplay()}>Copy replay</button>
							<a class="btn" href="/campaign">Ops</a>
							<a class="btn" href="/">Briefing</a>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<footer class="tray">
			{#if !watch && coop}
				<div class="coop-tray">
					<span class="coop-label">P2 · arrows + Enter</span>
					<button
						class="build"
						class:active={snap?.build2 === 0 && snap?.strike2 === 0}
						type="button"
						onclick={() => {
							session?.setStrike(0, 1);
							session?.setBuild(0, 1);
						}}
					>
						Inspect
						<small>{p2Bind('cancel')}</small>
					</button>
					{#each catalog as item (item.id)}
						<button
							class="build"
							class:active={snap?.build2 === item.id}
							type="button"
							onclick={() => session?.setBuild(item.id, 1)}
							disabled={((snap?.credits ?? 0) < item.cost || (atGunCap && item.range > 0)) &&
								snap?.build2 !== item.id}
							title={item.blurb}
						>
							{p2Bind(buildBind(item.id))} {item.name}
							<small>{item.cost}</small>
						</button>
					{/each}
					{#each strikeItems as item (item.id)}
						{@const hud = strikeHud(item.id)}
						<button
							class="build strike"
							class:active={snap?.strike2 === item.id}
							type="button"
							onclick={() => session?.setStrike(item.id, 1)}
							disabled={!(hud?.ready ?? false) && snap?.strike2 !== item.id}
							title={item.blurb}
						>
							{p2Bind(`strike${item.id}` as ActionId)} {item.name}
						</button>
					{/each}
				</div>
			{/if}
			{#if !watch}
			<div class="tray-main">
				<div class="build-list">
					<button
						class="build"
						class:active={snap?.build === 0 && snap?.strike === 0}
						type="button"
						onclick={() => {
							session?.setStrike(0);
							session?.setBuild(0);
						}}
					>
						Inspect
						<small>Select · {bindLabel('cancel')}</small>
					</button>
					{#each catalog as item (item.id)}
						<button
							class="build"
							class:active={snap?.build === item.id}
							type="button"
							onclick={() => session?.setBuild(item.id)}
							disabled={((snap?.credits ?? 0) < item.cost || (atGunCap && item.range > 0)) &&
								snap?.build !== item.id}
							title={item.blurb}
						>
							{bindLabel(buildBind(item.id))} {item.name}
							<small>{item.cost} · {item.role} · {targets(item)}</small>
						</button>
					{/each}
				</div>
				<div class="strike-list">
					{#each strikeItems as item (item.id)}
						{@const hud = strikeHud(item.id)}
						<button
							class="build strike"
							class:active={snap?.strike === item.id}
							type="button"
							onclick={() => session?.setStrike(item.id)}
							disabled={!(hud?.ready ?? false) && snap?.strike !== item.id}
							title={item.blurb}
						>
							{bindLabel(`strike${item.id}` as ActionId)} {item.name}
							<small>
								{item.cost}
								{#if hud && hud.cooldown > 0}
									· {hud.cooldown.toFixed(1)}s
								{/if}
							</small>
						</button>
					{/each}
				</div>
			</div>
			{/if}
			<p class="hint">
				{#if watch}
					Watching a recorded match. Pause and speed still work.
				{:else if snap?.relocating || (coop && snap?.relocating2)}
					Click a cell to move · 6 scrap · {bindLabel('cancel')} cancels.
				{:else if snap?.status === 'fortify' && snap.waveIntel}
					Next · {snap.waveIntel.script}
					{#if snap.waveIntel.parts.length}
						· {formatParts(snap.waveIntel.parts)}
					{/if}
				{:else if coop && snap?.hover2?.reason}
					P2 · {snap.hover2.reason}
				{:else if snap?.hover?.reason}
					{snap.hover.reason}
				{:else if coop}
					P1 mouse · drag paints while a gun is selected · P2 arrows + hold Enter to paint.
					P1 keys win if a bind overlaps. Walls and guns both block the walk.
				{:else}
					Drag to paint while a structure is selected · middle-drag pans · click the minimap to look ·
					wheel/pinch zoom · {bindLabel('viewReset')} resets. Shades walk past guns without Det.
				{/if}
			</p>
		</footer>
	</div>
{/if}
