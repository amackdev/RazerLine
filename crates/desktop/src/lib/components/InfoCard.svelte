<script lang="ts">
	import Card from "./Card.svelte";
	import { api } from "../api";

	let { delay = 0 }: { delay?: number } = $props();

	let firmware = $state("");
	let serial = $state("");
	let error = $state("");
	let loading = $state(false);

	async function refresh() {
		loading = true;
		error = "";
		try {
			const [fw, sn] = await Promise.all([api.getFirmwareVersion(), api.getSerial()]);
			firmware = fw;
			serial = sn;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
</script>

<Card title="Device Info" {delay}>
	<button type="button" onclick={refresh} disabled={loading}>
		{loading ? "Refreshing…" : "Refresh Details"}
	</button>
	{#if firmware || serial}
		<dl>
			{#if firmware}
				<dt>Firmware</dt>
				<dd>{firmware}</dd>
			{/if}
			{#if serial}
				<dt>Serial</dt>
				<dd>{serial}</dd>
			{/if}
		</dl>
	{/if}
	{#if error}<p class="error">{error}</p>{/if}
</Card>

<style>
	button {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text);
		font-weight: 600;
		font-size: 12px;
		padding: 7px 14px;
		border-radius: 7px;
		cursor: pointer;
	}
	button:hover {
		border-color: var(--border-strong);
	}
	button:disabled {
		opacity: 0.6;
		cursor: default;
	}

	dl {
		margin: 14px 0 0;
		font-family: var(--font-mono);
		font-size: 12px;
	}
	dt {
		color: var(--muted);
		float: left;
		width: 70px;
		clear: left;
	}
	dd {
		margin: 0 0 4px;
	}

	.error {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--danger);
	}
</style>
