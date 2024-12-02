<script lang="ts">
type Props = {
    isActive: boolean;
    onClickDone: () => void;
};
let { isActive, onClickDone }: Props = $props();
let remotePath = $state("/path/to/workspace");
</script>

<div class="modal" class:is-active={isActive}>
    <div class="modal-background" onclick={() => onClickDone()} aria-hidden="true"></div>
    <div class="modal-card">
        <header class="modal-card-head is-shadowless">
            <p class="modal-card-title">Enter the path of the Workspace Folder</p>
        </header>
        <section class="modal-card-body">
            <input class="input" type="text" placeholder="Path of the Workspace Folder" bind:value={remotePath} />
        </section>
        <footer class="modal-card-foot is-flex is-justify-content-flex-end py-4">
            <div class="buttons">
                <button class="button" aria-label="close" onclick={() => onClickDone()}>Cancel</button>
                <button class="button is-success" aria-label="close" onclick={() => onClickDone()}>Open</button>
            </div>
        </footer>
    </div>
</div>
