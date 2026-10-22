<script lang="ts">
	import { gsap } from "gsap";
	import { tick } from "svelte";

	let {
		tabs,
		active,
		onselect,
	}: { tabs: { id: string; label: string }[]; active: string; onselect: (id: string) => void } =
		$props();

	let tabEls: Record<string, HTMLButtonElement> = {};
	let underlineEl: HTMLDivElement;
	let trackEl: HTMLElement;

	async function moveUnderline() {
		await tick();
		const btn = tabEls[active];
		if (!btn || !underlineEl || !trackEl) return;
		const trackRect = trackEl.getBoundingClientRect();
		const btnRect = btn.getBoundingClientRect();
		gsap.to(underlineEl, {
			x: btnRect.left - trackRect.left,
			width: btnRect.width,
			duration: 0.4,
			ease: "power3.out",
		});
	}

	$effect(() => {
		active;
		moveUnderline();
	});
</script>

<nav bind:this={trackEl}>
	<div class="underline" bind:this={underlineEl}></div>
	{#each tabs as tab (tab.id)}
		<button
			type="button"
			class:active={tab.id === active}
			bind:this={tabEls[tab.id]}
			onclick={() => onselect(tab.id)}
		>
			{tab.label}
		</button>
	{/each}
</nav>

<style>
	nav {
		position: relative;
		display: flex;
		gap: 4px;
		padding: 0 22px;
		background: var(--bg-elevated);
		border-bottom: 1px solid var(--border);
	}

	.underline {
		position: absolute;
		bottom: -1px;
		left: 0;
		height: 2px;
		width: 0;
		background: var(--accent);
	}

	button {
		background: transparent;
		border: none;
		padding: 11px 14px;
		font-size: 12.5px;
		font-weight: 600;
		letter-spacing: 0.02em;
		color: var(--muted);
		cursor: pointer;
		transition: color 0.25s var(--ease-out);
	}

	button:hover {
		color: var(--text-dim);
	}

	button.active {
		color: var(--accent);
	}
</style>
