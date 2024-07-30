<script lang="ts">
    import { onMount } from 'svelte';
    import { type DTOSession } from 'cult-common';
    import PlayerCard from './PlayerCard.svelte';

    export let currrentSessions: DTOSession[];
</script>

<div class="player-container">
    {#each currrentSessions as session}
        <PlayerCard {session} />
    {/each}
</div>

<style>
    .player-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        align-items: flex-start;
        position: fixed;
        bottom: 20px;
        width: 100%;
        z-index: 1;
    }
</style>