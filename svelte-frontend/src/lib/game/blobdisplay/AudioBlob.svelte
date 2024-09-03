<script lang="ts">
    export let audio: Blob
</script>

<div class="flex flex-col items-center p-6 bg-gray-100 rounded-lg shadow-lg">
    <audio src={URL.createObjectURL(audio)} controls>
        <track kind="captions" />
    </audio>
</div>