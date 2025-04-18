<script lang="ts">
import RemotePathDialog from "$lib/RemotePathDialog.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMount } from "svelte";
import { fade } from "svelte/transition";

type PodStatus = {
    name: string;
    containerName?: string;
    status: "Running" | "Waiting" | "Terminated" | string;
    labels: Record<string, string>;
    workspaceFolder?: string;
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
let uniqueWorkspaceFolders = $derived.by(() => {
    return pods.reduce((workspaceFolders: string[], pod) => {
        if (pod.workspaceFolder && !workspaceFolders.includes(pod.workspaceFolder)) {
            workspaceFolders.push(pod.workspaceFolder);
        }
        return workspaceFolders;
    }, []);
});

const isSamePod = (pod1: PodStatus | null, pod2: PodStatus): boolean => {
    if (pod1) {
        return pod1.name === pod2.name && pod1.containerName === pod2.containerName;
    } else {
        return false;
    }
};

// https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase
const classForStatus = (status: string): string => {
    switch (status) {
        case "Running":
            return "success";
        case "Waiting":
            return "warning";
        case "Terminated":
            return "danger";
        default:
            return "unknown";
    }
};

const handleClickOpen = async () => {
    if (selectedPod && remotePath.startsWith("/")) {
        try {
            await invoke("open_remote_container", {
                context: context,
                namespace: namespace,
                podName: selectedPod.name,
                containerName: selectedPod.containerName ?? "",
                labels: selectedPod.labels,
                workspaceFolder: remotePath,
            });
            dangerNotification = null;
            successNotification = "Success!";
            pods = pods.map((pod) => {
                if (isSamePod(selectedPod, pod)) {
                    return { ...pod, workspaceFolder: remotePath };
                }
                return pod;
            });
            setTimeout(() => {
                successNotification = null;
            }, 2000);
        } catch (error) {
            console.error(error);
            successNotification = null;
            if (typeof error === "string") {
                dangerNotification = error;
            } else {
                // TODO: remove this condition
                dangerNotification = "Failed to open remote container.";
            }
        }
        selectedPod = null;
    }
};

onMount(async () => {
    try {
        await invoke("start_cluster_watcher");
        console.log("Start watching cluster.");
    } catch (error) {
        console.error("Failed to watch cluster:", error);
        if (typeof error === "string") {
            dangerNotification = error;
        } else {
            // TODO: remove this condition
            dangerNotification = "Failed to watch cluster.";
        }
    }
});

listen<ClusterStatus>("cluster-status", (event) => {
    const clusterStatus = event.payload;
    console.log("Received cluster status:", clusterStatus);
    context = clusterStatus.context;
    namespace = clusterStatus.namespace;
    pods = clusterStatus.pods;
});

listen<string>("cluster-status-error", (event) => {
    console.error("Failed to get cluster status:", event.payload);
    dangerNotification = `Failed to get cluster status: ${event.payload}`;
});
</script>

<main class="container is-fluid">
    <div id="titlebar" data-tauri-drag-region></div>
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
                <th><abbr title="Status">Stat</abbr></th>
                <th>Container</th>
                <th>Name</th>
                <th>Workspace Folder</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>
            {#each pods as pod}
                <tr>
                    <td
                        ><span
                            class={"circle " + classForStatus(pod.status)}
                            title={pod.status}
                        ></span></td
                    >
                    <td>{pod.containerName ?? "-"}</td>
                    <td>{pod.name}</td>
                    <td>{pod.workspaceFolder ?? "-"}</td>
                    <td>
                        <button class="button is-small is-info" disabled={pod.status !== "Running"} onclick={() => {
                            remotePath = pod.workspaceFolder ?? "/";
                            selectedPod = pod;
                        }}>Open</button>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
    <RemotePathDialog isActive={selectedPod !== null} onClose={() => { selectedPod = null; }} onOpen={handleClickOpen} bind:remotePath workspaceFolders={uniqueWorkspaceFolders}/>
    {#if successNotification}
        <div class="notification is-success p-3 m-4" out:fade={{ duration: 2000 }}>{successNotification}</div>
    {/if}
    {#if dangerNotification}
        <div class="notification is-danger py-3 pl-3 pr-6 m-4">
            <button class="delete" aria-label="close" onclick={() => { dangerNotification = null; }}></button>
            {dangerNotification}
        </div>
    {/if}
</main>

<style>
    #titlebar {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
		height: 30px;

        + * {
            margin-top: 30px; /* height of the titlebar */
        }
    }
    .circle {
        border-radius: 50%;
        width: 1rem;
        height: 1rem;
        display: inline-block;

        &.success {
            background-color: #28ca42;
        }
        &.warning {
            background-color: #ffbf2f;
        }
        &.danger {
            background-color: #fd4943;
        }
        &.unknown {
            background-color: #b5b5b5;
        }
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
