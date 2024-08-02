<script lang="ts">
    import { onMount } from 'svelte';
    import type { DtoQuestion, DTOSession, Vector2D, WebsocketPing } from 'cult-common';
    import PlayerCard from './PlayerCard.svelte';
	import type { WebSocketSubject } from 'rxjs/webSocket';
	import { SessionPingsStore } from '$lib/stores/SessionPings';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';


    let current_session : DTOSession[] = [];
    CurrentSessionsStore.subscribe(value => {
        current_session = value;
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