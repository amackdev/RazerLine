<script lang="ts">
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import { profile, type ProfileSummary, type UserProfile } from "../state/profiles.svelte";
	import { device } from "../state/device.svelte";

	interface Props {
		delay?: number;
	}

	let { delay = 0 }: Props = $props();

	let editingId: string | null = $state(null);
	let editName = $state("");
	let showNewProfile = $state(false);
	let newProfileName = $state("");

	function startEdit(p: ProfileSummary) {
		editingId = p.id;
		editName = p.name;
	}

	function cancelEdit() {
		editingId = null;
		editName = "";
	}

	async function saveEdit() {
		if (editName.trim() && editingId) {
			await profile.renameProfile(editingId, editName.trim());
			editingId = null;
			editName = "";
		}
	}

	async function handleKeyDown(event: KeyboardEvent, p: ProfileSummary) {
		if (event.key === "Enter") {
			await saveEdit();
		} else if (event.key === "Escape") {
			cancelEdit();
		}
	}

	async function handleNewProfileKeyDown(event: KeyboardEvent) {
		if (event.key === "Enter" && newProfileName.trim()) {
			await createNewProfile();
		} else if (event.key === "Escape") {
			showNewProfile = false;
			newProfileName = "";
		}
	}

	async function createNewProfile() {
		if (newProfileName.trim()) {
			await profile.createProfile(newProfileName.trim());
			showNewProfile = false;
			newProfileName = "";
		}
	}

	async function duplicateProfile() {
		await profile.duplicateProfile();
	}

	async function handleDelete(p: ProfileSummary) {
		if (profiles.length <= 1) return;
		if (!confirm(`Delete profile "${p.name}"?`)) return;
		await profile.deleteProfile(p.id);
	}

	let profiles = $derived(profile.profiles);
	let activeProfile = $derived(profile.activeProfile);
	let activeProfileId = $derived(activeProfile?.id ?? "");
	let isOnlyProfile = $derived(profiles.length <= 1);

	// Sync local profiles with device state
	$effect(() => {
		if (device.info?.product_id) {
			profile.loadProfiles(device.info.product_id);
		}
	});

	onMount(() => {
		profile.loadConfig();
	});
</script>

<div class="card" in:fly={{ y: 8, duration: 280, delay }}>
	<div class="card-header">
		<h3>PROFILES</h3>
		<div class="header-actions">
			{#if !showNewProfile}
				<button class="btn btn-ghost" onclick={() => (showNewProfile = true)}>+ New Profile</button>
			{/if}
		</div>
	</div>

	<div class="profiles-list">
		{#each profiles as p}
			<div class="profile-row">
				{#if editingId === p.id}
					<input
						class="edit-input"
						bind:value={editName}
						onkeydown={(e) => handleKeyDown(e, p)}
						onblur={saveEdit}
					/>
				{:else}
					<div 
						class="profile-main" 
						role="button" 
						tabindex="0"
						onclick={() => !p.is_active && profile.switchProfile(p.id)}
						onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); if (!p.is_active) profile.switchProfile(p.id); }}}
					>
						<div class="profile-info">
							<span class={`profile-name ${p.is_active ? "active" : ""}`}>{p.name}</span>
							{#if p.is_active}
								<span class="badge active">ACTIVE</span>
							{/if}
						</div>
						<div class="profile-actions">
							<button class="btn-icon" onclick={(e) => { e.stopPropagation(); startEdit(p); }} title="Rename">✎</button>
							<button class="btn-icon" onclick={(e) => { e.stopPropagation(); duplicateProfile(); }} title="Duplicate">⎘</button>
							{#if !isOnlyProfile}
								<button class="btn-icon danger" onclick={(e) => { e.stopPropagation(); handleDelete(p); }} title="Delete">🗑</button>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/each}

		{#if showNewProfile}
			<div class="profile-row new-profile">
				<input
					class="edit-input"
					id="new-profile-name"
					bind:value={newProfileName}
					onkeydown={handleNewProfileKeyDown}
					placeholder="Profile name"
				/>
				<div class="profile-actions">
					<button class="btn btn-primary btn-sm" onclick={createNewProfile}>Create</button>
					<button class="btn btn-ghost btn-sm" onclick={() => { showNewProfile = false; newProfileName = ""; }}>Cancel</button>
				</div>
			</div>
		{/if}
	</div>

	{#if activeProfile}
		<div class="profile-details">
			<h4>ACTIVE PROFILE SETTINGS</h4>
			<div class="details-grid">
				<div class="detail-group">
					<label for="detail-dpi">DPI</label>
					<div id="detail-dpi" class="detail-value">{activeProfile.dpi}</div>
				</div>
				<div class="detail-group">
					<label for="detail-dpi-stages">DPI Stages</label>
					<div id="detail-dpi-stages" class="detail-value">
						{#each activeProfile.dpi_stages as stage, i}
							<span class={`stage ${i === activeProfile.current_stage_idx ? "active" : ""}`}>{stage}</span>
						{/each}
					</div>
				</div>
				<div class="detail-group">
					<label for="detail-poll-rate">Poll Rate</label>
					<div id="detail-poll-rate" class="detail-value">{activeProfile.poll_rate_hz ?? "Auto"} Hz</div>
				</div>
				<div class="detail-group">
					<label for="detail-brightness">Brightness</label>
					<div id="detail-brightness" class="detail-value">{activeProfile.brightness}%</div>
				</div>
				<div class="detail-group">
					<label for="detail-led-color">LED Color</label>
					<div id="detail-led-color" class="detail-value">
						<div class="color-swatch" style="background: rgb({activeProfile.led_color.join(',')})"></div>
						rgb({activeProfile.led_color.join(', ')})
					</div>
				</div>
				<div class="detail-group">
					<label for="detail-sync-zones">Sync Zones</label>
					<div id="detail-sync-zones" class="detail-value">{activeProfile.sync_zones ? "On" : "Off"}</div>
				</div>
				<div class="detail-group">
					<label for="detail-effect">Effect</label>
					<div id="detail-effect" class="detail-value">{activeProfile.effect}</div>
				</div>
				<div class="detail-group">
					<label for="detail-matrix-color">Matrix Color</label>
					<div id="detail-matrix-color" class="detail-value">
						{#if activeProfile.matrix_color}
							<div class="color-swatch" style="background: rgb({activeProfile.matrix_color.join(',')})"></div>
							rgb({activeProfile.matrix_color.join(', ')})
						{:else}
							—
						{/if}
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
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

	.profiles-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 24px;
	}

	.profile-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		transition: border-color 150ms, background 150ms;
	}

	.profile-row:hover {
		border-color: var(--accent);
	}

	.profile-row.new-profile {
		background: var(--surface-elevated);
		border-color: var(--accent);
	}

	.profile-main {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex: 1;
		cursor: pointer;
		min-width: 0;
	}

	.profile-main:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
		border-radius: 6px;
	}

	.profile-info {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}

	.profile-name {
		font-size: 14px;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.profile-name.active {
		color: var(--accent);
	}

	.badge {
		font-size: 10px;
		font-weight: 600;
		padding: 2px 6px;
		border-radius: 4px;
		text-transform: uppercase;
	}

	.badge.active {
		background: var(--accent);
		color: var(--bg);
	}

	.profile-actions {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.btn-icon {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: transparent;
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
		color: var(--muted);
		transition: background 150ms, color 150ms;
	}

	.btn-icon:hover {
		background: var(--surface-elevated);
		color: var(--fg);
	}

	.btn-icon.danger:hover {
		background: var(--danger);
		color: white;
	}

	.edit-input {
		flex: 1;
		padding: 6px 10px;
		border: 1px solid var(--accent);
		background: var(--bg);
		color: var(--fg);
		border-radius: 6px;
		font-size: 14px;
		outline: none;
	}

	.edit-input:focus {
		box-shadow: 0 0 0 2px var(--accent);
	}

	.profile-details {
		padding-top: 16px;
		border-top: 1px solid var(--border);
	}

	.profile-details h4 {
		margin: 0 0 12px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.details-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
		gap: 12px;
	}

	.detail-group {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.detail-group label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: var(--muted);
		text-transform: uppercase;
	}

	.detail-value {
		font-size: 13px;
		color: var(--fg);
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.stage {
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 12px;
		font-weight: 500;
		background: var(--surface);
		border: 1px solid var(--border);
		color: var(--muted);
	}

	.stage.active {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--bg);
	}

	.color-swatch {
		width: 16px;
		height: 16px;
		border-radius: 4px;
		border: 1px solid var(--border);
		flex-shrink: 0;
	}
</style>