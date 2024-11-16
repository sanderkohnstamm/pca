<script>
	import { onMount } from "svelte";

	let counter = 0;
	let socket;

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
			console.log("Message received from server:", event.data);
			const data = event.data;
			if (data.startsWith("Counter: ")) {
				counter = parseInt(data.split(": ")[1]);
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

	function incrementCounter() {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "increment" }));
		} else {
			console.error("WebSocket is not open");
		}
	}
</script>

<main>
	<button on:click={incrementCounter}>Increment Counter</button>
	<div class="counter">
		Counter: {counter}
	</div>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 800px;
		margin: 0 auto;
		position: relative;
	}
	.counter {
		margin-top: 1em;
		font-size: 1.5em;
	}
</style>
