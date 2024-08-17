<script lang="ts">
    export let video: Blob

</script>


<video src={URL.createObjectURL(video)} controls>
    <track kind="captions" />
</video>