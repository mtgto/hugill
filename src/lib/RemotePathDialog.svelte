<script lang="ts">
type Props = {
    remotePath: string;
    isActive: boolean;
    onClose: () => void;
    onOpen: (workspaceFolder: string) => void;
};
let { isActive, onClose, onOpen, remotePath = $bindable() }: Props = $props();

const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
        onClose();
    }
};
</script>

<div class="modal" class:is-active={isActive}>
    <div class="modal-background" onclick={() => onClose()} aria-hidden={true}></div>
    <div class="modal-card">
        <header class="modal-card-head is-shadowless">
            <p class="modal-card-title">Enter the full path of the workspace folder</p>
        </header>
        <section class="modal-card-body">
            <input class="input" type="text" placeholder="Path of the Workspace Folder" bind:value={remotePath} />
        </section>
        <footer class="modal-card-foot is-flex is-justify-content-flex-end py-4">
            <div class="buttons">
                <button class="button" aria-label="close" onclick={() => onClose()}>Cancel</button>
                <button class="button is-success" disabled={!remotePath.startsWith("/")} aria-label="close" onclick={() => onOpen(remotePath)}>Open</button>
            </div>
        </footer>
    </div>
</div>
<svelte:window onkeydown={handleKeydown} />
