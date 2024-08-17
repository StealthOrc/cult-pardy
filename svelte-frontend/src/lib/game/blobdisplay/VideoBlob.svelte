<script lang="ts">
    export let video: Blob
    export let currUserIsAdmin: boolean = false;

    console.log(currUserIsAdmin);
</script>


<video src={URL.createObjectURL(video)} controls={currUserIsAdmin}>
    <track kind="captions" />
</video>