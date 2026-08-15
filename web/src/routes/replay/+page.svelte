<script lang="ts">
	import { onMount } from 'svelte';
	import { REPLAY_STORAGE } from '$lib/game/session';
	import type { VerifyReport } from '$lib/game/types';

	let raw = $state('');
	let report = $state<VerifyReport | null>(null);
	let message = $state('Paste a copied replay JSON, verify the hash, then watch it on the field.');
	let ready = $state(false);

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

	onMount(() => {
		const saved = sessionStorage.getItem(REPLAY_STORAGE);
		if (saved) raw = saved;
		ready = true;
	});
</script>

<main class="brief">
	<section class="brief-hero">
		<div>
			<p class="kicker">Frontier command · Replay</p>
			<h1>Replay desk</h1>
			<p class="lede">
				A replay is seed, orders, and a hash. Verify it headless, or watch the same ticks on the
				canvas. Pause and speed still work. You cannot issue new orders.
			</p>
			<div class="actions">
				<a class="btn" href="/">Briefing</a>
				<button type="button" onclick={() => verify()} disabled={!ready || !raw.trim()}>
					Verify
				</button>
				<button class="primary" type="button" onclick={() => watch()} disabled={!ready || !raw.trim()}>
					Watch
				</button>
			</div>
			<p class="hint">{message}</p>
			{#if report}
				<p class="hazard">
					<strong>{report.hash || '—'}</strong>
					{#if report.defeated}· relay down{/if}
				</p>
			{/if}
		</div>
	</section>
	<aside class="brief-side">
		<div>
			<h2>Order log</h2>
			<textarea
				class="replay-json"
				bind:value={raw}
				spellcheck="false"
				placeholder={'{ "version": 1, "seed": … }'}
			></textarea>
		</div>
	</aside>
</main>
