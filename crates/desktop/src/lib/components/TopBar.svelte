<script lang="ts">
	import { device } from "../state/device.svelte";
	import { battery } from "../state/battery.svelte";
	import BatteryBadge from "./BatteryBadge.svelte";
</script>

<header>
	<div class="brand">
		<span class="dot" class:connected={device.info !== null}></span>
		<span class="wordmark">RazerLine</span>
	</div>

	<div class="right">
		<BatteryBadge percent={battery.percent} charging={battery.charging} />
		{#if device.info}
			<span class="device-name">{device.info.name}</span>
		{:else if device.checked}
			<span class="device-name muted">No device connected</span>
		{/if}
	</div>
</header>

<style>
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 22px;
		background: var(--bg-elevated);
		border-bottom: 1px solid var(--border);
		-webkit-app-region: drag;
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--muted);
		transition: background 0.4s, box-shadow 0.4s;
	}

	.dot.connected {
		background: var(--accent);
		box-shadow: 0 0 8px color-mix(in srgb, var(--accent) 70%, transparent);
	}

	.wordmark {
		font-weight: 700;
		font-size: 15px;
		color: var(--accent);
		letter-spacing: -0.01em;
	}

	.right {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.device-name {
		font-size: 12px;
		color: var(--text-dim);
	}

	.device-name.muted {
		color: var(--muted);
	}
</style>
