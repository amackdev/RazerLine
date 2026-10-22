<script lang="ts">
	import { Tween } from "svelte/motion";
	import { cubicOut } from "svelte/easing";
	import Card from "./Card.svelte";
	import { dpi, setDpi } from "../state/dpi.svelte";

	let { min, max, delay = 0 }: { min: number; max: number; delay?: number } = $props();

	const displayed = Tween.of(() => dpi.x, { duration: 350, easing: cubicOut });

	let draft = $state(dpi.x);
	let dragging = $state(false);

	$effect(() => {
		if (!dragging) draft = dpi.x;
	});

	const presets = [400, 800, 1200, 1600, 2400, 3200, 6400, 12000, 16000, 20000].filter(
		(p) => p >= min && p <= max,
	);

	async function commit(value: number) {
		const clamped = Math.min(max, Math.max(min, Math.round(value)));
		draft = clamped;
		await setDpi(clamped, clamped);
	}
</script>

<Card title="Sensitivity" {delay}>
	<div class="readout">
		<span class="value">{Math.round(displayed.current)}</span>
		<span class="unit">DPI</span>
	</div>

	<input
		type="range"
		{min}
		{max}
		step="10"
		bind:value={draft}
		oninput={() => (dragging = true)}
		onchange={() => {
			dragging = false;
			commit(draft);
		}}
	/>

	<div class="presets">
		{#each presets as preset (preset)}
			<button type="button" class:active={dpi.x === preset} onclick={() => commit(preset)}>
				{preset}
			</button>
		{/each}
	</div>
</Card>

<style>
	.readout {
		display: flex;
		align-items: baseline;
		gap: 6px;
		margin-bottom: 14px;
	}

	.value {
		font-size: 34px;
		font-weight: 700;
		font-family: var(--font-mono);
		letter-spacing: -0.02em;
	}

	.unit {
		font-size: 13px;
		color: var(--muted);
		font-weight: 600;
	}

	input[type="range"] {
		width: 100%;
		accent-color: var(--accent);
		margin-bottom: 14px;
	}

	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.presets button {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text-dim);
		font-size: 12px;
		font-weight: 600;
		padding: 5px 10px;
		border-radius: 7px;
		cursor: pointer;
		transition: all 0.2s var(--ease-out);
	}

	.presets button:hover {
		border-color: var(--border-strong);
		color: var(--text);
	}

	.presets button.active {
		background: var(--accent);
		border-color: var(--accent);
		color: #061a0f;
	}
</style>
