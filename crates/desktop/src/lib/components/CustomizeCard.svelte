<script lang="ts">
	import { onMount } from "svelte";
	import { fly } from "svelte/transition";
	import MouseDiagram from "./MouseDiagram.svelte";
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
			<h3>BUTTON ASSIGNMENTS</h3>
			<p class="subtitle">Click any button pin on the mouse chassis to reassign its function</p>
		</div>
		<div class="header-actions">
			<button class="btn btn-ghost btn-sm" onclick={handleReset} title="Reset all button mappings to factory defaults">
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
					<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
					<path d="M3 3v5h5" />
				</svg>
				Reset Defaults
			</button>
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

	<!-- Interactive Mouse Diagram (Front, Center, and Full Width) -->
	<div class="diagram-container">
		<MouseDiagram
			src={currentSvgSrc}
			{selectedButton}
			mappings={buttonMapping.mappings}
			onSelect={handleSelect}
			viewMode={activeTab}
		/>
	</div>

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
		background: var(--card);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: 10px;
		border-bottom: 1px solid #232731;
	}

	.header-left h3 {
		margin: 0;
		font-size: 13px;
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

	.view-tabs-container {
		display: flex;
		justify-content: center;
		margin: 2px 0;
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
		padding: 6px 20px;
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

	.diagram-container {
		display: flex;
		align-items: center;
		justify-content: center;
		background: #111317;
		border: 1px solid #1f232c;
		border-radius: 14px;
		padding: 16px;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.3);
	}
</style>