<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { onMount } from "svelte";

let name = $state("");
let greeting = $state("");

const greet = async () => {
    greeting = await invoke("greet", {
        name: name,
    });
};

onMount(async () => {
    // NOTE: onDragDropEvent returns cleanup function.
    await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "over") {
            console.log("User hovering");
        } else if (event.payload.type === "drop") {
            console.log("User dropped", event.payload.paths);
        } else {
            console.log("File drop cancelled");
        }
    });
});
</script>

<article class="container">
    <h1>Welcome to Tauri</h1>

    <div class="row">
        <a href="https://vitejs.dev" target="_blank">
            <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
            <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://www.typescriptlang.org/docs" target="_blank">
            <img
                src="/typescript.svg"
                class="logo typescript"
                alt="typescript logo"
            />
        </a>
    </div>
    <p>Click on the Tauri logo to learn more about the framework</p>

    <form
        class="row"
        id="greet-form"
        onsubmit={(e) => {
            e.preventDefault();
            greet();
        }}
    >
        <input
            id="greet-input"
            placeholder="Enter a name..."
            bind:value={name}
        />
        <button type="submit">Greet</button>
    </form>
    <p>{greeting}</p>
</article>
