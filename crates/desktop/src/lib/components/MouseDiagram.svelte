<script lang="ts">
	import { fade } from "svelte/transition";
	import type { MouseButtonId, ButtonAction } from "$lib/types/buttons";
	import { MouseButtonId as MB } from "$lib/types/buttons";
	import { buttonActionLabel } from "$lib/types/buttons";

	interface Props {
		/** SVG source URL (from /assets/) */
		src: string;
		/** Currently selected button for editing */
		selectedButton: MouseButtonId | null;
		/** Current action for each button */
		mappings: Record<string, ButtonAction>;
		/** Callback when a button pin or callout is clicked */
		onSelect: (button: MouseButtonId) => void;
		/** View mode */
		viewMode: "top" | "bottom";
		/** Compact mode for dual side-by-side view */
		compact?: boolean;
	}

	let { src, selectedButton, mappings, onSelect, viewMode, compact = false }: Props = $props();

	interface Hotspot {
		x: number; // percentage 0-100 across mouse width
		y: number; // percentage 0-100 across mouse height
		label: string;
		side: "left" | "right";
		badgeY: number; // percentage vertical anchor for the badge
	}

	// Hotspots for each chassis family (top and bottom views)
	const hotspots: Record<string, Record<string, Hotspot>> = {
		// Basilisk V3 Top View
		"basilisk_v3-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up", side: "right", badgeY: 14 },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click", side: "left", badgeY: 18 },
			TiltLeft: { x: 44, y: 27, label: "Scroll Left", side: "left", badgeY: 26 },
			TiltRight: { x: 56, y: 27, label: "Scroll Right", side: "right", badgeY: 22 },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down", side: "right", badgeY: 30 },
			DpiSwitch: { x: 50, y: 41, label: "DPI Cycle", side: "right", badgeY: 38 },
			ScrollMode: { x: 50, y: 47, label: "Scroll Mode", side: "right", badgeY: 46 },
			SideForward: { x: 30, y: 35, label: "Mouse Button 5", side: "left", badgeY: 34 },
			SideBack: { x: 30, y: 43, label: "Mouse Button 4", side: "left", badgeY: 42 },
			MultiFunctionClutch: { x: 26, y: 50, label: "Sensitivity Clutch", side: "left", badgeY: 50 },
		},
		"basilisk_v3-bottom": {
			ProfileSwitch: { x: 43, y: 58, label: "Cycle Profile", side: "left", badgeY: 58 },
		},

		// Viper Ultimate Top View
		"viper_ultimate-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click", side: "left", badgeY: 20 },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down", side: "right", badgeY: 26 },
			DpiSwitch: { x: 50, y: 44, label: "DPI Switch", side: "right", badgeY: 36 },
			SideForward: { x: 32, y: 36, label: "Mouse Button 5", side: "left", badgeY: 34 },
			SideBack: { x: 32, y: 44, label: "Mouse Button 4", side: "left", badgeY: 44 },
		},
		"viper_ultimate-bottom": {
			ProfileSwitch: { x: 42, y: 56, label: "Cycle Profile", side: "left", badgeY: 56 },
		},

		// DeathAdder Top View
		"deathadder-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click", side: "left", badgeY: 20 },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down", side: "right", badgeY: 26 },
			DpiSwitch: { x: 50, y: 40, label: "DPI Up", side: "right", badgeY: 36 },
			ScrollMode: { x: 50, y: 47, label: "DPI Down", side: "right", badgeY: 46 },
			SideForward: { x: 28, y: 45, label: "Mouse Button 5", side: "left", badgeY: 38 },
			SideBack: { x: 28, y: 55, label: "Mouse Button 4", side: "left", badgeY: 48 },
		},
		"deathadder-bottom": {
			ProfileSwitch: { x: 41, y: 51, label: "Cycle Profile", side: "left", badgeY: 51 },
		},

		// Cobra Top View
		"cobra-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click", side: "left", badgeY: 20 },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down", side: "right", badgeY: 26 },
			DpiSwitch: { x: 50, y: 43, label: "DPI Cycle", side: "right", badgeY: 36 },
			SideForward: { x: 33, y: 44, label: "Mouse Button 5", side: "left", badgeY: 38 },
			SideBack: { x: 33, y: 52, label: "Mouse Button 4", side: "left", badgeY: 48 },
		},
		"cobra-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile", side: "left", badgeY: 50 },
		},

		// Naga Top View
		"naga-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click", side: "left", badgeY: 20 },
			TiltLeft: { x: 44, y: 28, label: "Scroll Left", side: "left", badgeY: 28 },
			TiltRight: { x: 56, y: 28, label: "Scroll Right", side: "right", badgeY: 24 },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down", side: "right", badgeY: 32 },
			DpiSwitch: { x: 50, y: 41, label: "DPI Up", side: "right", badgeY: 40 },
			ScrollMode: { x: 50, y: 47, label: "DPI Down", side: "right", badgeY: 48 },
			SideForward: { x: 28, y: 46, label: "Side Button 5", side: "left", badgeY: 42 },
			SideBack: { x: 28, y: 54, label: "Side Button 4", side: "left", badgeY: 52 },
		},
		"naga-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile", side: "left", badgeY: 50 },
		},

		// Mamba Top View
		"mamba-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click", side: "left", badgeY: 20 },
			TiltLeft: { x: 44, y: 27, label: "Scroll Left", side: "left", badgeY: 28 },
			TiltRight: { x: 56, y: 27, label: "Scroll Right", side: "right", badgeY: 24 },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down", side: "right", badgeY: 32 },
			DpiSwitch: { x: 50, y: 40, label: "DPI Up", side: "right", badgeY: 40 },
			ScrollMode: { x: 50, y: 46, label: "DPI Down", side: "right", badgeY: 48 },
			SideForward: { x: 30, y: 45, label: "Mouse Button 5", side: "left", badgeY: 42 },
			SideBack: { x: 30, y: 53, label: "Mouse Button 4", side: "left", badgeY: 52 },
		},
		"mamba-bottom": {
			ProfileSwitch: { x: 41, y: 51, label: "Cycle Profile", side: "left", badgeY: 51 },
		},

		// Orochi Top View
		"orochi-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click", side: "left", badgeY: 20 },
			ScrollDown: { x: 50, y: 33, label: "Scroll Down", side: "right", badgeY: 26 },
			DpiSwitch: { x: 50, y: 42, label: "DPI Button", side: "right", badgeY: 36 },
			SideForward: { x: 32, y: 45, label: "Mouse Button 5", side: "left", badgeY: 40 },
			SideBack: { x: 32, y: 52, label: "Mouse Button 4", side: "left", badgeY: 50 },
		},
		"orochi-bottom": {},

		// Generic Fallback
		"generic-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up", side: "right", badgeY: 16 },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click", side: "left", badgeY: 20 },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down", side: "right", badgeY: 26 },
			DpiSwitch: { x: 50, y: 42, label: "DPI Switch", side: "right", badgeY: 36 },
			SideForward: { x: 31, y: 45, label: "Mouse Button 5", side: "left", badgeY: 40 },
			SideBack: { x: 31, y: 53, label: "Mouse Button 4", side: "left", badgeY: 50 },
		},
		"generic-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile", side: "left", badgeY: 50 },
		},
	};

	// Clean extraction of device key e.g. "viper_ultimate-top"
	let baseName = $derived(
		src
			.split("/")
			.pop()
			?.replace(/(_bottom)?\.(svg|png)$/, "") ?? "generic"
	);
	let key = $derived(`${baseName}-${viewMode}`);
	let deviceHotspots = $derived(hotspots[key] ?? hotspots[`generic-${viewMode}`] ?? {});

	function getAction(button: MouseButtonId): ButtonAction {
		return mappings[button] ?? MB.defaultAction(button);
	}
</script>

<div class="mouse-stage" class:compact={compact}>
	<!-- Ambient Backlight Stage -->
	<div class="stage-backdrop"></div>

	<!-- Central SVG Chassis -->
	<div class="chassis-wrapper">
		<img {src} alt="Mouse Chassis Diagram" class="mouse-svg" />

		<!-- Interactive Hotspot Pins and Connected Callout Badges -->
		{#each Object.entries(deviceHotspots) as [buttonId, hotspot]}
			{@const isSelected = selectedButton === buttonId}
			{@const btnId = buttonId as MouseButtonId}
			{@const action = getAction(btnId)}

			<!-- Hotspot Pin on the Mouse Surface -->
			<button
				class="mouse-pin"
				class:selected={isSelected}
				style="left: {hotspot.x}%; top: {hotspot.y}%;"
				onclick={() => onSelect(btnId)}
				title="Click to assign: {hotspot.label}"
			>
				{#if isSelected}
					<span class="pin-halo"></span>
				{/if}
				<span class="pin-core"></span>
			</button>

			<!-- Callout Badge anchored to side -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="callout-card {hotspot.side}"
				class:selected={isSelected}
				style="top: {hotspot.badgeY}%;"
				onclick={() => onSelect(btnId)}
			>
				<div class="callout-content">
					<div class="callout-header">
						<span class="callout-label">{hotspot.label}</span>
						<span class="edit-hint">Click to change</span>
					</div>
					<div class="callout-action-pill">
						<span class="pill-dot"></span>
						<span class="pill-text">{buttonActionLabel(action)}</span>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	.mouse-stage {
		position: relative;
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 20px 0;
		min-height: 520px;
	}

	.stage-backdrop {
		position: absolute;
		inset: 20px;
		background: radial-gradient(circle at 50% 50%, rgba(64, 209, 128, 0.08) 0%, rgba(15, 16, 18, 0) 70%);
		pointer-events: none;
		border-radius: 24px;
	}

	.chassis-wrapper {
		position: relative;
		width: 100%;
		max-width: 540px;
		height: auto;
		display: flex;
		justify-content: center;
		user-select: none;
	}

	.mouse-svg {
		display: block;
		width: 100%;
		max-width: 440px;
		height: auto;
		filter: drop-shadow(0 14px 35px rgba(0, 0, 0, 0.7));
		pointer-events: none;
	}

	/* Hotspot Pin on Mouse Body */
	.mouse-pin {
		position: absolute;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: transparent;
		border: none;
		padding: 0;
		cursor: pointer;
		transform: translate(-50%, -50%);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 25;
		transition: transform 150ms cubic-bezier(0.16, 1, 0.3, 1);
	}

	.mouse-pin:hover {
		transform: translate(-50%, -50%) scale(1.3);
	}

	.pin-core {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: #40d180;
		box-shadow: 0 0 10px #40d180, 0 0 20px rgba(64, 209, 128, 0.6);
		border: 2px solid #14161a;
		transition: all 150ms;
	}

	.mouse-pin:hover .pin-core,
	.mouse-pin.selected .pin-core {
		background: #ffffff;
		box-shadow: 0 0 14px #40d180, 0 0 25px #40d180;
		transform: scale(1.2);
	}

	.pin-halo {
		position: absolute;
		inset: -6px;
		border-radius: 50%;
		border: 2px solid #40d180;
		animation: haloPulse 1.8s infinite cubic-bezier(0.25, 0.8, 0.25, 1);
	}

	@keyframes haloPulse {
		0% { transform: scale(0.8); opacity: 0.8; }
		50% { transform: scale(1.6); opacity: 0; }
		100% { transform: scale(0.8); opacity: 0; }
	}

	/* Callout Badges Anchored to Sides */
	.callout-card {
		position: absolute;
		background: #181b20;
		border: 1px solid #282c35;
		border-radius: 10px;
		padding: 8px 12px;
		width: 165px;
		cursor: pointer;
		box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
		transition: all 160ms cubic-bezier(0.16, 1, 0.3, 1);
		z-index: 20;
		user-select: none;
	}

	.callout-card.left {
		left: 0;
		transform: translateY(-50%);
	}

	.callout-card.right {
		right: 0;
		transform: translateY(-50%);
	}

	.callout-card:hover {
		border-color: #40d180;
		background: #20242a;
		transform: translateY(-50%) scale(1.03);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6), 0 0 12px rgba(64, 209, 128, 0.15);
	}

	.callout-card.selected {
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.12);
		box-shadow: 0 0 0 1px #40d180, 0 8px 24px rgba(0, 0, 0, 0.6);
	}

	.callout-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.callout-label {
		font-size: 11px;
		font-weight: 600;
		color: #e5e8ec;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.edit-hint {
		font-size: 9px;
		color: #8d9098;
		opacity: 0;
		transition: opacity 120ms;
	}

	.callout-card:hover .edit-hint {
		opacity: 1;
		color: #40d180;
	}

	.callout-action-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 2px 7px;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.08);
		max-width: 100%;
	}

	.callout-card.selected .callout-action-pill {
		background: rgba(64, 209, 128, 0.18);
		border-color: rgba(64, 209, 128, 0.35);
	}

	.pill-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: #40d180;
		flex-shrink: 0;
	}

	.pill-text {
		font-size: 10px;
		font-weight: 500;
		color: #40d180;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	/* Compact mode for side-by-side / dual view */
	.mouse-stage.compact {
		min-height: 420px;
		padding: 10px 0;
	}

	.mouse-stage.compact .chassis-wrapper {
		max-width: 380px;
	}

	.mouse-stage.compact .mouse-svg {
		max-width: 320px;
	}

	.mouse-stage.compact .callout-card {
		width: 120px;
		padding: 5px 8px;
	}

	.mouse-stage.compact .callout-label {
		font-size: 10px;
	}

	.mouse-stage.compact .edit-hint {
		display: none;
	}

	.mouse-stage.compact .pill-text {
		font-size: 9px;
	}
</style>