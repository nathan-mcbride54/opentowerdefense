<script lang="ts">
	import { onMount } from 'svelte';
	import { paintMapThumb, type MapThumbSrc } from './sprites';
	import type { MapDoc } from './types';

	let { map, large = false }: { map: MapDoc | MapThumbSrc | undefined; large?: boolean } = $props();
	let canvas: HTMLCanvasElement | undefined = $state();

	function src(): MapThumbSrc | null {
		if (!map || !map.w) return null;
		const extra = {
			slug: 'slug' in map ? map.slug : undefined,
			seed: 'seed' in map ? map.seed : undefined,
			name: 'name' in map ? map.name : undefined
		};
		if ('cores' in map && map.cores) {
			return { w: map.w, h: map.h, rocks: map.rocks, cores: map.cores, spawns: map.spawns, ...extra };
		}
		if ('core' in map) {
			const m = map as MapThumbSrc;
			return { w: m.w, h: m.h, rocks: m.rocks, cores: m.core, spawns: m.spawns, ...extra };
		}
		return map as MapThumbSrc;
	}

	function paint() {
		if (!canvas) return;
		const next = src();
		if (next) paintMapThumb(canvas, next, large);
	}

	onMount(() => {
		paint();
		const ro = new ResizeObserver(paint);
		if (canvas) ro.observe(canvas);
		return () => ro.disconnect();
	});

	$effect(() => {
		map;
		large;
		paint();
	});
</script>

<canvas
	bind:this={canvas}
	class="map-thumb"
	class:large
	width="240"
	height="150"
	aria-hidden="true"
></canvas>
