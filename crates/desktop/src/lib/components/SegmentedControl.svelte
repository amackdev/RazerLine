<script lang="ts">
	import { gsap } from "gsap";
	import { tick } from "svelte";

	let {
		options,
		value,
		onselect,
	}: {
		options: { label: string; value: number }[];
		value: number | null;
		onselect: (value: number) => void;
	} = $props();

	let trackEl: HTMLDivElement;
	let buttonEls: Record<number, HTMLButtonElement> = {};
	let highlightEl: HTMLDivElement;

	async function moveHighlight() {
		await tick();
		const idx = options.findIndex((o) => o.value === value);
		if (idx === -1 || !highlightEl || !trackEl) {
			if (highlightEl) gsap.to(highlightEl, { opacity: 0, duration: 0.2 });
			return;
		}
		const btn = buttonEls[options[idx].value];
		if (!btn) return;
		const trackRect = trackEl.getBoundingClientRect();
		const btnRect = btn.getBoundingClientRect();
		gsap.to(highlightEl, {
			x: btnRect.left - trackRect.left,
			width: btnRect.width,
			opacity: 1,
			duration: 0.45,
			ease: "elastic.out(1, 0.75)",
		});
	}

	$effect(() => {
		value;
		options;
		moveHighlight();
	});
</script>

<div class="track" bind:this={trackEl}>
	<div class="highlight" bind:this={highlightEl}></div>
	{#each options as option (option.value)}
		<button
			type="button"
			class:active={option.value === value}
			bind:this={buttonEls[option.value]}
			onclick={() => onselect(option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>

<style>
	.track {
		position: relative;
		display: inline-flex;
		gap: 2px;
		padding: 4px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
	}

	.highlight {
		position: absolute;
		top: 4px;
		left: 0;
		height: calc(100% - 8px);
		width: 0;
		opacity: 0;
		background: var(--accent);
		border-radius: 6px;
		z-index: 0;
	}

	button {
		position: relative;
		z-index: 1;
		background: transparent;
		border: none;
		padding: 7px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		color: var(--text-dim);
		transition: color 0.25s var(--ease-out);
	}

	button:hover {
		color: var(--text);
	}

	button.active {
		color: #061a0f;
	}
</style>
