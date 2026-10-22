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

	let activeTab = $state<"both" | "top" | "bottom">("both");
	let selectedButton = $state<MouseButtonId | null>(null);
	let isModalOpen = $state<boolean>(false);

	const tabs = [
		{ id: "both", label: "TOP & BOTTOM" },
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
	const svgSrc = $derived(`/assets/${getSVGName()}${activeTab === "bottom" ? "_bottom" : ""}.svg`);

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
		activeTab = "both";
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
			<p class="subtitle">Click any button on the mouse to assign custom actions and shortcuts</p>
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

	<!-- View Mode Tabs (Both, Top, Bottom) -->
	<div class="view-tabs-container">
		<div class="view-tabs">
			{#each tabs as tab}
				<button
					class="tab {activeTab === tab.id ? 'active' : ''}"
					onclick={() => (activeTab = tab.id)}
				>
					{#if tab.id === "both"}
						<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
							<rect x="3" y="4" width="8" height="16" rx="3" />
							<rect x="13" y="4" width="8" height="16" rx="3" />
						</svg>
					{:else if tab.id === "top"}
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

	<div class="customize-content" class:dual-layout={activeTab === "both"}>
		<!-- Left: Interactive Mouse Diagram (Front and Center) -->
		<div class="diagram-pane" class:dual={activeTab === "both"}>
			{#if activeTab === "both"}
				<div class="dual-container">
					<div class="dual-column">
						<div class="dual-tag">TOP VIEW</div>
						<MouseDiagram
							src={topSvgSrc}
							{selectedButton}
							mappings={buttonMapping.mappings}
							onSelect={handleSelect}
							viewMode="top"
							compact={true}
						/>
					</div>
					<div class="dual-column">
						<div class="dual-tag">BOTTOM VIEW</div>
						<MouseDiagram
							src={bottomSvgSrc}
							{selectedButton}
							mappings={buttonMapping.mappings}
							onSelect={handleSelect}
							viewMode="bottom"
							compact={true}
						/>
					</div>
				</div>
			{:else}
				<MouseDiagram
					src={svgSrc}
					{selectedButton}
					mappings={buttonMapping.mappings}
					onSelect={handleSelect}
					viewMode={activeTab}
				/>
			{/if}
		</div>

		<!-- Right: Quick Button Mapping Sidebar -->
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
		min-height: 0;
	}

	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 14px;
	}

	.header-left h3 {
		margin: 0;
		font-size: 13px;
		font-weight: 700;
		letter-spacing: 0.5px;
		color: var(--muted, #8d9098);
		text-transform: uppercase;
	}

	.subtitle {
		margin: 2px 0 0;
		font-size: 11px;
		color: #8d9098;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.device-badge {
		font-size: 11px;
		font-weight: 600;
		padding: 4px 12px;
		background: rgba(64, 209, 128, 0.1);
		border: 1px solid rgba(64, 209, 128, 0.3);
		border-radius: 20px;
		color: #40d180;
		display: inline-flex;
		align-items: center;
		gap: 6px;
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
		margin-bottom: 12px;
	}

	.view-tabs {
		display: inline-flex;
		background: #14161a;
		border: 1px solid #282c35;
		border-radius: 10px;
		padding: 4px;
		gap: 4px;
	}

	.tab {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 7px 22px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: 8px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.3px;
		color: #8d9098;
		cursor: pointer;
		transition: all 150ms cubic-bezier(0.16, 1, 0.3, 1);
		user-select: none;
	}

	.tab:hover {
		color: #ffffff;
		background: rgba(255, 255, 255, 0.05);
	}

	.tab.active {
		background: #40d180;
		color: #0b0c0e;
		border-color: #40d180;
		box-shadow: 0 2px 8px rgba(64, 209, 128, 0.35);
	}

	.customize-content {
		display: grid;
		grid-template-columns: 1.15fr 1fr;
		gap: 28px;
		flex: 1;
		min-height: 0;
		align-items: start;
	}

	.customize-content.dual-layout {
		grid-template-columns: 1.45fr 1fr;
		gap: 20px;
	}

	.diagram-pane {
		min-width: 0;
		background: #131518;
		border: 1px solid #22252d;
		border-radius: 16px;
		padding: 16px;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.3);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.diagram-pane.dual {
		padding: 12px;
	}

	.dual-container {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
		width: 100%;
	}

	.dual-column {
		display: flex;
		flex-direction: column;
		align-items: center;
		position: relative;
		background: #101215;
		border: 1px solid #1f2229;
		border-radius: 14px;
		padding: 10px 4px;
	}

	.dual-tag {
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: #40d180;
		background: rgba(64, 209, 128, 0.12);
		border: 1px solid rgba(64, 209, 128, 0.25);
		padding: 3px 12px;
		border-radius: 20px;
		margin-bottom: 2px;
	}

	.grid-pane {
		min-width: 0;
		overflow-y: auto;
		background: #131518;
		border: 1px solid #22252d;
		border-radius: 16px;
		padding: 16px 18px;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.3);
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

	@media (max-width: 960px) {
		.customize-content {
			grid-template-columns: 1fr;
		}
	}
</style>