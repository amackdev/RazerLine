<script lang="ts">
	import Card from "./Card.svelte";
	import { api } from "../api";

	let {
		hasIdleTime,
		idleMinSeconds,
		idleMaxSeconds,
		hasLowBatteryThreshold,
		delay = 0,
	}: {
		hasIdleTime: boolean;
		idleMinSeconds: number | null;
		idleMaxSeconds: number | null;
		hasLowBatteryThreshold: boolean;
		delay?: number;
	} = $props();

	let minMinutes = $derived(
		hasIdleTime ? Math.max(1, Math.ceil((idleMinSeconds ?? 60) / 60)) : 1,
	);
	let maxMinutes = $derived(hasIdleTime ? Math.floor((idleMaxSeconds ?? 900) / 60) : 15);

	let idleMinutes = $state(10);
	let idleStatus = $state("");
	let thresholdPercent = $state(15);
	let thresholdStatus = $state("");

	$effect(() => {
		if (hasIdleTime) {
			api
				.getIdleTime()
				.then((s) => (idleMinutes = Math.max(minMinutes, Math.round(s / 60))))
				.catch(() => {});
		}
		if (hasLowBatteryThreshold) {
			api
				.getLowBatteryThreshold()
				.then((p) => (thresholdPercent = p))
				.catch(() => {});
		}
	});

	async function applyIdle() {
		idleStatus = "";
		try {
			await api.setIdleTime(idleMinutes * 60);
			idleStatus = `Sleep after ${idleMinutes} min idle`;
		} catch (e) {
			idleStatus = String(e);
		}
	}

	async function applyThreshold() {
		thresholdStatus = "";
		try {
			await api.setLowBatteryThreshold(thresholdPercent);
			thresholdStatus = `Threshold set to ${thresholdPercent}%`;
		} catch (e) {
			thresholdStatus = String(e);
		}
	}
</script>

<Card title="Power Management" {delay}>
	{#if hasIdleTime}
		<div class="row">
			<div class="label-group">
				<span class="label">Wireless Power Saving</span>
				<span class="hint">
					Sleep after this many minutes idle ({minMinutes}-{maxMinutes})
				</span>
			</div>
			<div class="controls">
				<input type="number" min={minMinutes} max={maxMinutes} bind:value={idleMinutes} />
				<button type="button" onclick={applyIdle}>Apply</button>
			</div>
		</div>
		{#if idleStatus}<p class="status">{idleStatus}</p>{/if}
	{/if}

	{#if hasLowBatteryThreshold}
		<div class="row" class:spaced={hasIdleTime}>
			<div class="label-group">
				<span class="label">Low Power Mode</span>
				<span class="hint">Warn once battery drops below this percent (5-25%)</span>
			</div>
			<div class="controls">
				<input type="number" min="5" max="25" bind:value={thresholdPercent} />
				<button type="button" onclick={applyThreshold}>Apply</button>
			</div>
		</div>
		{#if thresholdStatus}<p class="status">{thresholdStatus}</p>{/if}
	{/if}
</Card>

<style>
	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}

	.row.spaced {
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid var(--border);
	}

	.label-group {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.label {
		font-size: 13px;
		font-weight: 600;
	}

	.hint {
		font-size: 11.5px;
		color: var(--muted);
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	input[type="number"] {
		width: 64px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 7px;
		color: var(--text);
		padding: 6px 8px;
		font-size: 13px;
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
		transition: filter 0.2s;
	}

	button:hover {
		filter: brightness(1.1);
	}

	.status {
		margin: 8px 0 0;
		font-size: 11.5px;
		color: var(--text-dim);
	}
</style>
