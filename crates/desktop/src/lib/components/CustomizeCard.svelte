<script lang="ts">
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import MouseDiagram from "./MouseDiagram.svelte";
	import ButtonMappingGrid from "./ButtonMappingGrid.svelte";
	import ActionPickerModal from "./ActionPickerModal.svelte";
	import { device } from "$lib/state/device.svelte";
	import { buttonMapping } from "$lib/state/buttonMapping.svelte";
	import type { MouseButtonId, ButtonAction } from "$lib/types/buttons";

	interface Props {
		delay?: number;
	}

	let { delay = 0 }: Props = $props();

	let activeTab = $state<"top" | "bottom">("top");
	let selectedButton = $state<MouseButtonId | null>(null);
	let isModalOpen = $state<boolean>(false);

	const tabs = [
		{ id: "top", label: "TOP VIEW" },
		{ id: "bottom", label: "BOTTOM VIEW" },
	] as const;

	function getSVGName(): string {
		if (!device.info) return "generic";
		const name = device.info.name.toLowerCase();
		if (name.includes("basilisk")) return "basilisk_v3";
		if (name.includes("viper")) return "viper_ultimate";
		if (name.includes("deathadder")) return "deathadder";
		if (name.includes("cobra")) return "cobra";
		if (name.includes("naga")) return "naga";
		if (name.includes("mamba")) return "mamba";
		if (name.includes("orochi") || name.includes("atheris")) return "orochi";
		return "generic";
	}

	const topSvgSrc = $derived(`/assets/${getSVGName()}.svg`);
	const bottomSvgSrc = $derived(`/assets/${getSVGName()}_bottom.svg`);
	const currentSvgSrc = $derived(activeTab === "top" ? topSvgSrc : bottomSvgSrc);

	function handleSelect(button: MouseButtonId) {
		selectedButton = button;
		isModalOpen = true;
	}

	function handleCloseModal() {
		isModalOpen = false;
		selectedButton = null;
	}

	function handleUpdateMapping(action: ButtonAction) {
		if (selectedButton) {
			buttonMapping.updateMapping(selectedButton, action);
		}
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
		isModalOpen = false;
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
		<div class="header-left">
			<h3>CUSTOMIZE</h3>
			<p class="subtitle">Click any button on the mouse or in the list to reassign its function</p>
		</div>
		<div class="header-actions">
			{#if device.info}
				<span class="device-badge">
					<span class="badge-dot"></span>
					{device.info.name}
				</span>
			{/if}
		</div>
	</div>

	<!-- View Mode Tabs (Top View & Bottom View) -->
	<div class="view-tabs-container">
		<div class="view-tabs">
			{#each tabs as tab}
				<button
					class="tab {activeTab === tab.id ? 'active' : ''}"
					onclick={() => (activeTab = tab.id)}
				>
					{#if tab.id === "top"}
						<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
							<rect x="7" y="3" width="10" height="18" rx="5" />
							<line x1="12" y1="3" x2="12" y2="9" />
						</svg>
					{:else}
						<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
							<rect x="7" y="3" width="10" height="18" rx="5" />
							<circle cx="12" cy="12" r="2" />
						</svg>
					{/if}
					{tab.label}
				</button>
			{/each}
		</div>
	</div>

	<div class="customize-content">
		<!-- Left: Interactive Mouse Diagram (Spacious and Centered) -->
		<div class="diagram-pane">
			<MouseDiagram
				src={currentSvgSrc}
				{selectedButton}
				mappings={buttonMapping.mappings}
				onSelect={handleSelect}
				viewMode={activeTab}
			/>
		</div>

		<!-- Right: Clean Button Mapping Grid -->
		<div class="grid-pane">
			<ButtonMappingGrid
				mappings={buttonMapping.mappings}
				{selectedButton}
				onSelect={handleSelect}
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

	<!-- Pop-up Function Assignment Modal -->
	<ActionPickerModal
		button={selectedButton}
		currentAction={selectedButton ? (buttonMapping.mappings[selectedButton] ?? null) : null}
		isOpen={isModalOpen}
		onSelect={handleUpdateMapping}
		onClose={handleCloseModal}
	/>
</div>

<style>
	.customize-card {
		display: flex;
		flex-direction: column;
		height: 100%;
		gap: 12px;
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: 8px;
		border-bottom: 1px solid #1f2229;
	}

	.header-left h3 {
		margin: 0;
		font-size: 14px;
		font-weight: 700;
		letter-spacing: 0.5px;
		color: #ffffff;
		text-transform: uppercase;
	}

	.subtitle {
		margin: 3px 0 0;
		font-size: 11px;
		color: #8d9098;
	}

	.device-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border-radius: 6px;
		background: #15181d;
		border: 1px solid #232731;
		font-size: 11px;
		font-weight: 600;
		color: #d1d5db;
	}

	.badge-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #40d180;
		box-shadow: 0 0 8px #40d180;
	}

	.view-tabs-container {
		display: flex;
		justify-content: center;
		margin-bottom: 4px;
	}

	.view-tabs {
		display: inline-flex;
		background: #121418;
		border: 1px solid #242831;
		border-radius: 8px;
		padding: 3px;
		gap: 4px;
	}

	.tab {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 6px 18px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: 6px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.3px;
		color: #8d9098;
		cursor: pointer;
		transition: all 140ms ease;
		user-select: none;
	}

	.tab:hover {
		color: #ffffff;
		background: rgba(255, 255, 255, 0.04);
	}

	.tab.active {
		background: #40d180;
		color: #0b0c0e;
		border-color: #40d180;
		box-shadow: 0 2px 8px rgba(64, 209, 128, 0.35);
	}

	.customize-content {
		display: grid;
		grid-template-columns: 1.3fr 1fr;
		gap: 20px;
		flex: 1;
		min-height: 0;
		align-items: start;
	}

	.diagram-pane {
		min-width: 0;
		background: #111317;
		border: 1px solid #1f232c;
		border-radius: 14px;
		padding: 16px;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.3);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.grid-pane {
		min-width: 0;
		overflow-y: auto;
		background: #111317;
		border: 1px solid #1f232c;
		border-radius: 14px;
		padding: 16px;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.3);
	}

	.status {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--muted);
		font-size: 13px;
		grid-column: 1 / -1;
	}
</style>