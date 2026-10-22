<script lang="ts">
	import { fly } from "svelte/transition";

	let { title, delay = 0, children }: { title: string; delay?: number; children: any } =
		$props();
</script>

<section class="card" in:fly={{ y: 12, duration: 420, delay, opacity: 0, easing: (t) => t }}>
	<div class="card-head">
		<span class="accent-bar"></span>
		<h2>{title}</h2>
	</div>
	<div class="card-body">
		{@render children()}
	</div>
</section>

<style>
	.card {
		background: var(--card);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 18px 20px;
		margin-bottom: 14px;
	}

	.card-head {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 14px;
	}

	.accent-bar {
		width: 4px;
		height: 16px;
		border-radius: 2px;
		background: var(--accent);
		flex-shrink: 0;
	}

	h2 {
		font-size: 14px;
		font-weight: 600;
		margin: 0;
		letter-spacing: 0.01em;
	}
</style>
