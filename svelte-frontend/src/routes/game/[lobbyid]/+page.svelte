<script lang="ts">
    import JeopardyBoard from '$lib/game/JeopardyBoard.svelte';
	import BtnExit from '$lib/ui/BtnExit.svelte';

	export let data;
</script>

<JeopardyBoard lobbyId={data.lobbyid}/>
<BtnExit onclick={() => window.location.href = '/'} />