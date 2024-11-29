<script>
	import { onMount } from "svelte";

	let counters = [];
	let texts = [];
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
			console.log("WebSocket message:", event.data);
			try {
				const data = JSON.parse(event.data);
				console.log("json:", data);
				if (data.type === "update") {
					counters = data.counters || [];
					texts = data.texts || [];
				}
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

	function clearCounter(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "remove", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function incrementCounter(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "increment", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function decrementCounter(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "decrement", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function setCounter(id, count) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "set", id, count }));
		} else {
			console.error("WebSocket is not open");
		}
	}

	function setToEmpty(id) {
		if (socket && socket.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify({ action: "set_to_empty", id }));
		} else {
			console.error("WebSocket is not open");
		}
	}
</script>

<main>
	<h1>Counters</h1>
	<ul>
		{#each counters as { id, count }}
			<li>
				ID: {id}, Count: {count}
				<button on:click={() => incrementCounter(id)}>Increment</button>
				<button on:click={() => decrementCounter(id)}>Decrement</button>
				<button on:click={() => setCounter(id, 0)}>Set to 0</button>
				<button on:click={() => clearCounter(id)}>Remove</button>
			</li>
		{/each}
	</ul>

	<h1>Texts</h1>
	<ul>
		{#each texts as { id, text }}
			<li>
				ID: {id}, Text: {text}
				<button on:click={() => setToEmpty(id)}>Set to Empty</button>
			</li>
		{/each}
	</ul>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 800px;
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
</style>
