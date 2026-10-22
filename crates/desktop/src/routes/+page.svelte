<script lang="ts">
	import { onMount } from "svelte";
	import { fade } from "svelte/transition";
	import "../lib/theme.css";

	import TopBar from "../lib/components/TopBar.svelte";
	import DpiCard from "../lib/components/DpiCard.svelte";
	import PollRateCard from "../lib/components/PollRateCard.svelte";
	import PowerCard from "../lib/components/PowerCard.svelte";
	import InfoCard from "../lib/components/InfoCard.svelte";

	import { device, refreshActiveDevice, startDeviceListeners } from "../lib/state/device.svelte";
	import { refreshDpi, startDpiListeners } from "../lib/state/dpi.svelte";
	import { startBatteryListeners } from "../lib/state/battery.svelte";

	onMount(() => {
		startDeviceListeners();
		startDpiListeners();
		startBatteryListeners();
		refreshActiveDevice();
	});

	$effect(() => {
		if (device.info) refreshDpi();
	});
</script>

<div class="app">
	<TopBar />

	<main>
		{#if !device.checked}
			<div class="status" in:fade={{ duration: 200 }}>Looking for a device…</div>
		{:else if !device.info}
			<div class="status" in:fade={{ duration: 200 }}>No Razer device connected</div>
		{:else}
			{#key device.info.product_id}
				<div class="grid">
					{#if device.info.dpi_min !== null && device.info.dpi_max !== null}
						<DpiCard min={device.info.dpi_min} max={device.info.dpi_max} delay={0} />
					{/if}
					{#if device.info.supported_poll_hz.length > 0}
						<PollRateCard supportedHz={device.info.supported_poll_hz} delay={60} />
					{/if}
					{#if device.info.has_battery || device.info.has_low_battery_threshold}
						<PowerCard
							hasIdleTime={device.info.idle_min_seconds !== null}
							idleMinSeconds={device.info.idle_min_seconds}
							idleMaxSeconds={device.info.idle_max_seconds}
							hasLowBatteryThreshold={device.info.has_low_battery_threshold}
							delay={120}
						/>
					{/if}
					{#if device.info.has_identity}
						<InfoCard delay={180} />
					{/if}
				</div>
			{/key}
		{/if}
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
	}

	main {
		flex: 1;
		overflow-y: auto;
		padding: 22px;
	}

	.grid {
		max-width: 640px;
		margin: 0 auto;
	}

	.status {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--muted);
		font-size: 14px;
	}
</style>
