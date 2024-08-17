<script lang="ts">
    import JeopardyBoard from '$lib/game/JeopardyBoard.svelte';

	export let data;
</script>

<JeopardyBoard lobbyId={data.lobbyid}/>