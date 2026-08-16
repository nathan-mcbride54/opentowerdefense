<script lang="ts">
	import { onMount } from 'svelte';
	import { paintCreepIcon } from './sprites';
	import type { CreepKind } from './types';

	let { kind }: { kind: CreepKind } = $props();
	let canvas: HTMLCanvasElement | undefined = $state();

	function paint() {
		if (canvas) paintCreepIcon(canvas, kind);
	}

	onMount(() => {
		paint();
		const ro = new ResizeObserver(paint);
		if (canvas) ro.observe(canvas);
		return () => ro.disconnect();
	});

	$effect(() => {
		kind;
		paint();
	});
</script>

<canvas bind:this={canvas} class="wave-pic" width="96" height="96" aria-hidden="true"></canvas>
