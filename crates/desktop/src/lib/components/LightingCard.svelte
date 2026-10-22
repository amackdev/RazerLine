<script lang="ts">
	import { untrack } from "svelte";
	import Card from "./Card.svelte";
	import EffectPreview from "./EffectPreview.svelte";
	import { api } from "../api";
	import type { LedZoneInfo } from "../state/device.svelte";

	let {
		zones,
		unsupportedEffects,
		delay = 0,
	}: { zones: LedZoneInfo[]; unsupportedEffects: string[]; delay?: number } = $props();

	const allEffects = [
		{ id: "static", label: "Static" },
		{ id: "breathing", label: "Breathing" },
		{ id: "spectrum", label: "Spectrum" },
		{ id: "wave", label: "Wave" },
		{ id: "off", label: "Off" },
	];
	let effects = allEffects.filter((e) => !unsupportedEffects.includes(e.id));

	let syncAll = $state(true);
	let selectedZoneId = $state(untrack(() => zones[0]?.id ?? 0));
	let color = $state<[number, number, number]>([64, 209, 128]);
	let effect = $state("static");
	let brightness = $state(100);
	let error = $state("");

	let selectedZone = $derived(zones.find((z) => z.id === selectedZoneId));
	let anyBrightness = $derived(zones.some((z) => z.has_brightness));

	async function applyEffect(next: string) {
		error = "";
		effect = next;
		try {
			if (syncAll) {
				if (next === "static") await api.setLedStaticAll(color);
				else if (next === "breathing") await api.setLedBreathingAll(color);
				else if (next === "spectrum") await api.setLedSpectrumAll();
				else if (next === "wave") await api.setLedWaveAll(0);
				else await api.setLedOffAll();
			} else {
				if (next === "static") await api.setLedStatic(selectedZoneId, color);
				else if (next === "breathing") await api.setLedBreathing(selectedZoneId, color);
				else if (next === "spectrum") await api.setLedSpectrum(selectedZoneId);
				else if (next === "wave") await api.setLedWave(selectedZoneId, 0);
				else await api.setLedOff(selectedZoneId);
			}
		} catch (e) {
			error = String(e);
		}
	}

	async function applyColor() {
		if (effect !== "static" && effect !== "breathing") return;
		await applyEffect(effect);
	}

	async function applyBrightness() {
		error = "";
		try {
			if (syncAll) await api.setLedBrightnessAll(brightness);
			else await api.setLedBrightness(selectedZoneId, brightness);
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
		applyColor();
	}

	let hexColor = $derived(
		"#" + color.map((c) => c.toString(16).padStart(2, "0")).join(""),
	);
</script>

<Card title="Lighting" {delay}>
	<div class="top-row">
		<div class="toggle">
			<button type="button" class:active={syncAll} onclick={() => (syncAll = true)}>
				All Zones
			</button>
			<button type="button" class:active={!syncAll} onclick={() => (syncAll = false)}>
				Individually
			</button>
		</div>

		{#if !syncAll && zones.length > 1}
			<select bind:value={selectedZoneId}>
				{#each zones as zone (zone.id)}
					<option value={zone.id}>{zone.name}</option>
				{/each}
			</select>
		{/if}
	</div>

	<EffectPreview {effect} {color} />

	<div class="controls-row">
		<input type="color" value={hexColor} oninput={onColorInput} />
		<div class="effects">
			{#each effects as e (e.id)}
				<button type="button" class:active={effect === e.id} onclick={() => applyEffect(e.id)}>
					{e.label}
				</button>
			{/each}
		</div>
	</div>

	{#if anyBrightness && (syncAll || selectedZone?.has_brightness)}
		<div class="brightness-row">
			<span class="label">Brightness</span>
			<input
				type="range"
				min="0"
				max="100"
				bind:value={brightness}
				onchange={applyBrightness}
			/>
			<span class="value">{brightness}%</span>
		</div>
	{/if}

	{#if error}<p class="error">{error}</p>{/if}
</Card>

<style>
	.top-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 14px;
	}

	.toggle {
		display: inline-flex;
		gap: 2px;
		padding: 3px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
	}

	.toggle button {
		background: transparent;
		border: none;
		padding: 5px 12px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 600;
		color: var(--text-dim);
		cursor: pointer;
		transition: all 0.2s;
	}

	.toggle button.active {
		background: var(--accent);
		color: #061a0f;
	}

	select {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text);
		font-size: 12px;
		padding: 5px 8px;
		border-radius: 7px;
	}

	.controls-row {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 14px;
	}

	input[type="color"] {
		width: 34px;
		height: 34px;
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 0;
		background: none;
		cursor: pointer;
		flex-shrink: 0;
	}

	.effects {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.effects button {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text-dim);
		font-size: 12px;
		font-weight: 600;
		padding: 6px 12px;
		border-radius: 7px;
		cursor: pointer;
		transition: all 0.2s var(--ease-out);
	}

	.effects button:hover {
		border-color: var(--border-strong);
		color: var(--text);
	}

	.effects button.active {
		background: var(--accent);
		border-color: var(--accent);
		color: #061a0f;
	}

	.brightness-row {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid var(--border);
	}

	.brightness-row .label {
		font-size: 12px;
		font-weight: 600;
		flex-shrink: 0;
	}

	.brightness-row input[type="range"] {
		flex: 1;
		accent-color: var(--accent);
	}

	.brightness-row .value {
		font-size: 12px;
		color: var(--muted);
		width: 36px;
		text-align: right;
		flex-shrink: 0;
	}

	.error {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--danger);
	}
</style>
