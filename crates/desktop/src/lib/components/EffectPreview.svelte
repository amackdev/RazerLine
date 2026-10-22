<script lang="ts">
	// Real CSS-driven animation — direct upgrade of the old egui app's
	// lerp-based preview (widgets::effect_preview), which recomputed a
	// color/gradient by hand every repaint. The browser's compositor runs
	// these for free; a Static/Off preview requests zero repaints at all.
	let { effect, color }: { effect: string; color: [number, number, number] } = $props();

	let rgb = $derived(`rgb(${color[0]}, ${color[1]}, ${color[2]})`);
</script>

<div class="preview" class:breathing={effect === "breathing"} style:--color={rgb}>
	{#if effect === "static"}
		<div class="fill static"></div>
	{:else if effect === "breathing"}
		<div class="fill breathing-fill"></div>
	{:else if effect === "spectrum"}
		<div class="fill rainbow spectrum"></div>
	{:else if effect === "wave"}
		<div class="fill rainbow wave"></div>
	{:else}
		<div class="fill off"></div>
	{/if}
</div>

<style>
	.preview {
		height: 40px;
		border-radius: var(--radius-sm);
		overflow: hidden;
		background: #161719;
	}

	.fill {
		width: 100%;
		height: 100%;
	}

	.static {
		background: var(--color);
	}

	.off {
		background: #161719;
	}

	.breathing-fill {
		background: var(--color);
		animation: breathe 2.2s ease-in-out infinite;
	}

	@keyframes breathe {
		0%,
		100% {
			opacity: 0.12;
		}
		50% {
			opacity: 1;
		}
	}

	.rainbow {
		background: repeating-linear-gradient(
			90deg,
			#ff3b3b 0%,
			#ffd23b 14.28%,
			#3bff6e 28.57%,
			#3bfff0 42.85%,
			#3b6eff 57.14%,
			#a23bff 71.42%,
			#ff3b9e 85.71%,
			#ff3b3b 100%
		);
		background-size: 200% 100%;
	}

	.spectrum {
		animation: scroll-x 4s linear infinite;
	}

	.wave {
		animation: scroll-x 1.3s linear infinite;
	}

	@keyframes scroll-x {
		from {
			background-position: 0% 0;
		}
		to {
			background-position: -200% 0;
		}
	}
</style>
