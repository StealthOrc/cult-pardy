<script lang="ts">
    import { onMount } from 'svelte';
    import { type DtoQuestion, type DTOSession, type Vector2D } from 'cult-common';
    import PlayerCard from './PlayerCard.svelte';
	import type { WebSocketSubject } from 'rxjs/webSocket';
	import type { Session } from '$lib/stores/currentSessions';

    export let currentSessions: Session[];
    export let current: DtoQuestion | null;
    export let ws : WebSocketSubject<any> | null;
</script>

<div class="player-container">
    {#each currentSessions as session}
        <PlayerCard {session} {ws} {current}/>
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