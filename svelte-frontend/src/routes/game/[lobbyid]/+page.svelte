<script context="module" lang="ts">
    import type { PageData } from './$types';
    import JeopardyBoard from '$lib/game/JeopardyBoard.svelte';
    console.log("?");
    
    export let data: PageData;
</script>

<JeopardyBoard lobbyId={"main"}/>
