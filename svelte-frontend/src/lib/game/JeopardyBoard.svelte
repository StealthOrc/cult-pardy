<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import PlayerCard from './PlayerCard.svelte';
    import { onMount } from 'svelte';
    import {webSocket} from "rxjs/webSocket";
    import { getCookies, type cookies } from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { DtoJeopardyBoard, DTOSession, WebsocketServerEvents } from 'cult-common';


    //import * as dd from "cult_common";
    
    
	//import type { DtoJeopardyBoard, DTOSession, WebsocketServerEvents } from 'cult_common';

    export let lobbyId: string = "main";	

    const cookies : cookies = getCookies();
    


    

    console.log("cookies:", cookies)

    let dto = {
        "test-data": "test-data",
    };
    
    let gameData: DtoJeopardyBoard;
    let currrentSessions: DTOSession[];


    console.log("userSessionId:", cookies.userSessionId);
    console.log("sessionToken:", cookies.sessionToken);

    const socket = webSocket({
        url: `ws://localhost:8000/ws?lobby-id=${lobbyId}&user-session-id=${cookies.userSessionId}&session-token=${cookies.sessionToken}`,
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
                message.then((data: string) => {
                    const parsed : WebsocketServerEvents = JSON.parse(data);


                    match(parsed)
                        //BoardEvents
                        .with({ Board: P.select() }, (boardEvent) => {
                            console.log("Event found:", boardEvent);
                            //CurrentBoard
                            match(boardEvent)
                                .with({ CurrentBoard: P.select() }, (currentBoard) => {
                                    console.log("Event found: ", currentBoard);
                                    gameData = currentBoard;
                                    console.log("gameData:", gameData);
                                    updateGridColumns();

                                 //Currently no added events
                                }).otherwise(() => {
                                    console.log("Event not found: ",message)
                                });
                        })

                        //SessionEvents
                        .with({Session: P.select() }, (boardEvent) => {
                            console.log("Event found:", boardEvent);
                            match(boardEvent)
                                .with({ CurrentSessions: P.select() }, (CurrentSessions) => {
                                    console.log("Event found: ", CurrentSessions);
                                    currrentSessions = CurrentSessions;
                                    updateGridColumns();
                                    //console.log(message)

                                //Currently no added events
                                }).otherwise(() => {
                                    console.log("Event not found: ",message)
                                });
                        })
                        //Currently no added events
                        .otherwise(() => {
                            console.log("Event not found: ",message)
                        });
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
        {#if gameData != null && gameData.categories != null}
            {#each gameData.categories as category}
                <JeopardyCategory {category} />
            {/each}
        {/if}
    </div>
</div>
<PlayerCard {currrentSessions} />

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