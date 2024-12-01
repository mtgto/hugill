<script lang="ts">
type Props = {
    isActive: boolean;
    onClickDone: () => void;
};
let { isActive, onClickDone }: Props = $props();
let remotePath = $state("");
</script>

<div class="modal" class:is-active={isActive}>
    <div class="modal-background"></div>
    <div class="modal-content">
        {remotePath}
    </div>
    <button class="modal-close is-large" aria-label="close" onclick={() => onClickDone()}></button>
</div>
