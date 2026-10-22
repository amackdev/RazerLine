<script lang="ts" generics="T extends MouseButtonId">
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import { gsap } from "gsap";
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

	// Hotspot positions for each device/chassis (normalized 0-1 coordinates)
	// These match the SVG viewBox coordinates from the original mouse_3d.rs
	const hotspots: Record<string, Record<string, { x: number; y: number; label: string; badgeX: number; badgeY: number }>> = {
		// Basilisk V3 Top View
		"basilisk_v3-top": {
			MiddleClick: { x: 0.5, y: 0.28, label: "Scroll Click", badgeX: 0.05, badgeY: 0.15 },
			TiltLeft: { x: 0.47, y: 0.28, label: "Scroll Left", badgeX: 0.05, badgeY: 0.22 },
			MultiFunctionClutch: { x: 0.34, y: 0.42, label: "Sensitivity Clutch", badgeX: 0.05, badgeY: 0.28 },
			SideForward: { x: 0.39, y: 0.36, label: "Mouse Button 5", badgeX: 0.05, badgeY: 0.34 },
			SideBack: { x: 0.39, y: 0.45, label: "Mouse Button 4", badgeX: 0.05, badgeY: 0.4 },
			ProfileSwitch: { x: 0.39, y: 0.58, label: "Cycle Profile", badgeX: 0.05, badgeY: 0.46 },
			ScrollUp: { x: 0.5, y: 0.24, label: "Scroll Up", badgeX: 0.95, badgeY: 0.15 },
			TiltRight: { x: 0.53, y: 0.28, label: "Scroll Right", badgeX: 0.95, badgeY: 0.22 },
			ScrollDown: { x: 0.5, y: 0.32, label: "Scroll Down", badgeX: 0.95, badgeY: 0.28 },
			ScrollMode: { x: 0.5, y: 0.48, label: "Scroll Mode", badgeX: 0.95, badgeY: 0.34 },
			DpiSwitch: { x: 0.5, y: 0.44, label: "Cycle Sensitivity", badgeX: 0.95, badgeY: 0.4 },
		},
		// Basilisk V3 Bottom View
		"basilisk_v3-bottom": {
			ProfileSwitch: { x: 0.42, y: 0.58, label: "Cycle Profile", badgeX: 0.05, badgeY: 0.46 },
		},
		// Viper Ultimate Top View
		"viper_ultimate-top": {
			MiddleClick: { x: 0.5, y: 0.28, label: "Scroll Click", badgeX: 0.05, badgeY: 0.15 },
			SideForward: { x: 0.32, y: 0.36, label: "Mouse Button 5", badgeX: 0.05, badgeY: 0.22 },
			SideBack: { x: 0.32, y: 0.43, label: "Mouse Button 4", badgeX: 0.05, badgeY: 0.28 },
			DpiSwitch: { x: 0.5, y: 0.44, label: "DPI Cycle", badgeX: 0.95, badgeY: 0.34 },
			ScrollUp: { x: 0.5, y: 0.24, label: "Scroll Up", badgeX: 0.95, badgeY: 0.15 },
			ScrollDown: { x: 0.5, y: 0.32, label: "Scroll Down", badgeX: 0.95, badgeY: 0.28 },
		},
		// Viper Ultimate Bottom View
		"viper_ultimate-bottom": {
			ProfileSwitch: { x: 0.42, y: 0.56, label: "Cycle Profile", badgeX: 0.05, badgeY: 0.46 },
		},
		// Generic Top View
		"generic-top": {
			MiddleClick: { x: 0.5, y: 0.28, label: "Scroll Click", badgeX: 0.05, badgeY: 0.15 },
			SideForward: { x: 0.35, y: 0.38, label: "Button 5", badgeX: 0.05, badgeY: 0.22 },
			SideBack: { x: 0.35, y: 0.45, label: "Button 4", badgeX: 0.05, badgeY: 0.28 },
			DpiSwitch: { x: 0.5, y: 0.44, label: "DPI Cycle", badgeX: 0.95, badgeY: 0.34 },
		},
		// Generic Bottom View (empty)
		"generic-bottom": {},
	};

	let svgElement: HTMLDivElement | null = $state(null);
	let containerRect: DOMRect | null = $state(null);

	function updateContainerRect() {
		if (svgElement) {
			containerRect = svgElement.getBoundingClientRect();
		}
	}

	onMount(() => {
		updateContainerRect();
		window.addEventListener("resize", updateContainerRect);
		return () => window.removeEventListener("resize", updateContainerRect);
	});

	// Reactive key that updates when src or viewMode changes
	let key = $derived(`${src.replace(/\.(svg|png)$/, "")}-${viewMode}`);
	let deviceHotspots = $derived(hotspots[key] ?? {});

	function getAction(button: MouseButtonId): ButtonAction {
		return mappings[button] ?? MB.defaultAction(button);
	}

	function handlePinClick(button: MouseButtonId) {
		onSelect(button);
	}

	function getBadgePosition(hotspot: { x: number; y: number; badgeX: number; badgeY: number }) {
		if (!containerRect) return { x: 0, y: 0 };
		return {
			x: containerRect.left + containerRect.width * hotspot.badgeX,
			y: containerRect.top + containerRect.height * hotspot.badgeY,
		};
	}

	function getPinPosition(hotspot: { x: number; y: number }) {
		if (!containerRect) return { x: 0, y: 0 };
		return {
			x: containerRect.left + containerRect.width * hotspot.x,
			y: containerRect.top + containerRect.height * hotspot.y,
		};
	}
</script>

<div class="mouse-diagram" bind:this={svgElement}>
	<!-- SVG Image -->
	<img 
		src={src} 
		alt="Mouse diagram" 
		class="mouse-svg"
		onload={updateContainerRect}
	/>

	<!-- Hotspot Pins -->
	{#each Object.entries(deviceHotspots) as [buttonId, hotspot]}
		{#if buttonId !== "ProfileSwitch" || viewMode === "bottom"}
			{#key `${buttonId}-${selectedButton}`}
				<div
					class="pin-container"
					style="
						left: calc({containerRect?.width ?? 0}px * {hotspot.x} + {containerRect?.left ?? 0}px);
						top: calc({containerRect?.height ?? 0}px * {hotspot.y} + {containerRect?.top ?? 0}px);
					"
				>
					<!-- Pin on mouse body -->
					<button
						class="pin {selectedButton === buttonId ? 'selected' : ''} {MB.isConfigurable(buttonId as MouseButtonId) ? '' : 'non-configurable'}"
						onclick={() => handlePinClick(buttonId as MouseButtonId)}
						style="--pin-color: {selectedButton === buttonId ? '#40D180' : '#3C404C'};"
					>
						<!-- Pulse ring for selected -->
						{#if selectedButton === buttonId}
							<div class="pin-pulse"></div>
						{/if}
					</button>

					<!-- Badge/Callout -->
					<div
						class="pin-badge {selectedButton === buttonId ? 'active' : ''}"
						style="
							left: {getBadgePosition(hotspot).x}px;
							top: {getBadgePosition(hotspot).y}px;
						"
					>
						<div class="badge-content">
							<div class="badge-label">{hotspot.label}</div>
							<div class="badge-action">{buttonActionLabel(getAction(buttonId as MouseButtonId))}</div>
						</div>
						<!-- Leader line connecting badge to pin -->
						<div
							class="leader-line"
							style="
								--pin-x: {getPinPosition(hotspot).x}px;
								--pin-y: {getPinPosition(hotspot).y}px;
								--badge-x: {getBadgePosition(hotspot).x}px;
								--badge-y: {getBadgePosition(hotspot).y}px;
							"
						>
							<svg class="leader-svg" viewBox="0 0 200 100" preserveAspectRatio="none">
								<path
									d="M 0 50 Q 50 50 50 50 Q 50 50 100 50"
									stroke="{selectedButton === buttonId ? '#40D180' : '#3C404C'}"
									stroke-width="1.5"
									fill="none"
									vector-effect="non-scaling-stroke"
								/>
							</svg>
						</div>
					</div>
				</div>
			{/key}
		{/if}
	{/each}
</div>

<style>
	.mouse-diagram {
		position: relative;
		width: 100%;
		max-width: 560px;
		margin: 0 auto;
	}

	.mouse-svg {
		display: block;
		width: 100%;
		height: auto;
		pointer-events: none;
	}

	.pin-container {
		position: absolute;
		pointer-events: none;
		z-index: 10;
	}

	.pin {
		position: absolute;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--pin-color);
		border: 2px solid var(--surface);
		box-shadow: 0 0 0 2px var(--pin-color), 0 4px 12px rgba(0, 0, 0, 0.4);
		cursor: pointer;
		transform: translate(-50%, -50%);
		pointer-events: auto;
		transition: transform 150ms, box-shadow 150ms;
		z-index: 20;
	}

	.pin:hover {
		transform: translate(-50%, -50%) scale(1.2);
		box-shadow: 0 0 0 3px var(--pin-color), 0 6px 20px rgba(0, 0, 0, 0.5);
	}

	.pin.selected {
		animation: pinSelected 150ms ease-out;
	}

	.pin.non-configurable {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.pin.non-configurable:hover {
		transform: translate(-50%, -50%);
	}

	@keyframes pinSelected {
		0% { transform: translate(-50%, -50%) scale(0.8); }
		50% { transform: translate(-50%, -50%) scale(1.3); }
		100% { transform: translate(-50%, -50%) scale(1); }
	}

	.pin-pulse {
		position: absolute;
		inset: -4px;
		border-radius: 50%;
		border: 2px solid #40D180;
		opacity: 0;
		animation: pulse 2s ease-out infinite;
	}

	@keyframes pulse {
		0% { transform: scale(1); opacity: 0.6; }
		100% { transform: scale(1.5); opacity: 0; }
	}

	.pin-badge {
		position: absolute;
		pointer-events: none;
		transform: translateX(-50%);
		z-index: 15;
		opacity: 0;
		transform: translateX(-50%) translateY(10px);
		transition: opacity 200ms, transform 200ms;
	}

	.pin-badge.active {
		opacity: 1;
		transform: translateX(-50%) translateY(0);
	}

	.badge-content {
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px 12px;
		min-width: 160px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
	}

	.badge-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--fg);
		margin-bottom: 4px;
		white-space: nowrap;
	}

	.badge-action {
		font-size: 11px;
		color: var(--muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 220px;
	}

	.leader-line {
		position: absolute;
		left: 0;
		top: 0;
		width: 200px;
		height: 100px;
		pointer-events: none;
		transform-origin: 0 0;
	}

	.leader-svg {
		width: 100%;
		height: 100%;
	}
</style>