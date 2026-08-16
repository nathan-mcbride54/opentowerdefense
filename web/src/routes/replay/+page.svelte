<script lang="ts">
	import { onMount } from 'svelte';
	import { REPLAY_STORAGE } from '$lib/game/session';
	import MenuChrome from '$lib/game/MenuChrome.svelte';
	import type { VerifyReport } from '$lib/game/types';

	let raw = $state('');
	let report = $state<VerifyReport | null>(null);
	let message = $state('Paste a copied replay JSON, drop a file, verify the hash, then watch it on the field.');
	let ready = $state(false);
	let dragging = $state(false);

	async function wasm() {
		const { default: init, WasmGame } = await import('$lib/wasm/otd');
		await init();
		return WasmGame;
	}

	async function verify() {
		const WasmGame = await wasm();
		try {
			JSON.parse(raw);
		} catch {
			report = null;
			message = 'That is not JSON.';
			return;
		}
		report = JSON.parse(WasmGame.verifyReplay(raw)) as VerifyReport;
		message = report.error
			? report.error
			: report.ok
				? `Hash holds. Wave ${report.wave} · ${report.kills} kills · ${report.leaks} leaks · ${report.ticks} ticks.`
				: `Mismatch. Hash ${report.hashOk ? 'ok' : 'fails'} · outcome ${report.outcomeOk ? 'ok' : 'fails'}.`;
	}

	function watch() {
		try {
			JSON.parse(raw);
		} catch {
			message = 'That is not JSON.';
			return;
		}
		sessionStorage.setItem(REPLAY_STORAGE, raw);
		window.location.href = '/play?replay=1';
	}

	function readFile(file: File) {
		void file.text().then((t) => {
			raw = t;
			message = `Loaded ${file.name}. Verify, then watch.`;
			report = null;
		});
	}

	onMount(() => {
		const saved = sessionStorage.getItem(REPLAY_STORAGE);
		if (saved) raw = saved;
		ready = true;
	});
</script>

<main class="brief replay-desk">
	<MenuChrome titleMark="Record" title="Replay desk" current="replay" />
	<section class="brief-hero">
		<div>
			<p class="lede">
				A replay is seed, orders, and a hash. Verify it headless, or watch the same ticks on the
				canvas. Pause and speed still work. You cannot issue new orders.
			</p>
			<div class="actions">
				<button type="button" onclick={() => verify()} disabled={!ready || !raw.trim()}>
					Verify
				</button>
				<button class="primary" type="button" onclick={() => watch()} disabled={!ready || !raw.trim()}>
					Watch
				</button>
				<label class="btn file-btn">
					Open file
					<input
						type="file"
						accept="application/json,.json"
						onchange={(e) => {
							const f = e.currentTarget.files?.[0];
							if (f) readFile(f);
						}}
					/>
				</label>
			</div>
			<p class="hint">{message}</p>
			{#if report}
				<dl class="replay-sheet">
					<div><dt>Hash</dt><dd>{report.hash || '—'}</dd></div>
					<div><dt>Wave</dt><dd>{report.wave ?? '—'}</dd></div>
					<div><dt>Kills</dt><dd>{report.kills ?? '—'}</dd></div>
					<div><dt>Leaks</dt><dd>{report.leaks ?? '—'}</dd></div>
					<div><dt>Ticks</dt><dd>{report.ticks ?? '—'}</dd></div>
					<div><dt>Relay</dt><dd>{report.defeated ? 'down' : 'held'}</dd></div>
				</dl>
			{/if}
		</div>
	</section>
	<aside class="brief-side">
		<div>
			<h2>Order log</h2>
			<div
				class="drop-wrap"
				class:dragging
				role="group"
				aria-label="Replay JSON"
				ondragover={(e) => {
					e.preventDefault();
					dragging = true;
				}}
				ondragleave={() => (dragging = false)}
				ondrop={(e) => {
					e.preventDefault();
					dragging = false;
					const f = e.dataTransfer?.files?.[0];
					if (f) readFile(f);
				}}
			>
				<textarea
					class="replay-json"
					bind:value={raw}
					spellcheck="false"
					placeholder={'Drop a .json here, or paste:\n{ "version": 1, "seed": … }'}
				></textarea>
			</div>
		</div>
	</aside>
</main>
