<script>
	import { onMount, onDestroy } from "svelte";

	let socket;
	let detectors = [];

	// Automatically connect when the component is mounted
	onMount(() => {
		connect();
	});

	function connect() {
		socket = new WebSocket("ws://localhost:3030/ws");

		socket.onopen = () => {
			console.log("WebSocket connection established");
		};

		socket.onmessage = (event) => {
			console.log("WebSocket message:", event.data);
			try {
				const data = JSON.parse(event.data);
				updateDetectors(data);
			} catch (e) {
				console.error("Failed to parse WebSocket message:", e);
			}
		};

		socket.onerror = (error) => {
			console.error("WebSocket error:", error);
		};

		socket.onclose = () => {
			console.log("WebSocket connection closed");
			// Optionally, try to reconnect
			setTimeout(connect, 1000);
		};
	}

	function updateDetectors(data) {
		data.forEach((detector) => {
			const index = detectors.findIndex((d) => d.id === detector.id);
			if (index !== -1) {
				detectors[index] = detector;
			} else {
				detectors.push(detector);
			}
		});
	}

	function removeDetector(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "remove", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function setToEmpty(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "set_empty", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	// Clean up the WebSocket connection when the component is destroyed
	onDestroy(() => {
		if (socket) {
			socket.close();
		}
	});
</script>

<main>
	<h1>Detectors</h1>
	<ul>
		{#each detectors as { id, detections }}
			<li>
				ID: {id},
				<button on:click={() => setToEmpty(id)}>Set to Empty</button>
				<button on:click={() => removeDetector(id)}>Remove</button>
				<div class="detections-container">
					<iframe
						class="background-iframe"
						src="http://localhost:8889/cam/"
						width="1280"
						height="720"
					></iframe>
					{#each detections as detection}
						<div
							class="detection-box"
							style="
								left: {(detection.bounding_box.center_x - detection.bounding_box.width / 2) *
								100}%;
								top: {(detection.bounding_box.center_y - detection.bounding_box.height / 2) *
								100}%;
								width: {detection.bounding_box.width * 100}%;
								height: {detection.bounding_box.height * 100}%;
							"
						>
							Class: {detection.class}, Score: {detection.score}
						</div>
					{/each}
				</div>
			</li>
		{/each}
	</ul>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 1280px;
		margin: 0 auto;
		position: relative;
	}
	ul {
		list-style-type: none;
		padding: 0;
	}
	li {
		margin: 1em 0;
	}
	button {
		margin-left: 1em;
	}
	.detections-container {
		position: relative;
		width: 100%;
		height: 720px;
		border: 1px solid #ccc;
		overflow: hidden;
	}
	.background-iframe {
		position: absolute;
		width: 100%;
		height: 100%;
		border: none;
		top: 0;
		left: 0;
		z-index: 0;
	}
	.detection-box {
		position: absolute;
		border: 2px solid red;
		box-sizing: border-box;
		background-color: rgba(255, 0, 0, 0.1);
		z-index: 1;
	}
</style>
