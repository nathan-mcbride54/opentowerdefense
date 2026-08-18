<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { createSession, PACK_STORAGE, REPLAY_STORAGE, WORKSHOP_STORAGE, type Session, type SessionOpts } from '$lib/game/session';
	import ShopPic from '$lib/game/ShopPic.svelte';
	import CreepPic from '$lib/game/CreepPic.svelte';
	import SettingsDock from '$lib/game/SettingsDock.svelte';
	import { draggable } from '$lib/game/draggable';
	import { formatKey, type ActionId } from '$lib/game/keys';
	import { loadSettings, subscribeSettings } from '$lib/game/settings';
	import {
		markCampaignCleared,
		readBestWave,
		type CatalogItem,
		type CreepKind,
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
	const replayParam = params.get('replay') === '1';
	const retryHref = (() => {
		if (replayParam) return '/play?replay=1';
		const extra = `${usePack ? '&pack=1' : ''}`;
		if (workshop) return `/play?workshop=1${extra}`;
		if (missionId != null && !Number.isNaN(missionId)) return `/play?mission=${missionId}`;
		if (challengeId != null && !Number.isNaN(challengeId))
			return `/play?challenge=${challengeId}`;
		if (utcDay != null && !Number.isNaN(utcDay)) return `/play?day=${utcDay}${extra}`;
		if (seedHex) return `/play?map=${mapId}&mod=${modId}&seed=${encodeURIComponent(seedHex)}${extra}`;
		return `/play?map=${mapId}&mod=${modId}${extra}`;
	})();
	/** Mission count comes from the engine's campaign list, not a hardcoded index. */
	let missionCount = $state(8);
	const lastMission = $derived(missionCount - 1);
	const nextMissionHref = $derived(
		snap?.missionId != null && snap.missionId < lastMission
			? `/play?mission=${snap.missionId + 1}`
			: '/campaign'
	);
	const campaignDone = $derived(snap?.missionId === lastMission && snap.objectiveCleared);

	function bindLabel(id: ActionId) {
		return formatKey(keys[id]);
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

	function waveIcons(parts: KindCount[] | undefined) {
		if (!parts?.length) return [];
		const out: { kind: CreepKind; name: string; key: string }[] = [];
		for (const p of parts) {
			const cap = p.kind === 'colossus' ? 2 : p.kind === 'bulwark' ? 4 : 7;
			const n = Math.min(Math.max(1, p.count), cap);
			for (let i = 0; i < n; i++) {
				out.push({ kind: p.kind, name: p.name, key: `${p.kind}-${i}` });
			}
		}
		return out.slice(0, 16);
	}

	onMount(() => {
		if (!canvas) return;
		const unsub = subscribeSettings(() => {
			keys = loadSettings().keys;
		});
		best = readBestWave(mapId, modId);
		let active: Session | null = null;
		let dead = false;
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
			const next = { ...opts, replayJson: replayJson ?? undefined };
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
				// The component can unmount while wasm is still loading; without this the
				// session would be created after cleanup ran and never torn down.
				if (dead) {
					s.destroy();
					return;
				}
				active = s;
				session = s;
				// After the match exists. A parallel campaign() during fromMapJson /
				// mapStatic grew the wasm heap and the next snapshot() faulted — black
				// field, frozen fortify timer. Built-in theaters often fit in the
				// initial pages and never showed it.
				void (async () => {
					const { default: init, WasmGame } = await import('$lib/wasm/otd');
					await init();
					const list = JSON.parse(WasmGame.campaign()) as unknown[];
					if (list.length > 0) missionCount = list.length;
				})();
			})
			.catch((e: unknown) => {
				if (dead) return;
				error = e instanceof Error ? e.message : 'Failed to link simulation';
			});
		return () => {
			dead = true;
			unsub();
			active?.destroy();
		};
	});

	function dps(sel: SelectedInfo) {
		if (sel.fireInterval <= 0) return 0;
		return sel.damage / sel.fireInterval;
	}

	function targets(item: CatalogItem | SelectedInfo) {
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
	/** Drives the relay readout's threat colour. 1 when there is no snapshot yet. */
	const relayFrac = $derived(
		snap && snap.integrityMax > 0 ? snap.integrity / snap.integrityMax : 1
	);
	/** Theater name, kept on its own so it can hold the ice blue in the dock. */
	const matchName = $derived(snap ? (snap.missionName ?? snap.mapName) : '…');
	/** Everything qualifying the match: modifier, then any custom loadout. */
	const matchMode = $derived(
		snap
			? [
					snap.missionName == null && snap.modifierName !== 'Standard'
						? snap.modifierName
						: null,
					snap.packName
				]
					.filter(Boolean)
					.join(' · ')
			: ''
	);

	function keepGoing() {
		heldOpen = false;
		if (session?.paused) session.togglePause();
	}
</script>

{#snippet inspectPanel(sel: SelectedInfo)}
	<aside class="inspect" use:draggable={{ handle: '.inspect-grip' }}>
		<div class="inspect-grip" title="Drag to move this panel">
			<p class="kicker">Selected unit</p>
			<h3>{sel.name}</h3>
			<p class="hint">{sel.tierName}</p>
		</div>
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
		<button type="button" onclick={() => session?.cycleTargeting()}>
			Target {bindLabel('target')} · {sel.targetingLabel}
		</button>
		<button
			type="button"
			onclick={() => session?.lift()}
			disabled={(snap?.credits ?? 0) < (snap?.moveCost ?? 0)}
			class:active={snap?.relocating}
		>
			{snap?.relocating ? 'Cancel move' : 'Move'}
			{bindLabel('move')} · {`$${snap?.moveCost ?? '—'}`}
		</button>
		{#if sel.range > 0}
			<button
				type="button"
				onclick={() => session?.overcharge()}
				disabled={(snap?.credits ?? 0) < (snap?.overchargeCost ?? 0)}
			>
				Overcharge {bindLabel('overcharge')} · {`$${snap?.overchargeCost ?? '—'}`}
			</button>
		{/if}
		{#if sel.canConvert}
			<button
				type="button"
				onclick={() => session?.convert()}
				disabled={(sel.convertCost ?? 1) > (snap?.credits ?? 0)}
			>
				Air-tune {bindLabel('convert')} · {`$${sel.convertCost}`}
			</button>
		{/if}
		<button
			type="button"
			onclick={() => session?.upgrade()}
			disabled={sel.upgradeCost == null || (sel.upgradeCost ?? 0) > (snap?.credits ?? 0)}
		>
			Upgrade {bindLabel('upgrade')}
			{#if sel.upgradeCost != null}
				· {`$${sel.upgradeCost}`}
			{/if}
		</button>
		<button type="button" onclick={() => session?.sell()}>
			Sell {bindLabel('sell')} · {`$${sel.sellValue}`}
		</button>
	</aside>
{/snippet}

{#if error}
	<div class="boot-error">{error}</div>
{:else}
	<div class="play">
		<div class="stage">
			<canvas bind:this={canvas}></canvas>
			<span class="stage-chip">
				<b>{matchName}</b>{#if matchMode}<i>{matchMode}</i>{/if}
			</span>
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
				{@render inspectPanel(snap.selected)}
			{/if}
			{#if paused && !snap?.defeated && !heldOpen}
				<div class="defeat pause-overlay">
					<div class="defeat-card">
						<p class="kicker">Taking a breath</p>
						<h2>Paused</h2>
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
						<h2>{campaignDone ? 'Campaign complete' : 'You held it'}</h2>
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
						<p class="kicker">The lamp went out</p>
						<h2>You held to wave {snap.wave}</h2>
						<p class="hint">
							{snap.kills} kills · {snap.leaks} leaks · best {best}
						</p>
						{#if snap.after}
							<p class="after">
								Spent {`$${snap.after.spent}`}
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

		<footer class="tray dock">
			<!-- `watch` is on the body, not the command block, because the missing arsenal
			     changes the TRACK layout as well as the button flow. -->
			<div class="dock-body" class:watch>
			{#if !watch}
			<div class="tray-main dock-arsenal">
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
						<span class="shop-head">
							<span class="shop-key">{bindLabel('cancel')}</span>
						</span>
						<ShopPic icon={{ kind: 'inspect' }} />
						<span class="shop-name">Inspect</span>
						<small>Select</small>
					</button>
					{#each catalog as item (item.id)}
						<button
							class="build"
							class:active={snap?.build === item.id}
							type="button"
							data-id={item.id}
							onclick={() => session?.setBuild(item.id)}
							disabled={((snap?.credits ?? 0) < item.cost || (atGunCap && item.range > 0)) &&
								snap?.build !== item.id}
							title={item.blurb}
						>
							<span class="shop-head">
								<span class="shop-key">{bindLabel(buildBind(item.id))}</span>
							</span>
							<ShopPic icon={{ kind: 'build', id: item.id }} />
							<span class="shop-name">{item.name}</span>
							<span class="shop-cost">{`$${item.cost}`}</span>
							<small>{item.role} · {targets(item)}</small>
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
							disabled={(!(hud?.ready ?? false) || (snap?.credits ?? 0) < item.cost) &&
								snap?.strike !== item.id}
							title={item.blurb}
						>
							<span class="shop-head">
								<span class="shop-key">{bindLabel(`strike${item.id}` as ActionId)}</span>
							</span>
							<ShopPic icon={{ kind: 'strike', id: item.id }} />
							<span class="shop-name">{item.name}</span>
							<span class="shop-cost">{`$${item.cost}`}</span>
							<small>
								{#if hud && hud.cooldown > 0}
									{hud.cooldown.toFixed(1)}s
								{:else}
									Ready
								{/if}
							</small>
						</button>
					{/each}
				</div>
			</div>
			{/if}
			<div class="dock-intel">
				<div class="dock-scores">
					<div class="dock-credits">
						<span>Credits</span>
						<b>{snap != null ? `$${snap.credits}` : '—'}</b>
						{#if snap && snap.interestBps > 0}
							<small>
								{#if snap.interestPaid > 0}+{snap.interestPaid}{:else}{(snap.interestBps / 100).toFixed(0)}% int.{/if}
							</small>
						{/if}
					</div>
					<div
						class="dock-stat integrity"
						class:hurt={relayFrac <= 0.6 && relayFrac > 0.25}
						class:crit={relayFrac <= 0.25}
					>
						<span>Tower</span>
						<b>{snap?.integrity ?? '—'}{#if snap}<i>/{snap.integrityMax}</i>{/if}</b>
					</div>
					<div class="dock-stat wave" class:incoming={snap?.status === 'incoming'}>
						<span>Wave</span>
						<b>{snap?.wave ?? '—'}</b>
					</div>
					<!-- Credits, Tower and Wave are the three read-at-a-glance readouts and keep
					     line 1 to themselves. Everything below them travels as one inline run so
					     the wrap breaks in the right place at every width, and so screen-reader
					     order matches visual order (a CSS `order` would have desynced them). -->
					<div class="dock-run">
						<div class="dock-score">
							<span>Score</span>
							<b>{snap?.kills ?? 0}</b>
						</div>
						{#if snap}
							<div class="dock-stat walk">
								<span>Walk</span>
								<b>
									{#if snap.hover?.walkAfter != null && snap.hover.walkAfter !== snap.walk}
										{@const after = snap.hover.walkAfter}
										<span class="was">{snap.walk}</span><span
											class="delta"
											class:up={after > snap.walk}
											class:down={after < snap.walk}
											>→{after}</span
										>
									{:else}
										{snap.walk}
									{/if}
								</b>
							</div>
						{/if}
						<div class="dock-stat best">
							<span>Best</span>
							<b>{best || '—'}</b>
						</div>
						{#if snap?.turretCap != null}
							<div class="dock-stat guns">
								<span>Guns</span>
								<b>{snap.turretCount}/{snap.turretCap}</b>
							</div>
						{/if}
						{#if snap?.objectiveWave != null}
							<div class="dock-stat hold" class:cleared={snap.objectiveCleared}>
								<span>{snap.objectiveCleared ? 'Objective' : 'Hold'}</span>
								<b>{snap.objectiveCleared ? 'Held' : snap.objectiveWave}</b>
							</div>
						{/if}
					</div>
				</div>
				<div class="wave-preview" class:empty={!snap?.waveIntel?.parts.length}>
					{#if snap?.waveIntel?.parts.length}
						<!-- Icons get their own track so a long wave never pushes the script
						     name out of the strip; the script block stays pinned. -->
						<div class="wave-units">
							{#each waveIcons(snap.waveIntel.parts) as unit (unit.key)}
								<div class="wave-unit" title={unit.name}>
									<CreepPic kind={unit.kind} />
								</div>
							{/each}
						</div>
						<div class="wave-script">
							<strong>{snap.waveIntel.script}</strong>
							<span>{formatParts(snap.waveIntel.parts)}</span>
						</div>
					{:else}
						<span class="wave-empty">No inbound yet</span>
					{/if}
				</div>
				<p class="dock-eta" class:hot={snap?.status === 'incoming'}>
					{#if watch}
						Watching replay
					{:else if snap?.status === 'fortify'}
						Wave {snap.wave} · next in {snap.nextWaveIn.toFixed(2)}s
					{:else if snap?.status === 'incoming'}
						Wave {snap.wave} · {snap.creepsAlive + snap.creepsRemaining} inbound
					{:else}
						Wave {snap?.wave ?? '—'}
					{/if}
				</p>
			</div>
			<div class="dock-cmd">
				{#if !watch}
					<button
						type="button"
						class="send-now cmd"
						class:primary={!!snap?.canCallWave}
						onclick={() => session?.callWave()}
						disabled={!snap?.canCallWave}
					>
						<span class="cmd-label">Send wave</span>
						<kbd>{bindLabel('call')}</kbd>
					</button>
					<button
						type="button"
						class="cmd"
						onclick={() => session?.repair()}
						disabled={
							!snap ||
							snap.defeated ||
							snap.integrity >= snap.integrityMax ||
							snap.credits < snap.repairCost
						}
					>
						<span class="cmd-label">Repair</span>
						<span class="cmd-cost">{`$${snap?.repairCost ?? '—'}`}</span>
						<kbd>{bindLabel('repair')}</kbd>
					</button>
				{/if}
				<button type="button" class="cmd" onclick={() => session?.togglePause()}>
					<span class="cmd-label">{paused ? 'Resume' : 'Pause'}</span>
					<kbd>{bindLabel('pause')}</kbd>
				</button>
				<button type="button" class="cmd" onclick={() => session?.cycleSpeed()}>
					<span class="cmd-label">Speed {speed}×</span>
					<kbd>{bindLabel('speed')}</kbd>
				</button>
				<button type="button" class="cmd" onclick={() => session?.resetView()}>
					<span class="cmd-label">View</span>
					<kbd>{bindLabel('viewReset')}</kbd>
				</button>
				<SettingsDock compact />
				<a class="btn" href="/campaign">Ops</a>
				<a class="btn" href="/">Briefing</a>
			</div>
			</div>
		</footer>
	</div>
{/if}
