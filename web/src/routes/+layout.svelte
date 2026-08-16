<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import { applyUiScale, loadSettings } from '$lib/game/settings';

	let { children } = $props();

	onMount(() => applyUiScale(loadSettings().uiScale));
</script>

<svelte:head>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Barlow+Condensed:wght@600;700;800&family=IBM+Plex+Sans:wght@400;500&family=Oswald:wght@500;600;700&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="fx-grain" aria-hidden="true"></div>
{@render children()}
