<script lang="ts">
	import { fly } from "svelte/transition";
	import { flip } from "svelte/animate";
	import { gsap } from "gsap";
	import type { ButtonAction, MouseButtonId } from "$lib/types/buttons";
	import { MouseButtonId as MB } from "$lib/types/buttons";
	import { buttonActionLabel } from "$lib/types/buttons";

	interface Props {
		mappings: Record<string, ButtonAction>;
		selectedButton: MouseButtonId | null;
		onSelect: (button: MouseButtonId) => void;
		onUpdate: (button: MouseButtonId, action: ButtonAction) => void;
		onReset?: () => void;
	}

	let { mappings, selectedButton, onSelect, onUpdate, onReset }: Props = $props();

	const configurableButtons = MB.CONFIGURABLE;

	function getAction(button: MouseButtonId): ButtonAction {
		return mappings[button] ?? MB.defaultAction(button);
	}

	function handleActionChange(button: MouseButtonId, newAction: ButtonAction) {
		onUpdate(button, newAction);
	}

	// Action categories for the selector dropdown
	const actionCategories = [
		{ label: "Default", actions: [{ type: "Default" as const }] },
		{
			label: "Mouse Functions",
			actions: [
				{ type: "Mouse" as const, function: "LeftClick" },
				{ type: "Mouse" as const, function: "RightClick" },
				{ type: "Mouse" as const, function: "MiddleClick" },
				{ type: "Mouse" as const, function: "ScrollLeft" },
				{ type: "Mouse" as const, function: "ScrollRight" },
				{ type: "Mouse" as const, function: "Back" },
				{ type: "Mouse" as const, function: "Forward" },
				{ type: "Mouse" as const, function: "DoubleClick" },
			],
		},
		{
			label: "Sensitivity",
			actions: [
				{ type: "Sensitivity" as const, action: { type: "CycleDpi" as const } },
				{ type: "Sensitivity" as const, action: { type: "DpiUp" as const } },
				{ type: "Sensitivity" as const, action: { type: "DpiDown" as const } },
				{ type: "Sensitivity" as const, action: { type: "SniperClutch", clutch_dpi: 400 } },
			],
		},
		{
			label: "Multimedia",
			actions: [
				{ type: "Multimedia" as const, action: "PlayPause" },
				{ type: "Multimedia" as const, action: "VolumeUp" },
				{ type: "Multimedia" as const, action: "VolumeDown" },
				{ type: "Multimedia" as const, action: "Mute" },
				{ type: "Multimedia" as const, action: "NextTrack" },
				{ type: "Multimedia" as const, action: "PrevTrack" },
			],
		},
		{
			label: "Windows Shortcuts",
			actions: [
				{ type: "WindowsShortcut" as const, action: "LockPc" },
				{ type: "WindowsShortcut" as const, action: "TaskView" },
				{ type: "WindowsShortcut" as const, action: "ShowDesktop" },
				{ type: "WindowsShortcut" as const, action: "OpenExplorer" },
			],
		},
		{ label: "Profile", actions: [{ type: "ProfileCycle" as const }] },
		{ label: "Disabled", actions: [{ type: "Disabled" as const }] },
	] as const;
</script>

<div class="mapping-grid">
	<div class="grid-header">
		<h3>BUTTON MAPPINGS</h3>
		{#if onReset}
			<button class="btn btn-ghost btn-sm" onclick={onReset}>Reset All</button>
		{/if}
	</div>

	<div class="grid-body">
		{#each configurableButtons as button}
			<div class="mapping-row {selectedButton === button ? 'selected' : ''}" onclick={() => onSelect(button)}>
				<div class="button-info">
					<div class="button-icon" style="--btn-color: {selectedButton === button ? '#40D180' : '#3C404C'}">
						{#if button === "MiddleClick"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/></svg>
						{:else if button === "TiltLeft"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18H9M15 12H9M15 6H9"/></svg>
						{:else if button === "TiltRight"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18H15M9 12H15M9 6H15"/></svg>
						{:else if button === "ScrollUp"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
						{:else if button === "ScrollDown"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12l7 7 7-7"/></svg>
						{:else if button === "SideBack"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
						{:else if button === "SideForward"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
						{:else if button === "MultiFunctionClutch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
						{:else if button === "DpiSwitch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/></svg>
						{:else if button === "ScrollMode"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v18M3 12h18"/></svg>
						{:else if button === "ProfileSwitch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
						{/if}
					</div>
					<div class="button-name">{MB.displayName(button)}</div>
					<div class="button-short">{MB.shortName(button)}</div>
				</div>

				<div class="action-selector">
					<div class="current-action" onclick={(e) => { e.stopPropagation(); onSelect(button); }}>
						<span class="action-label">{buttonActionLabel(getAction(button))}</span>
						<svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
					</div>

					{#if selectedButton === button}
						<div class="action-dropdown">
							{#each actionCategories as category}
								<div class="dropdown-section">
									<div class="dropdown-section-label">{category.label}</div>
									<div class="dropdown-actions">
										{#each category.actions as action}
											<button
												class="dropdown-action {JSON.stringify(getAction(button)) === JSON.stringify(action) ? 'active' : ''}"
												onclick={(e) => { e.stopPropagation(); handleActionChange(button, action); }}
											>
												<span>{buttonActionLabel(action)}</span>
											</button>
										{/each}
									</div>
								</div>
							{/each}
							<button class="dropdown-action danger" onclick={(e) => { e.stopPropagation(); handleActionChange(button, { type: "Disabled" }); }}>
								Disabled
							</button>
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	.grid-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
	}

	.grid-header h3 {
		margin: 0;
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.grid-body {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.mapping-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 12px 14px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		transition: border-color 150ms, background 150ms;
		cursor: pointer;
	}

	.mapping-row:hover {
		border-color: var(--accent);
	}

	.mapping-row.selected {
		border-color: var(--accent);
		background: var(--surface-elevated);
	}

	.button-info {
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
		flex: 1;
	}

	.button-icon {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		color: var(--btn-color);
		flex-shrink: 0;
		transition: border-color 150ms, color 150ms;
	}

	.button-icon svg {
		width: 18px;
		height: 18px;
	}

	.button-name {
		font-size: 14px;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.button-short {
		font-size: 11px;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.action-selector {
		position: relative;
		flex-shrink: 0;
	}

	.current-action {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: 8px;
		font-size: 13px;
		color: var(--fg);
		cursor: pointer;
		transition: border-color 150ms;
		min-width: 200px;
	}

	.current-action:hover {
		border-color: var(--accent);
	}

	.action-label {
		flex: 1;
		text-align: right;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.chevron {
		width: 16px;
		height: 16px;
		color: var(--muted);
		flex-shrink: 0;
		transition: transform 150ms;
	}

	.current-action:focus-within .chevron,
	.current-action:hover .chevron {
		transform: rotate(180deg);
	}

	.action-dropdown {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		width: 280px;
		max-height: 400px;
		overflow-y: auto;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 8px;
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
		z-index: 100;
	}

	.dropdown-section {
		border-bottom: 1px solid var(--border);
	}

	.dropdown-section:last-child {
		border-bottom: none;
	}

	.dropdown-section-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
		padding: 8px 12px 4px;
	}

	.dropdown-actions {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 0 8px 8px;
	}

	.dropdown-action {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 10px 12px;
		background: transparent;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		color: var(--fg);
		text-align: left;
		cursor: pointer;
		transition: background 150ms, color 150ms;
	}

	.dropdown-action:hover {
		background: var(--surface);
	}

	.dropdown-action.active {
		background: var(--accent);
		color: var(--bg);
	}

	.dropdown-action.danger {
		color: var(--danger);
		margin-top: 8px;
		padding-top: 12px;
		border-top: 1px solid var(--border);
	}

	.dropdown-action.danger:hover {
		background: var(--danger);
		color: white;
	}
</style>