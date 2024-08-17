<script lang="ts">
    export let image: Blob

</script>


<div class="flex flex-col items-center p-6 bg-gray-100 rounded-lg shadow-lg">
    <img src={URL.createObjectURL(image)} alt=""class="mb-4 rounded-lg shadow-sm"/>
</div>