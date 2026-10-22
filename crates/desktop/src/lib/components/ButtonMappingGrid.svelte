<script lang="ts">
	import type { ButtonAction, MouseButtonId } from "$lib/types/buttons";
	import { MouseButtonId as MB } from "$lib/types/buttons";
	import { buttonActionLabel } from "$lib/types/buttons";

	interface Props {
		mappings: Record<string, ButtonAction>;
		selectedButton: MouseButtonId | null;
		onSelect: (button: MouseButtonId) => void;
		onReset?: () => void;
	}

	let { mappings, selectedButton, onSelect, onReset }: Props = $props();

	const configurableButtons = MB.CONFIGURABLE;

	function getAction(button: MouseButtonId): ButtonAction {
		return mappings[button] ?? MB.defaultAction(button);
	}
</script>

<div class="mapping-container">
	<div class="grid-header">
		<div class="header-left">
			<span class="header-title">BUTTON ASSIGNMENTS</span>
			<span class="header-subtitle">Select any button to remap function</span>
		</div>
		{#if onReset}
			<button class="btn btn-ghost btn-sm" onclick={onReset} title="Reset all buttons to factory defaults">
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
					<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
					<path d="M3 3v5h5" />
				</svg>
				Reset All
			</button>
		{/if}
	</div>

	<div class="grid-body">
		{#each configurableButtons as button, idx}
			{@const isSelected = selectedButton === button}
			{@const action = getAction(button)}

			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="mapping-row"
				class:selected={isSelected}
				onclick={() => onSelect(button)}
			>
				<!-- Index number matching diagram pin -->
				<div class="button-num" class:selected={isSelected}>
					{idx + 1}
				</div>

				<!-- Button Name -->
				<div class="button-text">
					<span class="button-display-name">{MB.displayName(button)}</span>
					<span class="button-tag">{MB.shortName(button)}</span>
				</div>

				<!-- Current Function Action Pill -->
				<div class="action-pill" class:selected={isSelected}>
					<span class="pill-dot"></span>
					<span class="pill-label">{buttonActionLabel(action)}</span>
					<svg class="pill-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M9 18l6-6-6-6" />
					</svg>
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	.mapping-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-width: 0;
	}

	.grid-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
		padding-bottom: 10px;
		border-bottom: 1px solid #23262e;
	}

	.header-left {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.header-title {
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.5px;
		color: #ffffff;
		text-transform: uppercase;
	}

	.header-subtitle {
		font-size: 11px;
		color: #8d9098;
	}

	.grid-body {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow-y: auto;
		max-height: calc(100vh - 290px);
		padding-right: 4px;
	}

	/* CSS Grid row: Fixed 26px badge, Flexible title, Auto right-aligned action */
	.mapping-row {
		display: grid;
		grid-template-columns: 26px minmax(0, 1fr) auto;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		background: #14171c;
		border: 1px solid #232731;
		border-radius: 8px;
		cursor: pointer;
		transition: all 140ms ease;
		user-select: none;
	}

	.mapping-row:hover {
		border-color: #40d180;
		background: #1a1e25;
		transform: translateX(2px);
	}

	.mapping-row.selected {
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.09);
		box-shadow: 0 0 0 1px #40d180;
	}

	.button-num {
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: #1c2027;
		border: 1px solid #2d323e;
		color: #8d9098;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 11px;
		font-weight: 700;
		font-family: var(--font-mono, monospace);
		flex-shrink: 0;
		transition: all 140ms ease;
	}

	.mapping-row:hover .button-num,
	.mapping-row.selected .button-num {
		background: #40d180;
		color: #0b0c0e;
		border-color: #40d180;
		box-shadow: 0 0 8px rgba(64, 209, 128, 0.5);
	}

	.button-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.button-display-name {
		font-size: 13px;
		font-weight: 600;
		color: #f1f3f5;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.button-tag {
		font-size: 10px;
		color: #8d9098;
		font-family: var(--font-mono, monospace);
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.action-pill {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		padding: 5px 10px;
		background: #1a1e24;
		border: 1px solid #282d38;
		border-radius: 6px;
		max-width: 170px;
		flex-shrink: 0;
		transition: all 140ms ease;
	}

	.mapping-row:hover .action-pill {
		border-color: #40d180;
		background: #20252e;
	}

	.action-pill.selected {
		background: rgba(64, 209, 128, 0.16);
		border-color: #40d180;
	}

	.pill-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #40d180;
		box-shadow: 0 0 6px #40d180;
		flex-shrink: 0;
	}

	.pill-label {
		font-size: 11px;
		font-weight: 500;
		color: #e5e8ec;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.pill-chevron {
		width: 12px;
		height: 12px;
		color: #8d9098;
		flex-shrink: 0;
		transition: transform 140ms ease;
	}

	.mapping-row:hover .pill-chevron {
		color: #40d180;
		transform: translateX(2px);
	}
</style>