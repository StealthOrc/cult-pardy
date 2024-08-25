<script lang="ts">
    import type { DTOSession } from 'cult-common';
    import PlayerCard from './PlayerCard.svelte';
	import { CurrentSessionsStore, sortSessions } from '$lib/stores/SessionStore';

    let current_session : DTOSession[] = [];
    CurrentSessionsStore.subscribe(value => {
        let temp : DTOSession[] = [];
        value.forEach(session => {
            console.log("Session2", session);
            temp.push(session);
        });
        console.log("Current Sessions", temp);
        temp.sort(sortSessions);
        current_session = temp;
    })
</script>

<div class="player-container">
    {#each current_session as session}
        <PlayerCard {session}/>
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
        z-index: 2;
    }
</style>