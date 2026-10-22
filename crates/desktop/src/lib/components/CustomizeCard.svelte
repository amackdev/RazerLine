<script lang="ts">
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import MouseDiagram from "./MouseDiagram.svelte";
	import ButtonMappingGrid from "./ButtonMappingGrid.svelte";
	import { device } from "$lib/state/device.svelte";
	import { buttonMapping } from "$lib/state/buttonMapping.svelte";
	import type { MouseButtonId, ButtonAction } from "$lib/types/buttons";
	import { MouseButtonId as MB } from "$lib/types/buttons";

	interface Props {
		delay?: number;
	}

	let { delay = 0 }: Props = $props();

	let activeTab = $state<"top" | "bottom">("top");
	let selectedButton = $state<MouseButtonId | null>(null);

	const tabs = [
		{ id: "top", label: "TOP VIEW" },
		{ id: "bottom", label: "BOTTOM VIEW" },
	] as const;

	function getSVGName(): string {
		if (!device.info) return "generic";
		const name = device.info.name.toLowerCase();
		if (name.includes("basilisk")) return "basilisk_v3";
		if (name.includes("viper ultimate")) return "viper_ultimate";
		return "generic";
	}

	const svgSrc = $derived(`/assets/${getSVGName()}${activeTab === "bottom" ? "_bottom" : ""}.svg`);

	function handleSelect(button: MouseButtonId) {
		selectedButton = selectedButton === button ? null : button;
	}

	function handleUpdateMapping(button: MouseButtonId, action: ButtonAction) {
		buttonMapping.updateMapping(button, action);
	}

	function handleReset() {
		buttonMapping.resetToDefaults();
	}

	// Sync with device state
	$effect(() => {
		if (device.info?.product_id) {
			buttonMapping.loadMappings(device.info.product_id);
		}
	});

	// Reset selection when device changes
	$effect(() => {
		device.info;
		selectedButton = null;
		activeTab = "top";
	});

	onMount(() => {
		if (device.info?.product_id) {
			buttonMapping.loadMappings(device.info.product_id);
		}
	});
</script>

<div class="customize-card" in:fly={{ y: 8, duration: 280, delay }}>
	<div class="card-header">
		<h3>CUSTOMIZE</h3>
		<div class="header-actions">
			{#if device.info}
				<span class="device-badge">{device.info.name}</span>
			{/if}
		</div>
	</div>

	<!-- View Mode Tabs -->
	<div class="view-tabs">
		{#each tabs as tab}
			<button
				class="tab {activeTab === tab.id ? 'active' : ''}"
				onclick={() => (activeTab = tab.id as "top" | "bottom")}
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<div class="customize-content">
		<!-- Left: Mouse Diagram -->
		<div class="diagram-pane">
			<MouseDiagram
				src={svgSrc}
				selectedButton={selectedButton}
				mappings={buttonMapping.mappings}
				onSelect={handleSelect}
				viewMode={activeTab}
			/>
		</div>

		<!-- Right: Button Mapping Grid -->
		<div class="grid-pane">
			<ButtonMappingGrid
				mappings={buttonMapping.mappings}
				selectedButton={selectedButton}
				onSelect={handleSelect}
				onUpdate={handleUpdateMapping}
				onReset={handleReset}
			/>
		</div>
	</div>

	<!-- No device state -->
	{#if !device.checked}
		<div class="status" in:fade={{ duration: 200 }}>Looking for a device…</div>
	{:else if !device.info}
		<div class="status" in:fade={{ duration: 200 }}>No Razer device connected</div>
	{/if}
</div>

<style>
	.customize-card {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}

	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
	}

	.card-header h3 {
		margin: 0;
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.device-badge {
		font-size: 11px;
		font-weight: 500;
		padding: 4px 10px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--accent);
	}

	.view-tabs {
		display: flex;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 4px;
		margin-bottom: 16px;
	}

	.tab {
		flex: 1;
		padding: 8px 16px;
		background: transparent;
		border: none;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--muted);
		cursor: pointer;
		transition: background 150ms, color 150ms;
	}

	.tab:hover {
		color: var(--fg);
		background: var(--surface-elevated);
	}

	.tab.active {
		background: var(--accent);
		color: var(--bg);
	}

	.customize-content {
		display: grid;
		grid-template-columns: 1fr 1.2fr;
		gap: 24px;
		flex: 1;
		min-height: 0;
	}

	.diagram-pane {
		min-width: 0;
	}

	.grid-pane {
		min-width: 0;
		overflow-y: auto;
	}

	.status {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--muted);
		font-size: 14px;
		grid-column: 1 / -1;
	}

	@media (max-width: 900px) {
		.customize-content {
			grid-template-columns: 1fr;
		}
	}
</style>