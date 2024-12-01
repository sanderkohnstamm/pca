<script>
	import { onMount, onDestroy } from "svelte";
	let socket;
	let detectorMetadata = new Map();
	let detectorDetections = new Map();

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

	function isSocketOpen() {
		return socket && socket.readyState === WebSocket.OPEN;
	}

	function updateDetectors(data) {
		let newDetections = false;
		let newDetectors = false;

		data.forEach((detector) => {
			const { id, ip, detections } = detector;
			const existingMetadata = detectorMetadata.get(id);
			const existingDetections = detectorDetections.get(id);

			if (!existingMetadata || existingMetadata.ip !== ip) {
				detectorMetadata.set(id, { id, ip });
				newDetectors = true;
			}

			if (
				!existingDetections ||
				JSON.stringify(existingDetections) !==
					JSON.stringify(detections)
			) {
				detectorDetections.set(id, detections);
				newDetections = true;
			}
		});
		if (newDetections || newDetectors) {
			detectorMetadata = new Map(detectorMetadata); // Trigger reactivity
			detectorDetections = new Map(detectorDetections); // Trigger reactivity
		}
	}

	function removeDetector(id) {
		if (isSocketOpen()) {
			socket.send(JSON.stringify({ action: "remove", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function setToEmpty(id) {
		if (isSocketOpen()) {
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

	function hashStringToColor(str, alpha = 0.2) {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			hash = str.charCodeAt(i) + ((hash << 5) - hash);
		}
		const r = (hash >> 16) & 0xff;
		const g = (hash >> 8) & 0xff;
		const b = hash & 0xff;
		return `rgba(${r}, ${g}, ${b}, ${alpha})`;
	}
</script>

<main>
	<h1>Detectors</h1>
	<ul>
		{#each Array.from(detectorMetadata.values()) as { id, ip }}
			ID: {id} | IP: {ip}
			<li>
				<!-- <button on:click={() => setToEmpty(id)}>Set to Empty</button>
                <button on:click={() => removeDetector(id)}>Remove</button> -->
				<div class="detections-container">
					<!-- svelte-ignore a11y-missing-attribute -->
					<iframe
						class="background-iframe"
						src={`http://${ip}:8889/cam/`}
						width="1280"
						height="720"
					></iframe>
					{#each detectorDetections.get(id) || [] as detection}
						<div
							class="detection-box"
							style="
                                left: {(detection.bounding_box.center_x -
								detection.bounding_box.width / 2) *
								100}%;
                                top: {(detection.bounding_box.center_y -
								detection.bounding_box.height / 2) *
								100}%;
                                width: {detection.bounding_box.width * 100}%;
                                height: {detection.bounding_box.height * 100}%;
                                background-color: {hashStringToColor(
								detection.class,
							)};
                                border-color: {hashStringToColor(
								detection.class,
							)};
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
	}
	ul {
		list-style-type: none;
		padding: 0;
	}
	li {
		margin-bottom: 2em;
	}
	.detections-container {
		position: relative;
		display: inline-block;
	}
	.background-iframe {
		border: none;
		border-radius: 10px;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
	}
	.detection-box {
		position: absolute;
		border: 2px solid;
		border-radius: 5px;
		color: white;
		font-size: 0.8em;
		padding: 2px;
	}
</style>
