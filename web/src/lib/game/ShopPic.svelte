<script lang="ts">
	import { onMount } from 'svelte';
	import { paintShopIcon, type ShopSlot } from './sprites';

	let { icon }: { icon: ShopSlot } = $props();
	let canvas: HTMLCanvasElement | undefined = $state();

	function paint() {
		if (canvas) paintShopIcon(canvas, icon);
	}

	onMount(() => {
		paint();
		const ro = new ResizeObserver(paint);
		if (canvas) ro.observe(canvas);
		return () => ro.disconnect();
	});

	$effect(() => {
		icon;
		paint();
	});
</script>

<canvas bind:this={canvas} class="shop-pic" width="128" height="128" aria-hidden="true"></canvas>
