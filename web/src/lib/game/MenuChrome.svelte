<script lang="ts">
	type Desk = 'maps' | 'campaign' | 'workshop' | 'pack' | 'replay';

	let {
		titleMark,
		title,
		current,
		compact = false,
		lead,
		actions
	}: {
		titleMark?: string;
		title?: string;
		current: Desk;
		compact?: boolean;
		lead?: import('svelte').Snippet;
		actions?: import('svelte').Snippet;
	} = $props();

	const links = [
		{ id: 'maps' as const, href: '/', label: 'Maps' },
		{ id: 'campaign' as const, href: '/campaign', label: 'Campaign' },
		{ id: 'workshop' as const, href: '/workshop', label: 'Workshop' },
		{ id: 'pack' as const, href: '/pack', label: 'Loadout' },
		{ id: 'replay' as const, href: '/replay', label: 'Replay' }
	];

	const docTitle = $derived(
		!title || current === 'maps' ? 'Open Tower Defense' : `${title} · Open Tower Defense`
	);
</script>

<svelte:head>
	<title>{docTitle}</title>
</svelte:head>

<div class="menu-chrome">
	<header class="menu-top" class:compact-top={compact}>
		{#if !compact}
			<div>
				{#if title}
					<h1>
						{#if titleMark}<span class="title-mark">{titleMark}</span>{/if}{title}
					</h1>
				{/if}
				{@render lead?.()}
			</div>
		{/if}
		<nav class="menu-nav" aria-label="Operations">
			{#each links as link (link.id)}
				<a
					class="btn"
					href={link.href}
					aria-current={current === link.id ? 'page' : undefined}
				>
					{link.label}
				</a>
			{/each}
			{@render actions?.()}
		</nav>
	</header>
</div>
