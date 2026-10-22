<script lang="ts">
	import Card from "./Card.svelte";
	import SegmentedControl from "./SegmentedControl.svelte";
	import { api } from "../api";

	let { supportedHz, delay = 0 }: { supportedHz: number[]; delay?: number } = $props();

	let current = $state<number | null>(null);
	let error = $state("");

	$effect(() => {
		api
			.getPollRate()
			.then((hz) => (current = hz))
			.catch(() => {});
	});

	async function select(hz: number) {
		error = "";
		try {
			await api.setPollRate(hz);
			current = hz;
		} catch (e) {
			error = String(e);
		}
	}
</script>

<Card title="Polling Rate" {delay}>
	<SegmentedControl
		options={supportedHz.map((hz) => ({ label: `${hz} Hz`, value: hz }))}
		value={current}
		onselect={select}
	/>
	{#if error}
		<p class="error">{error}</p>
	{/if}
</Card>

<style>
	.error {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--danger);
	}
</style>
