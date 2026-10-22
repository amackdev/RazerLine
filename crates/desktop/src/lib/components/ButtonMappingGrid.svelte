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
			<h3>BUTTON MAPPINGS</h3>
			<span class="header-subtitle">Click any button to configure its function</span>
		</div>
		{#if onReset}
			<button class="btn btn-ghost btn-sm" onclick={onReset}>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
					<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
					<path d="M3 3v5h5" />
				</svg>
				Reset All
			</button>
		{/if}
	</div>

	<div class="grid-body">
		{#each configurableButtons as button}
			{@const isSelected = selectedButton === button}
			{@const action = getAction(button)}

			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="mapping-row"
				class:selected={isSelected}
				onclick={() => onSelect(button)}
			>
				<div class="button-identity">
					<div class="button-icon-box" class:selected={isSelected}>
						{#if button === "MiddleClick"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3" /><rect x="9" y="5" width="6" height="14" rx="3" /></svg>
						{:else if button === "TiltLeft"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18H9M15 12H9M15 6H9" /></svg>
						{:else if button === "TiltRight"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18H15M9 12H15M9 6H15" /></svg>
						{:else if button === "ScrollUp"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 19V5M5 12l7-7 7 7" /></svg>
						{:else if button === "ScrollDown"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12l7 7 7-7" /></svg>
						{:else if button === "SideBack"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>
						{:else if button === "SideForward"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7" /></svg>
						{:else if button === "MultiFunctionClutch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" /></svg>
						{:else if button === "DpiSwitch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 3" /></svg>
						{:else if button === "ScrollMode"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3" /><circle cx="12" cy="12" r="8" stroke-dasharray="3 3" /></svg>
						{:else if button === "ProfileSwitch"}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" /></svg>
						{:else}
							<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="4" /></svg>
						{/if}
					</div>

					<div class="button-text-stack">
						<span class="button-display-name">{MB.displayName(button)}</span>
						<span class="button-tag">{MB.shortName(button)}</span>
					</div>
				</div>

				<div class="action-pill-box">
					<div class="action-pill" class:selected={isSelected}>
						<span class="pill-dot"></span>
						<span class="pill-label">{buttonActionLabel(action)}</span>
						<svg class="pill-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M9 18l6-6-6-6" />
						</svg>
					</div>
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
	}

	.grid-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 14px;
		padding-bottom: 10px;
		border-bottom: 1px solid #23262d;
	}

	.header-left h3 {
		margin: 0;
		font-size: 13px;
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
		max-height: calc(100vh - 280px);
		padding-right: 4px;
	}

	.mapping-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 14px;
		background: #16181d;
		border: 1px solid #242831;
		border-radius: 10px;
		cursor: pointer;
		transition: all 140ms cubic-bezier(0.16, 1, 0.3, 1);
		user-select: none;
	}

	.mapping-row:hover {
		border-color: #40d180;
		background: #1c1f26;
		transform: translateX(2px);
	}

	.mapping-row.selected {
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.08);
		box-shadow: 0 0 0 1px #40d180;
	}

	.button-identity {
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
		flex: 1;
	}

	.button-icon-box {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		background: #1b1e24;
		border: 1px solid #2b2f38;
		color: #8d9098;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		transition: all 140ms;
	}

	.button-icon-box svg {
		width: 16px;
		height: 16px;
	}

	.button-icon-box.selected,
	.mapping-row:hover .button-icon-box {
		color: #40d180;
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.12);
	}

	.button-text-stack {
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

	.action-pill-box {
		flex-shrink: 0;
	}

	.action-pill {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		background: #1c1f26;
		border: 1px solid #2b2f3a;
		border-radius: 8px;
		transition: all 140ms;
		max-width: 200px;
	}

	.mapping-row:hover .action-pill {
		border-color: #40d180;
		background: #232832;
	}

	.action-pill.selected {
		background: rgba(64, 209, 128, 0.15);
		border-color: #40d180;
	}

	.pill-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #40d180;
		box-shadow: 0 0 8px rgba(64, 209, 128, 0.6);
		flex-shrink: 0;
	}

	.pill-label {
		font-size: 12px;
		font-weight: 500;
		color: #e5e8ec;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.pill-chevron {
		width: 14px;
		height: 14px;
		color: #8d9098;
		flex-shrink: 0;
		transition: transform 140ms;
	}

	.mapping-row:hover .pill-chevron {
		color: #40d180;
		transform: translateX(2px);
	}
</style>