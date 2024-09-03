<script lang="ts">
    import type { DTOSession } from 'cult-common';
    import PlayerCard from './PlayerCard.svelte';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';


</script>

<div class="fixed flex flex-wrap justify-center items-start bottom-5 w-full z-40">
    {#each $CurrentSessionsStore as session}
        <PlayerCard {session}/>
    {/each}
</div>