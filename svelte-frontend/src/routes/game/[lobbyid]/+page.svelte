<script lang="ts">
    import type { PageData } from './$types';
    import JeopardyBoard from '$lib/game/JeopardyBoard.svelte';
    
    export let data: PageData;
</script>
<JeopardyBoard lobbyId={"main"}/>