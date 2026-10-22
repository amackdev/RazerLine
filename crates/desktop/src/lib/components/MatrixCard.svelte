<script lang="ts">
	import Card from "./Card.svelte";
	import { api } from "../api";

	let { rows, cols, delay = 0 }: { rows: number; cols: number; delay?: number } = $props();

	let color = $state<[number, number, number]>([64, 209, 128]);
	let on = $state(true);
	let error = $state("");

	let hexColor = $derived(
		"#" + color.map((c) => c.toString(16).padStart(2, "0")).join(""),
	);

	async function fill() {
		error = "";
		try {
			await api.fillMatrix(color);
			on = true;
		} catch (e) {
			error = String(e);
		}
	}

	async function off() {
		error = "";
		try {
			await api.fillMatrix([0, 0, 0]);
			on = false;
		} catch (e) {
			error = String(e);
		}
	}

	function onColorInput(e: Event) {
		const hex = (e.target as HTMLInputElement).value;
		color = [
			parseInt(hex.slice(1, 3), 16),
			parseInt(hex.slice(3, 5), 16),
			parseInt(hex.slice(5, 7), 16),
		];
		if (on) fill();
	}
</script>

<Card title="Underglow" {delay}>
	<p class="hint">{rows} × {cols} addressable LEDs</p>
	<div class="strip" class:on style:--color={on ? `rgb(${color.join(",")})` : "transparent"}>
		{#each Array(cols) as _, i (i)}
			<div class="led"></div>
		{/each}
	</div>
	<div class="controls-row">
		<input type="color" value={hexColor} oninput={onColorInput} />
		<button type="button" onclick={fill}>Fill</button>
		<button type="button" class="secondary" onclick={off}>Off</button>
	</div>
	{#if error}<p class="error">{error}</p>{/if}
</Card>

<style>
	.hint {
		margin: 0 0 12px;
		font-size: 12px;
		color: var(--muted);
	}

	.strip {
		display: flex;
		gap: 2px;
		height: 18px;
		margin-bottom: 14px;
	}

	.led {
		flex: 1;
		border-radius: 2px;
		background: #161719;
		transition: background 0.4s var(--ease-out);
	}

	.strip.on .led {
		background: var(--color);
		box-shadow: 0 0 6px var(--color);
	}

	.controls-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	input[type="color"] {
		width: 34px;
		height: 34px;
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 0;
		background: none;
		cursor: pointer;
	}

	button {
		background: var(--accent);
		border: none;
		color: #061a0f;
		font-weight: 600;
		font-size: 12px;
		padding: 7px 14px;
		border-radius: 7px;
		cursor: pointer;
	}

	button.secondary {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text);
	}

	.error {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--danger);
	}
</style>
