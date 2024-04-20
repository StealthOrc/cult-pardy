<script>
    import JeopardyCategory from './JeopardyCategory.svelte';
    import PlayerCard from './PlayerCard.svelte';
    import { onMount } from 'svelte';
    import {WebSocketSubject} from "rxjs/internal/observable/dom/WebSocketSubject";
    const dto = {
        id: 1,
        type: "starting_game_data",
    };

    let gameData = {};
    const socket = new WebSocketSubject('ws://localhost:8081/ws')

    function updateGridColumns() {
        const categoriesSize = Object.keys(gameData.categories).length;
        const gridColumnTemplate = `repeat(${categoriesSize}, 1fr)`;
        document.documentElement.style.setProperty('--grid-columns', gridColumnTemplate);
    }

    onMount(() => {


        socket.next(dto);

        socket.subscribe(
            (message) => {
                if(message.gametype === "GameData"){
                    gameData = message;
                    updateGridColumns();
                    console.log(message)
                } else {
                    console.log(message)
                }
            },
            (error) => {
                console.error('WebSocket error:', error);
            }
        );
    });
</script>

<div class="jeopardy-container">
    <div class="jeopardy-board">
        {#if Object.keys(gameData).length > 0}
            {#each Object.entries(gameData.categories) as [category, questions]}
                <JeopardyCategory {category} {questions} />
            {/each}
        {/if}
    </div>
</div>
<PlayerCard />

<style>
    .jeopardy-container {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
    }

    .jeopardy-board {
        display: grid;
        grid-template-columns: var(--grid-columns, repeat(5, 1fr));
        grid-gap: 10px;
        max-width: 90%;
        max-height: 90%;
    }
</style>