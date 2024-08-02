<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { onMount } from 'svelte';
    import {webSocket, WebSocketSubject} from "rxjs/webSocket";
    import { CookieStore, type SessionCookies} from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, SessionEvent, WebsocketEvent, WebsocketServerEvents, WebsocketSessionEvent } from 'cult-common';
	import Players from './Players.svelte';
	import {CurrentSessionsStore } from '$lib/stores/SessionStore';
	import type { Observable, Observer } from 'rxjs';
	import { SessionPingsStore } from '$lib/stores/SessionPings';
	import { newWebSocketStore} from '$lib/stores/WebsocketStore';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';

    export let lobbyId: string = "main";	

    var cookies : SessionCookies | null = null;
    CookieStore.subscribe(value => {
        cookies = value;
    })
    if (cookies == undefined) {
        throw new Error("Cookies are null");
    }

    let wsStore = newWebSocketStore(lobbyId, cookies);
    let ws : WebSocketSubject<WebsocketSessionEvent> | null = null;
    wsStore.subscribe(value => {
        ws = value;
    })

    let gameData: DtoJeopardyBoard | null = null;

    JeopardyBoardStore.subscribe(value => {
        gameData = value;
    })

    let currentSessions: DTOSession[] = [];

    CurrentSessionsStore.subscribe(value => {
        currentSessions = value;
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

        if (ws != null) {
            ws.subscribe({
                next: (message) => {
                    if (message instanceof ArrayBuffer) {
                        const decoder = new TextDecoder();
                        let json : string = decoder.decode(message);
                        const parsed : WebsocketServerEvents = JSON.parse(json);
                        const updated = handleEvent(parsed);
                        if (updated) {
                            updateGridColumns();
                        }
                    };
                },
                error: (error) => {
                    console.log(error);
                    wsStore.stop();

                    //wsStore.new_ws();
                    //console.error('WebSocket error:', error);
                }
            });
        }
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
            JeopardyBoardStore.setBoard(data);
            updateGridColumns();
            return true;
        })
        .with({ CurrentQuestion: P.select() }, (data) => {
            JeopardyBoardStore.setCurrent(data);
            updateGridColumns();
            return true;
        })    
        .otherwise(() => {
            console.log("Event not found: ",boardEvent)
        });
        return true;
    }

    // Handle SessionEvents
    function handleSessionEvent(sessionEvent: SessionEvent): boolean {
        match(sessionEvent)
        .with({ CurrentSessions: P.select() }, (data) => {
            console.log("CurrentSessions: ", data, currentSessions);
            CurrentSessionsStore.setSessions(data);
            return true;
        })
        .with({ SessionJoined: P.select() }, (data) => {
            console.log("Joined Session: ", data, currentSessions);
            // search inside currentSessions for a object with the same user_session_id as data, if not: add data
            CurrentSessionsStore.addSession(data); 
            console.log("Joined Session 2: ", data, currentSessions);
            return true;
        })
        .with({ SessionDisconnected: P.select() }, (data) => {
            console.log("Disconnected Session: ", data, currentSessions);
            CurrentSessionsStore.removeSessionById(data);
            SessionPingsStore.removeBySessionId(data);
            return true;
        })
        .with({ SessionsPing : P.select() }, (data) => {
            SessionPingsStore.updateSessionsPing(data);
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

{#if gameData != null && gameData.categories != null}
    <div class="jeopardy-container">
        <div class="jeopardy-board">
                {#each gameData.categories as category}
                    <JeopardyCategory {category} />
                {/each}
        </div>
    </div>
    <Players/>
{/if}

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

