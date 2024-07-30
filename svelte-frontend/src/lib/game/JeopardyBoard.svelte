<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { onMount } from 'svelte';
    import {webSocket} from "rxjs/webSocket";
    import { getCookies, type cookies } from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, SessionEvent, WebsocketEvent, WebsocketServerEvents } from 'cult-common';
	import Players from './Players.svelte';

    export let lobbyId: string = "main";	

    const cookies : cookies = getCookies();

    let gameData: DtoJeopardyBoard;
    let currentSessions: DTOSession[] = [];

    const socket = webSocket({
        url: `ws://localhost:8000/ws?lobby-id=${lobbyId}&user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`,
        deserializer: (e) => e.data.text(),
    })

    function updateGridColumns() {
        if (gameData == null || gameData.categories == null) {
            return;
        }
        const categoriesSize = Object.keys(gameData.categories).length;
        const gridColumnTemplate = `repeat(${categoriesSize}, 1fr)`;
        document.documentElement.style.setProperty('--grid-columns', gridColumnTemplate);
    }

    onMount(() => {
        socket.subscribe({
            next: (message) => {
                message.then((data: string) => {
                    const parsed : WebsocketServerEvents = JSON.parse(data);
                    const updated = handleEvent(parsed);
                    if (updated) {
                        updateGridColumns();
                    }
                });
            },
            error: (error) => {
                console.log(error);
                //console.error('WebSocket error:', error);
            }
        });
    });

    function handleEvent(event: WebsocketServerEvents): boolean {
        match(event)
        //BoardEvents
        .with({ Board: P.select() }, (boardEvent) => handleBoardEvent(boardEvent))
        //SessionEvents
        .with({Session: P.select() }, (boardEvent) => handleSessionEvent(boardEvent))
        //TextEvents
        .with({ Text: P.select() }, (textEvent) => console.log('Websocket textEvent not implemented:', textEvent))
        //ErrorEvents
        .with({ Error: P.select() }, (errorEvent) => {console.error('Websocket errorEvent:', errorEvent)})
        //WebsocketEvents
        .with({ Websocket: P.select() }, (websocketEvent) => handleWebsocketEvent(websocketEvent))
        .otherwise(() => {
            console.log("Event not found: ",event)
        });

        return true;
    }

    // Handle BoardEvents
    function handleBoardEvent(boardEvent: BoardEvent): boolean {
        match(boardEvent)
        .with({ CurrentBoard: P.select() }, (data) => {
            console.log("Event found: ", data);
            gameData = data;
            updateGridColumns();
            return true;
        }).otherwise(() => {
            console.log("Event not found: ",boardEvent)
        });
        return true;
    }

    // Handle SessionEvents
    function handleSessionEvent(sessionEvent: SessionEvent): boolean {
        match(sessionEvent)
        .with({ CurrentSessions: P.select() }, (data) => {
            console.log("CurrentSessions: ", data, currentSessions);
            currentSessions = data;
            return true;
        })
        .with({ SessionJoined: P.select() }, (data) => {
            console.log("Joined Session: ", data, currentSessions);
            // search inside currentSessions for a object with the same user_session_id as data, if not: add data
            currentSessions = currentSessions.filter(session => session.user_session_id != data.user_session_id);
            currentSessions.push(data);
            
            console.log("Joined Session 2: ", data, currentSessions);
            return true;
        })
        .with({ SessionDisconnected: P.select() }, (data) => {
            console.log("Disconnected Session: ", data, currentSessions);
            currentSessions = currentSessions.filter(session => session.user_session_id != data);
            return true;
        })
        .exhaustive();
        return true;
    }

    //handle websocket joined and disconnected event
    function handleWebsocketEvent(websocketEvent: WebsocketEvent): boolean {
        //match joined and disconnected
        match(websocketEvent)
        .with({ WebsocketJoined: P.select() }, (data) => {
            console.log("Someone joined: ", data);
            return true;
        })
        .with({ WebsocketDisconnected: P.select() }, (data) => {
            console.log("Someone disconnected: ", data);
            return true;
        })
        .exhaustive();
        return true;
    }
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
<Players {currentSessions} />

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