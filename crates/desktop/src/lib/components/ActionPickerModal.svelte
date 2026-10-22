<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import type { ButtonAction, MouseButtonId } from "$lib/types/buttons";
	import { MouseButtonId as MB, buttonActionLabel } from "$lib/types/buttons";

	interface Props {
		button: MouseButtonId | null;
		currentAction: ButtonAction | null;
		isOpen: boolean;
		onSelect: (action: ButtonAction) => void;
		onClose: () => void;
	}

	let { button, currentAction, isOpen, onSelect, onClose }: Props = $props();

	let activeCategory = $state<string>("mouse");
	let clutchDpi = $state<number>(400);

	let clutchAction = $derived<ButtonAction>({
		type: "Sensitivity",
		action: { type: "SniperClutch", clutch_dpi: clutchDpi }
	});
	const profileCycleAction: ButtonAction = { type: "ProfileCycle" };
	const disabledAction: ButtonAction = { type: "Disabled" };

	$effect(() => {
		if (currentAction && currentAction.type === "Sensitivity" && currentAction.action.type === "SniperClutch") {
			clutchDpi = currentAction.action.clutch_dpi ?? 400;
		}
	});

	function isCurrent(action: ButtonAction): boolean {
		if (!currentAction) return false;
		if (action.type !== currentAction.type) return false;
		if (action.type === "Default" || action.type === "Disabled" || action.type === "ProfileCycle") return true;
		if (action.type === "Mouse" && currentAction.type === "Mouse") {
			return action.function === currentAction.function;
		}
		if (action.type === "Multimedia" && currentAction.type === "Multimedia") {
			return action.action === currentAction.action;
		}
		if (action.type === "WindowsShortcut" && currentAction.type === "WindowsShortcut") {
			return action.action === currentAction.action;
		}
		if (action.type === "Sensitivity" && currentAction.type === "Sensitivity") {
			return action.action.type === currentAction.action.type;
		}
		return false;
	}

	function handleChoose(action: ButtonAction) {
		onSelect(action);
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && button}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-backdrop" onclick={onClose} transition:fade={{ duration: 150 }}>
		<div class="modal-card" onclick={(e) => e.stopPropagation()} transition:scale={{ start: 0.94, duration: 180 }}>
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="header-title-group">
					<div class="button-badge">
						<span class="badge-dot"></span>
						{MB.displayName(button)}
					</div>
					<h3>Assign Function</h3>
					<p class="header-desc">Select the action triggered by pressing this mouse button</p>
				</div>
				<button class="close-btn" onclick={onClose} title="Close (Esc)">
					<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M18 6L6 18M6 6l12 12" />
					</svg>
				</button>
			</div>

			<!-- Modal Body with Category Tabs and Action Grid -->
			<div class="modal-body">
				<!-- Category Tabs -->
				<div class="category-tabs">
					<button class="cat-tab" class:active={activeCategory === "default"} onclick={() => (activeCategory = "default")}>
						<span class="cat-icon">⚡</span> Default
					</button>
					<button class="cat-tab" class:active={activeCategory === "mouse"} onclick={() => (activeCategory = "mouse")}>
						<span class="cat-icon">🖱️</span> Mouse
					</button>
					<button class="cat-tab" class:active={activeCategory === "sensitivity"} onclick={() => (activeCategory = "sensitivity")}>
						<span class="cat-icon">🎯</span> Sensitivity
					</button>
					<button class="cat-tab" class:active={activeCategory === "multimedia"} onclick={() => (activeCategory = "multimedia")}>
						<span class="cat-icon">🎵</span> Media
					</button>
					<button class="cat-tab" class:active={activeCategory === "shortcuts"} onclick={() => (activeCategory = "shortcuts")}>
						<span class="cat-icon">🪟</span> Windows
					</button>
					<button class="cat-tab" class:active={activeCategory === "profile"} onclick={() => (activeCategory = "profile")}>
						<span class="cat-icon">🔄</span> Profile
					</button>
					<button class="cat-tab" class:active={activeCategory === "disabled"} onclick={() => (activeCategory = "disabled")}>
						<span class="cat-icon">🚫</span> Disable
					</button>
				</div>

				<!-- Category Action Panels -->
				<div class="actions-panel">
					<!-- 1. Default Function -->
					{#if activeCategory === "default"}
						<div class="panel-section">
							<div class="action-card-hero" onclick={() => handleChoose({ type: "Default" })}>
								<div class="hero-icon">✓</div>
								<div class="hero-info">
									<div class="hero-title">Factory Default Function</div>
									<div class="hero-desc">Restores the standard hardware function: {buttonActionLabel(MB.defaultAction(button))}</div>
								</div>
								<button class="select-chip" class:active={isCurrent({ type: "Default" })}>
									{isCurrent({ type: "Default" }) ? "Active" : "Select"}
								</button>
							</div>
						</div>
					{/if}

					<!-- 2. Mouse Functions -->
					{#if activeCategory === "mouse"}
						<div class="actions-grid">
							{#each [
								{ fn: "LeftClick", name: "Left Click", desc: "Primary mouse click" },
								{ fn: "RightClick", name: "Right Click", desc: "Context menu click" },
								{ fn: "MiddleClick", name: "Scroll Click", desc: "Middle mouse click" },
								{ fn: "Back", name: "Mouse 4 (Back)", desc: "Navigate back in browser/apps" },
								{ fn: "Forward", name: "Mouse 5 (Forward)", desc: "Navigate forward in browser/apps" },
								{ fn: "ScrollLeft", name: "Scroll Tilt Left", desc: "Horizontal wheel tilt left" },
								{ fn: "ScrollRight", name: "Scroll Tilt Right", desc: "Horizontal wheel tilt right" },
								{ fn: "DoubleClick", name: "Double Click", desc: "Instantly fires rapid double click" },
							] as m}
								{@const action = { type: "Mouse" as const, function: m.fn }}
								<button class="action-card" class:active={isCurrent(action)} onclick={() => handleChoose(action)}>
									<div class="card-top">
										<span class="card-name">{m.name}</span>
										{#if isCurrent(action)}
											<span class="check-pill">Active</span>
										{/if}
									</div>
									<span class="card-desc">{m.desc}</span>
								</button>
							{/each}
						</div>
					{/if}

					<!-- 3. Sensitivity / DPI Functions -->
					{#if activeCategory === "sensitivity"}
						<div class="actions-grid">
							{#each [
								{ type: "CycleDpi", name: "Cycle DPI Stages", desc: "Step through configured DPI sensitivity stages" },
								{ type: "DpiUp", name: "DPI Up", desc: "Switch to next higher sensitivity stage" },
								{ type: "DpiDown", name: "DPI Down", desc: "Switch to next lower sensitivity stage" },
							] as s}
								{@const action = { type: "Sensitivity" as const, action: { type: s.type as any } }}
								<button class="action-card" class:active={isCurrent(action)} onclick={() => handleChoose(action)}>
									<div class="card-top">
										<span class="card-name">{s.name}</span>
										{#if isCurrent(action)}
											<span class="check-pill">Active</span>
										{/if}
									</div>
									<span class="card-desc">{s.desc}</span>
								</button>
							{/each}

							<!-- Sniper Clutch with DPI selector -->
							<div class="clutch-card" class:active={isCurrent(clutchAction)}>
								<div class="card-top">
									<span class="card-name">Sensitivity Clutch / Sniper</span>
									{#if isCurrent(clutchAction)}
										<span class="check-pill">Active</span>
									{/if}
								</div>
								<span class="card-desc">Temporarily drops sensitivity while holding button</span>

								<div class="clutch-slider-row">
									<span class="clutch-value">{clutchDpi} DPI</span>
									<input
										type="range"
										min="100"
										max="3200"
										step="50"
										bind:value={clutchDpi}
										class="clutch-range"
									/>
									<button class="btn btn-primary btn-sm" onclick={() => handleChoose(clutchAction)}>
										Set Clutch
									</button>
								</div>
							</div>
						</div>
					{/if}

					<!-- 4. Multimedia Functions -->
					{#if activeCategory === "multimedia"}
						<div class="actions-grid">
							{#each [
								{ act: "PlayPause", name: "Play / Pause", desc: "Toggle media playback" },
								{ act: "VolumeUp", name: "Volume Up", desc: "Increase system master volume" },
								{ act: "VolumeDown", name: "Volume Down", desc: "Decrease system master volume" },
								{ act: "Mute", name: "Mute Audio", desc: "Mute / unmute audio output" },
								{ act: "NextTrack", name: "Next Track", desc: "Skip to next media track" },
								{ act: "PrevTrack", name: "Previous Track", desc: "Return to previous track" },
							] as media}
								{@const action = { type: "Multimedia" as const, action: media.act }}
								<button class="action-card" class:active={isCurrent(action)} onclick={() => handleChoose(action)}>
									<div class="card-top">
										<span class="card-name">{media.name}</span>
										{#if isCurrent(action)}
											<span class="check-pill">Active</span>
										{/if}
									</div>
									<span class="card-desc">{media.desc}</span>
								</button>
							{/each}
						</div>
					{/if}

					<!-- 5. Windows Shortcuts -->
					{#if activeCategory === "shortcuts"}
						<div class="actions-grid">
							{#each [
								{ act: "LockPc", name: "Lock PC", desc: "Instant workstation lock (Win+L)" },
								{ act: "TaskView", name: "Task View", desc: "Overview all open windows (Win+Tab)" },
								{ act: "ShowDesktop", name: "Show Desktop", desc: "Minimize all windows (Win+D)" },
								{ act: "OpenExplorer", name: "File Explorer", desc: "Launch Windows File Explorer (Win+E)" },
							] as win}
								{@const action = { type: "WindowsShortcut" as const, action: win.act }}
								<button class="action-card" class:active={isCurrent(action)} onclick={() => handleChoose(action)}>
									<div class="card-top">
										<span class="card-name">{win.name}</span>
										{#if isCurrent(action)}
											<span class="check-pill">Active</span>
										{/if}
									</div>
									<span class="card-desc">{win.desc}</span>
								</button>
							{/each}
						</div>
					{/if}

					<!-- 6. Profile Cycle -->
					{#if activeCategory === "profile"}
						<div class="panel-section">
							<div class="action-card-hero" onclick={() => handleChoose(profileCycleAction)}>
								<div class="hero-icon">🔄</div>
								<div class="hero-info">
									<div class="hero-title">Cycle Next Profile</div>
									<div class="hero-desc">Switches immediately to your next saved hardware configuration profile</div>
								</div>
								<button class="select-chip" class:active={isCurrent(profileCycleAction)}>
									{isCurrent(profileCycleAction) ? "Active" : "Select"}
								</button>
							</div>
						</div>
					{/if}

					<!-- 7. Disabled Button -->
					{#if activeCategory === "disabled"}
						<div class="panel-section">
							<div class="action-card-hero danger" onclick={() => handleChoose(disabledAction)}>
								<div class="hero-icon danger">✕</div>
								<div class="hero-info">
									<div class="hero-title">Disable This Button</div>
									<div class="hero-desc">Completely suppresses and mutes inputs from this button</div>
								</div>
								<button class="select-chip danger" class:active={isCurrent(disabledAction)}>
									{isCurrent(disabledAction) ? "Disabled" : "Disable Button"}
								</button>
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(8, 9, 11, 0.75);
		backdrop-filter: blur(8px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: 24px;
	}

	.modal-card {
		background: #14161a;
		border: 1px solid #2e323b;
		border-radius: 16px;
		width: 100%;
		max-width: 680px;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(64, 209, 128, 0.15);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		padding: 20px 24px 16px;
		border-bottom: 1px solid #252830;
		background: #181b20;
	}

	.button-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 10px;
		background: rgba(64, 209, 128, 0.12);
		border: 1px solid rgba(64, 209, 128, 0.35);
		border-radius: 20px;
		color: #40d180;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.3px;
		margin-bottom: 6px;
	}

	.badge-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #40d180;
		box-shadow: 0 0 8px #40d180;
	}

	.header-title-group h3 {
		margin: 0;
		font-size: 18px;
		font-weight: 600;
		color: #ffffff;
		letter-spacing: -0.01em;
	}

	.header-desc {
		margin: 4px 0 0;
		font-size: 12px;
		color: #8d9098;
	}

	.close-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		background: transparent;
		border: 1px solid transparent;
		color: #8d9098;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 150ms;
	}

	.close-btn:hover {
		background: rgba(255, 255, 255, 0.08);
		color: #fff;
		border-color: #383c44;
	}

	.close-btn svg {
		width: 18px;
		height: 18px;
	}

	.modal-body {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		flex: 1;
	}

	/* Category Horizontal Tabs */
	.category-tabs {
		display: flex;
		gap: 6px;
		padding: 12px 20px;
		background: #111316;
		border-bottom: 1px solid #23262d;
		overflow-x: auto;
	}

	.cat-tab {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: 8px;
		color: #8d9098;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: all 150ms;
	}

	.cat-tab:hover {
		color: #f1f3f5;
		background: rgba(255, 255, 255, 0.05);
	}

	.cat-tab.active {
		background: rgba(64, 209, 128, 0.12);
		border-color: rgba(64, 209, 128, 0.4);
		color: #40d180;
		font-weight: 600;
	}

	.actions-panel {
		padding: 20px 24px;
		overflow-y: auto;
		max-height: 480px;
	}

	.actions-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
	}

	.action-card {
		background: #1a1d22;
		border: 1px solid #282c34;
		border-radius: 10px;
		padding: 14px 16px;
		text-align: left;
		cursor: pointer;
		transition: all 160ms cubic-bezier(0.16, 1, 0.3, 1);
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.action-card:hover {
		border-color: #40d180;
		background: #20242b;
		transform: translateY(-2px);
		box-shadow: 0 6px 16px rgba(0, 0, 0, 0.3);
	}

	.action-card.active {
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.08);
		box-shadow: 0 0 0 1px #40d180;
	}

	.card-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.card-name {
		font-size: 13px;
		font-weight: 600;
		color: #f1f3f5;
	}

	.check-pill {
		font-size: 10px;
		font-weight: 700;
		padding: 2px 7px;
		border-radius: 10px;
		background: #40d180;
		color: #0b0c0e;
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.card-desc {
		font-size: 11px;
		color: #8d9098;
		line-height: 1.4;
	}

	/* Hero Card for Default & Single actions */
	.action-card-hero {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 18px 20px;
		background: #1a1d22;
		border: 1px solid #282c34;
		border-radius: 12px;
		cursor: pointer;
		transition: all 160ms;
	}

	.action-card-hero:hover {
		border-color: #40d180;
		background: #20242b;
	}

	.action-card-hero.danger:hover {
		border-color: #e05e5e;
		background: rgba(224, 94, 94, 0.08);
	}

	.hero-icon {
		width: 44px;
		height: 44px;
		border-radius: 10px;
		background: rgba(64, 209, 128, 0.12);
		border: 1px solid rgba(64, 209, 128, 0.3);
		color: #40d180;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		flex-shrink: 0;
	}

	.hero-icon.danger {
		background: rgba(224, 94, 94, 0.12);
		border-color: rgba(224, 94, 94, 0.3);
		color: #e05e5e;
	}

	.hero-info {
		flex: 1;
	}

	.hero-title {
		font-size: 14px;
		font-weight: 600;
		color: #ffffff;
		margin-bottom: 3px;
	}

	.hero-desc {
		font-size: 12px;
		color: #8d9098;
	}

	.select-chip {
		padding: 6px 14px;
		border-radius: 8px;
		font-size: 12px;
		font-weight: 600;
		background: rgba(255, 255, 255, 0.06);
		border: 1px solid #383c44;
		color: #f1f3f5;
		cursor: pointer;
	}

	.select-chip.active {
		background: #40d180;
		border-color: #40d180;
		color: #000;
	}

	.select-chip.danger {
		color: #e05e5e;
		border-color: rgba(224, 94, 94, 0.4);
	}

	/* Clutch Card with Slider */
	.clutch-card {
		grid-column: span 2;
		background: #1a1d22;
		border: 1px solid #282c34;
		border-radius: 10px;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.clutch-card.active {
		border-color: #40d180;
		background: rgba(64, 209, 128, 0.06);
	}

	.clutch-slider-row {
		display: flex;
		align-items: center;
		gap: 14px;
		margin-top: 6px;
	}

	.clutch-value {
		font-size: 14px;
		font-weight: 700;
		color: #40d180;
		font-family: var(--font-mono, monospace);
		min-width: 75px;
	}

	.clutch-range {
		flex: 1;
		accent-color: #40d180;
		cursor: pointer;
	}
</style>
