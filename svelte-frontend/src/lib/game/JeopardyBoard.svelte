<script lang="ts">
    import JeopardyCategory from './JeopardyCategory.svelte';
    import PlayerCard from './PlayerCard.svelte';
    import { onMount } from 'svelte';
    import {webSocket} from "rxjs/webSocket";
    import cookies,{updateCookies} from "$lib/stores/cookies.js";

    export let lobbyId: string = "main";	

    updateCookies();
    console.log("cookies:", $cookies)

    let dto = {
        "test-data": "test-data",
    };

    let gameData: GameData = {
        creator: "",
        categories: [],
        current: null,
    };
    const userSessionId: string = $cookies["user-session-id"];
    const sessionToken: string = $cookies["session-token"];

    const socket = webSocket({
        url: `ws://localhost:8000/ws?lobby-id=${lobbyId}&user-session-id=${userSessionId}&session-token=${sessionToken}`,
        deserializer: (e) => e.data.text(),
    })

    function updateGridColumns() {
        const categoriesSize = Object.keys(gameData.categories).length;
        const gridColumnTemplate = `repeat(${categoriesSize}, 1fr)`;
        document.documentElement.style.setProperty('--grid-columns', gridColumnTemplate);
    }

    onMount(() => {


        socket.next(dto);

        socket.subscribe({
            next: (message) => {
                message.then((data) => {
                    const parsed = JSON.parse(data);
                    console.log("message:", parsed);
                    if(parsed.hasOwnProperty("Board")) {
                        gameData = parsed.Board.CurrentBoard;
                        console.log("gameData:", gameData);
                        updateGridColumns();
                        //console.log(message)
                    } else {
                        //console.log(message)
                    }
                });
            },
            error: (error) => {
                console.log(error);
                //console.error('WebSocket error:', error);
            }
        });
    });
</script>

<div class="jeopardy-container">
    <div class="jeopardy-board">
        {#if Object.keys(gameData).length > 0}
            {#each gameData.categories as category}
                <JeopardyCategory {category} />
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