<script lang="ts">
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import { profile as profileState } from "$lib/state/profiles.svelte";
	import { device } from "$lib/state/device.svelte";

	let configPath = $state("");

	interface Props {
		delay?: number;
	}

	let { delay = 0 }: Props = $props();

	async function toggleStartWithWindows(e: Event) {
		const target = e.target as HTMLInputElement;
		await profileState.setStartWithWindows(target.checked);
		await profileState.saveConfig();
	}

	async function toggleCloseToTray(e: Event) {
		const target = e.target as HTMLInputElement;
		await profileState.setCloseToTray(target.checked);
		await profileState.saveConfig();
	}

	async function saveConfig() {
		await profileState.saveConfig();
	}

	async function resetToDefaults() {
		if (!confirm("Reset all settings to defaults?")) return;
		await profileState.saveConfig();
	}

	function getConfigPath() {
		if (configPath) return configPath;
		return "~/.config/razerline/config.json (Linux) / %APPDATA%\\RazerLine\\config.json (Windows)";
	}

	onMount(async () => {
		await profileState.loadConfig();
		configPath = getConfigPath();
	});
</script>

<div class="settings-card" in:fly={{ y: 8, duration: 280, delay }}>
	<div class="card-header">
		<h3>SETTINGS</h3>
		<div class="header-actions">
			<button class="btn btn-ghost btn-sm" onclick={resetToDefaults}>Reset All</button>
			<button class="btn btn-primary btn-sm" onclick={saveConfig}>Save</button>
		</div>
	</div>

	<div class="settings-grid">
		<!-- General Section -->
		<div class="settings-section">
			<h4>GENERAL</h4>
			
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Start with Windows</span>
					<span class="setting-desc">Launch RazerLine automatically on system startup</span>
				</div>
				<label class="toggle">
					<input type="checkbox" checked={profileState.config.start_with_windows} onchange={toggleStartWithWindows} />
					<span class="toggle-slider"></span>
				</label>
			</div>

			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Minimize to Tray on Close</span>
					<span class="setting-desc">Clicking the window close button hides to system tray instead of exiting</span>
				</div>
				<label class="toggle">
					<input type="checkbox" checked={profileState.config.close_to_tray} onchange={toggleCloseToTray} />
					<span class="toggle-slider"></span>
				</label>
			</div>

			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Config File Path</span>
					<span class="setting-desc">{getConfigPath()}</span>
				</div>
				<div class="setting-value">{getConfigPath()}</div>
			</div>
		</div>

		<!-- Device Section -->
		{#if device.info}
		<div class="settings-section">
			<h4>ACTIVE DEVICE</h4>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Device</span>
					<span class="setting-desc">{device.info.name}</span>
				</div>
				<span class="device-badge">{device.info.name}</span>
			</div>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Product ID</span>
					<span class="setting-desc">0x{device.info.product_id.toString(16).toUpperCase().padStart(4, '0')}</span>
				</div>
				<span class="setting-value">0x{device.info.product_id.toString(16).toUpperCase().padStart(4, '0')}</span>
			</div>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Capabilities</span>
					<span class="setting-desc">
						{#if device.info.dpi_min !== null}DPI {device.info.dpi_min}-{device.info.dpi_max} {/if}
						{#if device.info.supported_poll_hz.length > 0}Poll Rate {device.info.supported_poll_hz.join(", ")}Hz {/if}
						{#if device.info.has_battery}Battery {/if}
						{#if device.info.led_zones.length > 0}Lighting ({device.info.led_zones.length} zones) {/if}
						{#if device.info.matrix_rows !== null}Matrix ({device.info.matrix_rows}x{device.info.matrix_cols}) {/if}
					</span>
				</div>
				<div class="setting-value"></div>
			</div>
		</div>
		{/if}

		<!-- About Section -->
		<div class="settings-section">
			<h4>ABOUT</h4>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Version</span>
					<span class="setting-desc">RazerLine v0.1.0</span>
				</div>
				<span class="setting-value">v0.1.0</span>
			</div>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Architecture</span>
					<span class="setting-desc">Tauri v2 + Svelte 5 + Rust</span>
				</div>
				<span class="setting-value">Tauri v2 + Svelte 5</span>
			</div>
			<div class="setting-row">
				<div class="setting-info">
					<span class="setting-label">Repository</span>
					<span class="setting-desc">github.com/razerline/razerline</span>
				</div>
				<a href="https://github.com/razerline/razerline" target="_blank" class="setting-link">View on GitHub</a>
			</div>
		</div>
	</div>
</div>

<style>
	.settings-card {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}

	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
	}

	.card-header h3 {
		margin: 0;
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.header-actions {
		display: flex;
		gap: 8px;
	}

	.settings-grid {
		display: flex;
		flex-direction: column;
		gap: 24px;
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}

	.settings-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.settings-section h4 {
		margin: 0 0 8px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 10px 12px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		transition: border-color 150ms;
	}

	.setting-row:hover {
		border-color: var(--accent);
	}

	.setting-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.setting-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--fg);
	}

	.setting-desc {
		font-size: 11px;
		color: var(--muted);
	}

	.setting-value {
		font-size: 12px;
		color: var(--muted);
		font-family: monospace;
	}

	.setting-link {
		font-size: 12px;
		color: var(--accent);
		text-decoration: none;
	}

	.setting-link:hover {
		text-decoration: underline;
	}

	.device-badge {
		font-size: 11px;
		font-weight: 500;
		padding: 4px 10px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--accent);
	}

	/* Toggle Switch */
	.toggle {
		position: relative;
		width: 44px;
		height: 24px;
		flex-shrink: 0;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: var(--border);
		border-radius: 24px;
		transition: background-color 200ms;
	}

	.toggle-slider:before {
		position: absolute;
		content: "";
		height: 18px;
		width: 18px;
		left: 3px;
		bottom: 3px;
		background-color: var(--fg);
		border-radius: 50%;
		transition: transform 200ms;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.toggle input:checked + .toggle-slider {
		background-color: var(--accent);
	}

	.toggle input:checked + .toggle-slider:before {
		transform: translateX(20px);
	}

	.toggle input:focus-visible + .toggle-slider {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}
</style>