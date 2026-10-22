<script lang="ts">
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
		/** Callback when a button pin is clicked */
		onSelect: (button: MouseButtonId) => void;
		/** View mode */
		viewMode: "top" | "bottom";
	}

	let { src, selectedButton, mappings, onSelect, viewMode }: Props = $props();

	interface Hotspot {
		x: number; // percentage 0-100 across mouse width
		y: number; // percentage 0-100 across mouse height
		label: string;
	}

	// Hotspots for each chassis family (top and bottom views)
	const hotspots: Record<string, Record<string, Hotspot>> = {
		// Basilisk V3 Top View
		"basilisk_v3-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click" },
			TiltLeft: { x: 44, y: 27, label: "Scroll Left" },
			TiltRight: { x: 56, y: 27, label: "Scroll Right" },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 41, label: "DPI Cycle" },
			ScrollMode: { x: 50, y: 47, label: "Scroll Mode" },
			SideForward: { x: 30, y: 35, label: "Mouse Button 5" },
			SideBack: { x: 30, y: 43, label: "Mouse Button 4" },
			MultiFunctionClutch: { x: 26, y: 50, label: "Sensitivity Clutch" },
		},
		"basilisk_v3-bottom": {
			ProfileSwitch: { x: 43, y: 58, label: "Cycle Profile" },
		},

		// Viper Ultimate Top View
		"viper_ultimate-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click" },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 44, label: "DPI Switch" },
			SideForward: { x: 32, y: 36, label: "Mouse Button 5" },
			SideBack: { x: 32, y: 44, label: "Mouse Button 4" },
		},
		"viper_ultimate-bottom": {
			ProfileSwitch: { x: 42, y: 56, label: "Cycle Profile" },
		},

		// DeathAdder Top View
		"deathadder-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click" },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 40, label: "DPI Up" },
			ScrollMode: { x: 50, y: 47, label: "DPI Down" },
			SideForward: { x: 28, y: 45, label: "Mouse Button 5" },
			SideBack: { x: 28, y: 55, label: "Mouse Button 4" },
		},
		"deathadder-bottom": {
			ProfileSwitch: { x: 41, y: 51, label: "Cycle Profile" },
		},

		// Cobra Top View
		"cobra-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click" },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 43, label: "DPI Cycle" },
			SideForward: { x: 33, y: 44, label: "Mouse Button 5" },
			SideBack: { x: 33, y: 52, label: "Mouse Button 4" },
		},
		"cobra-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile" },
		},

		// Naga Top View
		"naga-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click" },
			TiltLeft: { x: 44, y: 28, label: "Scroll Left" },
			TiltRight: { x: 56, y: 28, label: "Scroll Right" },
			ScrollDown: { x: 50, y: 34, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 41, label: "DPI Up" },
			ScrollMode: { x: 50, y: 47, label: "DPI Down" },
			SideForward: { x: 28, y: 46, label: "Side Button 5" },
			SideBack: { x: 28, y: 54, label: "Side Button 4" },
		},
		"naga-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile" },
		},

		// Mamba Top View
		"mamba-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click" },
			TiltLeft: { x: 44, y: 27, label: "Scroll Left" },
			TiltRight: { x: 56, y: 27, label: "Scroll Right" },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 40, label: "DPI Up" },
			ScrollMode: { x: 50, y: 46, label: "DPI Down" },
			SideForward: { x: 30, y: 45, label: "Mouse Button 5" },
			SideBack: { x: 30, y: 53, label: "Mouse Button 4" },
		},
		"mamba-bottom": {
			ProfileSwitch: { x: 41, y: 51, label: "Cycle Profile" },
		},

		// Orochi Top View
		"orochi-top": {
			ScrollUp: { x: 50, y: 23, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 28, label: "Scroll Click" },
			ScrollDown: { x: 50, y: 33, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 42, label: "DPI Button" },
			SideForward: { x: 32, y: 45, label: "Mouse Button 5" },
			SideBack: { x: 32, y: 52, label: "Mouse Button 4" },
		},
		"orochi-bottom": {},

		// Generic Fallback
		"generic-top": {
			ScrollUp: { x: 50, y: 22, label: "Scroll Up" },
			MiddleClick: { x: 50, y: 27, label: "Scroll Click" },
			ScrollDown: { x: 50, y: 32, label: "Scroll Down" },
			DpiSwitch: { x: 50, y: 42, label: "DPI Switch" },
			SideForward: { x: 31, y: 45, label: "Mouse Button 5" },
			SideBack: { x: 31, y: 53, label: "Mouse Button 4" },
		},
		"generic-bottom": {
			ProfileSwitch: { x: 41, y: 50, label: "Cycle Profile" },
		},
	};

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

<div class="mouse-stage">
	<!-- Ambient Backlight Stage -->
	<div class="stage-backdrop"></div>

	<!-- Central SVG Chassis -->
	<div class="chassis-wrapper">
		<img {src} alt="Mouse Chassis Diagram" class="mouse-svg" />

		<!-- Interactive Numbered Hotspot Pins on Mouse Surface -->
		{#each Object.entries(deviceHotspots) as [buttonId, hotspot], idx}
			{@const isSelected = selectedButton === buttonId}
			{@const btnId = buttonId as MouseButtonId}
			{@const action = getAction(btnId)}

			<button
				class="mouse-pin"
				class:selected={isSelected}
				style="left: {hotspot.x}%; top: {hotspot.y}%;"
				onclick={() => onSelect(btnId)}
			>
				{#if isSelected}
					<span class="pin-halo"></span>
				{/if}
				<span class="pin-num">{idx + 1}</span>

				<!-- Hover Tooltip -->
				<div class="pin-tooltip">
					<div class="tooltip-title">{hotspot.label}</div>
					<div class="tooltip-action">{buttonActionLabel(action)}</div>
				</div>
			</button>
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
		padding: 24px 0;
		min-height: 480px;
	}

	.stage-backdrop {
		position: absolute;
		inset: 10px;
		background: radial-gradient(circle at 50% 50%, rgba(64, 209, 128, 0.09) 0%, rgba(15, 16, 18, 0) 70%);
		pointer-events: none;
		border-radius: 20px;
	}

	.chassis-wrapper {
		position: relative;
		width: 100%;
		max-width: 420px;
		height: auto;
		display: flex;
		justify-content: center;
		user-select: none;
	}

	.mouse-svg {
		display: block;
		width: 100%;
		max-width: 360px;
		height: auto;
		filter: drop-shadow(0 14px 35px rgba(0, 0, 0, 0.75));
		pointer-events: none;
	}

	/* Numbered Hotspot Pin directly on Mouse Surface */
	.mouse-pin {
		position: absolute;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: #14171d;
		border: 2px solid #40d180;
		color: #40d180;
		padding: 0;
		cursor: pointer;
		transform: translate(-50%, -50%);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 25;
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		font-weight: 700;
		box-shadow: 0 0 10px rgba(64, 209, 128, 0.35), 0 4px 12px rgba(0, 0, 0, 0.6);
		transition: all 150ms cubic-bezier(0.16, 1, 0.3, 1);
	}

	.mouse-pin:hover,
	.mouse-pin.selected {
		transform: translate(-50%, -50%) scale(1.22);
		background: #40d180;
		color: #0b0c0e;
		border-color: #ffffff;
		box-shadow: 0 0 16px #40d180, 0 0 28px rgba(64, 209, 128, 0.5);
		z-index: 35;
	}

	.pin-num {
		line-height: 1;
		pointer-events: none;
	}

	.pin-halo {
		position: absolute;
		inset: -7px;
		border-radius: 50%;
		border: 2px solid #40d180;
		animation: haloPulse 1.8s infinite cubic-bezier(0.25, 0.8, 0.25, 1);
		pointer-events: none;
	}

	@keyframes haloPulse {
		0% { transform: scale(0.8); opacity: 0.8; }
		50% { transform: scale(1.6); opacity: 0; }
		100% { transform: scale(0.8); opacity: 0; }
	}

	/* Pin Tooltip (shows only on hover or selection) */
	.pin-tooltip {
		position: absolute;
		bottom: calc(100% + 10px);
		left: 50%;
		transform: translateX(-50%);
		background: #16181f;
		border: 1px solid #303644;
		border-radius: 8px;
		padding: 6px 12px;
		white-space: nowrap;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.75);
		pointer-events: none;
		opacity: 0;
		visibility: hidden;
		transition: opacity 140ms, visibility 140ms, transform 140ms;
		z-index: 50;
	}

	.mouse-pin:hover .pin-tooltip,
	.mouse-pin.selected .pin-tooltip {
		opacity: 1;
		visibility: visible;
		transform: translateX(-50%) translateY(-3px);
	}

	.tooltip-title {
		font-size: 11px;
		font-weight: 600;
		color: #ffffff;
		margin-bottom: 2px;
	}

	.tooltip-action {
		font-size: 10px;
		font-weight: 500;
		color: #40d180;
	}
</style>