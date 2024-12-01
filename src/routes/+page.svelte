<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type PodStatus = {
    name: string;
    container_name?: string;
    status: string;
};

type ClusterStatus = {
    context: string;
    namespace: string;
    pods: PodStatus[];
};

let name = $state("");
let greetMsg = $state("");
let context = $state("");
let namespace = $state("");
let pods = $state<PodStatus[]>([]);

async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
}

listen<ClusterStatus>("cluster-status", (event) => {
    const clusterStatus = event.payload;
    console.log("Received cluster status:", clusterStatus);
    context = clusterStatus.context;
    namespace = clusterStatus.namespace;
    pods = clusterStatus.pods;
});
</script>
<main class="container is-fluid">
    <h2 class="title px-3 pt-4">Pods</h2>
    <header class="columns pt-2 pb-0 px-3">
        <div class="column is-one-third">
            <p class="title is-6">Context</p>
            <p class="subtitle is-6">{context}</p>
        </div>
        <div class="column">
            <p class="title is-6">Namespace</p>
            <p class="subtitle is-6">{namespace}</p>
        </div>
    </header>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th>Container</th>
                <th>Name</th>
                <th>Status</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>
            {#each pods as pod}
                <tr>
                    <td>{pod.container_name ?? "-"}</td>
                    <td>{pod.name}</td>
                    <td>{pod.status}</td>
                    <td><button class="button is-small is-info">Open</button></td>
                </tr>
            {/each}
        </tbody>
    </table>
</main>

<style>
    .success {
        color: var(--bulma-success);
    }
</style>
