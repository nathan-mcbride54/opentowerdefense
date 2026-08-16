<script lang="ts">
	import { onMount } from 'svelte';
	import {
		ACTION_IDS,
		ACTION_LABELS,
		formatKey,
		normalizeKey,
		type ActionId
	} from './keys';
	import {
		applyUiScale,
		loadSettings,
		patchSettings,
		rebindKey,
		resetKeys,
		subscribeSettings,
		type Palette,
		type Settings,
		type UiScale
	} from './settings';

	let { compact = false }: { compact?: boolean } = $props();
	let open = $state(false);
	let s = $state<Settings>(loadSettings());
	let capturing = $state<ActionId | null>(null);

	/** Teardown for an armed rebind. A pending capture must never outlive the panel:
	 *  a leaked capture-phase listener swallows the next keypress anywhere in the app
	 *  and silently rebinds the wrong action. */
	let cancelCapture: (() => void) | null = null;

	onMount(() => {
		applyUiScale(s.uiScale);
		const unsub = subscribeSettings(() => {
			s = loadSettings();
			applyUiScale(s.uiScale);
		});
		return () => {
			unsub();
			cancelCapture?.();
		};
	});

	$effect(() => {
		if (!open) cancelCapture?.();
	});

	function setPalette(palette: Palette) {
		patchSettings({ palette });
	}

	function setScale(uiScale: UiScale) {
		patchSettings({ uiScale });
	}

	function capture(id: ActionId) {
		cancelCapture?.();
		capturing = id;
		const on = (ev: KeyboardEvent) => {
			ev.preventDefault();
			ev.stopImmediatePropagation();
			// Chords belong to the browser; binding one would make it unreachable in play.
			if (ev.ctrlKey || ev.metaKey || ev.altKey) return;
			if (ev.key !== 'Escape' || id === 'cancel') rebindKey(id, normalizeKey(ev));
			cancelCapture?.();
		};
		cancelCapture = () => {
			window.removeEventListener('keydown', on, true);
			capturing = null;
			cancelCapture = null;
		};
		window.addEventListener('keydown', on, true);
	}
</script>

<div class="settings" class:compact>
	<button type="button" class="gear" onclick={() => (open = !open)} aria-expanded={open}>
		Settings
	</button>
	{#if open}
		<div class="panel">
			<label class="row">
				<input
					type="checkbox"
					checked={s.mute}
					onchange={() => patchSettings({ mute: !s.mute })}
				/>
				Mute
			</label>
			<label class="row">
				Volume
				<input
					type="range"
					min="0"
					max="1"
					step="0.05"
					value={s.volume}
					oninput={(e) => patchSettings({ volume: Number(e.currentTarget.value) })}
				/>
			</label>
			<label class="row">
				<input
					type="checkbox"
					checked={s.reducedFx}
					onchange={() => patchSettings({ reducedFx: !s.reducedFx })}
				/>
				Reduced FX
			</label>
			<div class="row wrap">
				<span>Palette</span>
				<button type="button" class:active={s.palette === 'default'} onclick={() => setPalette('default')}>
					Default
				</button>
				<button type="button" class:active={s.palette === 'safe'} onclick={() => setPalette('safe')}>
					Color-safe
				</button>
				<button type="button" class:active={s.palette === 'high'} onclick={() => setPalette('high')}>
					High contrast
				</button>
			</div>
			<div class="row wrap">
				<span>UI</span>
				<button type="button" class:active={s.uiScale === 'sm'} onclick={() => setScale('sm')}>S</button>
				<button type="button" class:active={s.uiScale === 'md'} onclick={() => setScale('md')}>M</button>
				<button type="button" class:active={s.uiScale === 'lg'} onclick={() => setScale('lg')}>L</button>
			</div>
			<div class="keys-head">
				<span>Keys</span>
				<button type="button" onclick={() => resetKeys()}>Reset</button>
			</div>
			<div class="keys">
				{#each ACTION_IDS as id}
					<button
						type="button"
						class="bind"
						class:capturing={capturing === id}
						data-rebind="1"
						onclick={() => capture(id)}
					>
						<span>{ACTION_LABELS[id]}</span>
						<kbd>{capturing === id ? '…' : formatKey(s.keys[id])}</kbd>
					</button>
				{/each}
			</div>
			<p class="hint">Click a bind, then tap a key. Esc cancels. Drag the field to pan; wheel or pinch to zoom.</p>
		</div>
	{/if}
</div>

<style>
	.settings {
		position: relative;
	}
	.panel {
		position: absolute;
		right: 0;
		top: calc(100% + 0.4rem);
		z-index: 20;
		min-width: 18rem;
		max-height: min(70vh, 36rem);
		overflow: auto;
		padding: 0.75rem 0.85rem;
		background: linear-gradient(180deg, #141816, #070807);
		border: 1px solid rgba(77, 184, 212, 0.4);
		box-shadow: 0 12px 28px rgba(0, 0, 0, 0.45);
		display: flex;
		flex-direction: column;
		gap: 0.55rem;
	}
	/* Compact is only used by the play dock, whose gear sits at the right edge —
	   left-anchoring pushed the panel past the viewport, where `.play { overflow: hidden }`
	   clipped the right column of key binds. */
	.compact .panel {
		right: 0;
		left: auto;
		bottom: calc(100% + 0.4rem);
		top: auto;
		max-width: calc(100vw - 1.5rem);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.85rem;
		color: var(--muted);
	}
	.wrap {
		flex-wrap: wrap;
	}
	.row button.active {
		border-color: var(--accent);
		color: var(--accent);
	}
	.gear {
		padding: 0.4rem 0.7rem;
	}
	input[type='range'] {
		flex: 1;
	}
	.keys-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		color: var(--muted);
		font-size: 0.85rem;
		margin-top: 0.2rem;
	}
	.keys {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.3rem;
	}
	.bind {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.4rem;
		padding: 0.35rem 0.45rem;
		font-size: 0.75rem;
		text-transform: none;
		letter-spacing: 0;
		font-family: var(--font);
		font-weight: 400;
	}
	.bind.capturing {
		border-color: var(--warn);
		color: var(--warn);
	}
	.hint {
		margin: 0;
		font-size: 0.72rem;
		color: var(--muted);
	}
</style>
