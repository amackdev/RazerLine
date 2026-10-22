<script lang="ts">
	let { percent, charging }: { percent: number | null; charging: boolean | null } = $props();

	let fillColor = $derived(
		percent === null
			? "var(--muted)"
			: percent <= 15
				? "var(--danger)"
				: percent <= 35
					? "var(--warn)"
					: "var(--accent)",
	);
</script>

{#if percent !== null}
	<div class="badge" class:charging>
		<span class="percent">{percent}%</span>
		<div class="icon">
			<div class="body">
				<div class="fill" style:width="{percent}%" style:background={fillColor}></div>
			</div>
			<div class="nub"></div>
		</div>
	</div>
{/if}

<style>
	.badge {
		display: flex;
		align-items: center;
		gap: 7px;
	}

	.percent {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-dim);
		transition: color 0.3s;
	}

	.icon {
		display: flex;
		align-items: center;
	}

	.body {
		width: 22px;
		height: 11px;
		border: 1.3px solid var(--border-strong);
		border-radius: 2.5px;
		padding: 1.5px;
		display: flex;
		transition: border-color 0.4s;
	}

	.nub {
		width: 2px;
		height: 5px;
		background: var(--border-strong);
		border-radius: 0 1px 1px 0;
		transition: background 0.4s;
	}

	.fill {
		height: 100%;
		border-radius: 1px;
		transition: width 0.6s var(--ease-out);
	}

	.charging .body,
	.charging .nub {
		border-color: var(--accent);
		background: var(--accent);
		animation: pulse 1.8s ease-in-out infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.55;
		}
	}
</style>
