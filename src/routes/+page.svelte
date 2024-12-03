<script lang="ts">
import RemotePathDialog from "$lib/RemotePathDialog.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { fade } from "svelte/transition";

type PodStatus = {
    name: string;
    container_name?: string;
    status: string;
    workspace_folder?: string;
};

type ClusterStatus = {
    context: string;
    namespace: string;
    pods: PodStatus[];
};

let context = $state("-");
let namespace = $state("-");
let pods = $state<PodStatus[]>([]);
let selectedPod = $state<PodStatus | null>(null);
let remotePath = $state("");
let successNotification = $state<string | null>(null);
let dangerNotification = $state<string | null>(null);

const handleClickOpen = async (workspaceFolder: string) => {
    if (selectedPod) {
        try {
            await invoke("open_remote_container", {
                context: context,
                namespace: namespace,
                podName: selectedPod.name,
                containerName: selectedPod.container_name ?? "",
                workspaceFolder: workspaceFolder,
            });
            dangerNotification = null;
            successNotification = "Success!";
            setTimeout(() => {
                successNotification = null;
            }, 2000);
        } catch (error) {
            successNotification = null;
            dangerNotification = "Failed to open remote container.";
        }
        selectedPod = null;
    }
};

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
    <button class="button" onclick={() => { dangerNotification = "Error!"; }}>Show Notification</button>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th>Container</th>
                <th>Name</th>
                <th>Status</th>
                <th>Workspace Folder</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>
            {#each pods as pod}
                <tr>
                    <td>{pod.container_name ?? "-"}</td>
                    <td>{pod.name}</td>
                    <td class="success">{pod.status}</td>
                    <td>/path/to/workspace</td>
                    <td><button class="button is-small is-info" onclick={() => { remotePath = "/"; selectedPod = pod; }}>Open</button></td>
                </tr>
            {/each}
        </tbody>
    </table>
    <RemotePathDialog isActive={selectedPod !== null} onClose={() => { selectedPod = null; }} onOpen={handleClickOpen} />
    {#if successNotification}
        <div class="notification is-success p-3 m-4" out:fade={{ duration: 2000 }}>
            {successNotification}
        </div>
    {/if}
    {#if dangerNotification}
        <div class="notification is-danger py-3 pl-3 pr-6 m-4">
            <button class="delete" aria-label="close" onclick={() => { dangerNotification = null; }}></button>
            {dangerNotification}
        </div>
    {/if}
</main>

<style>
    .success {
        color: var(--bulma-success);
    }
    .notification {
        position: fixed;
        right: 0;
        top: 0;
    }
    .notification > .delete {
        top: 0.5rem;
        right: 0.5rem;
    }
</style>
