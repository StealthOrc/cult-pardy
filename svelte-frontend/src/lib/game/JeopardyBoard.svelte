<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { onMount } from 'svelte';
    import {webSocket, WebSocketSubject} from "rxjs/webSocket";
    import { cookieStore, dev_loaded, getCookies, type cookies } from "$lib/stores/cookies";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, SessionEvent, WebsocketEvent, WebsocketServerEvents } from 'cult-common';
	import Players from './Players.svelte';
	import { session_data } from '$lib/api/ApiRequests';
    import Cookies from "js-cookie";
	import { dev } from '$app/environment';

    export let lobbyId: string = "main";	

    let cookies : cookies;

    let gameData: DtoJeopardyBoard;
    let currentSessions: DTOSession[] = [];
    let is_dev_loaded = false;
    let socket : WebSocketSubject<any>;

    function updateGridColumns() {
        if (gameData == null || gameData.categories == null) {
            return;
        }
        const categoriesSize = Object.keys(gameData.categories).length;
        const gridColumnTemplate = `repeat(${categoriesSize}, 1fr)`;
        document.documentElement.style.setProperty('--grid-columns', gridColumnTemplate);
    }

    onMount(async () => {
        dev_loaded.subscribe(value => {
            is_dev_loaded = value;
        })
        cookieStore.subscribe(value => {
            cookies = value;
        });

        if (dev) {
            let sessiondata = await session_data();
            if (sessiondata) {
                if (cookies.userSessionId.id != sessiondata.user_session_id.id) {
                    console.log("USER SESSION ID CHANGED");
                    Cookies.set("user-session-id", sessiondata.user_session_id.id);
                    cookieStore.update(value => {
                        value.userSessionId.id = sessiondata.user_session_id.id;
                        return value;
                    });
                }
                if (cookies.sessionToken != sessiondata.session_token.token) {
                    console.log("SESSION TOKEN CHANGED");
                    Cookies.set("session-token", sessiondata.session_token.token);
                    cookieStore.update(value => {
                        value.sessionToken = sessiondata.session_token.token;
                        return value;
                    });
                }
                console.log("DATA", sessiondata);
            }
            dev_loaded.set(true);
        }


        socket = webSocket({
            url: `ws://localhost:8000/ws?lobby-id=${lobbyId}&user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`,
            deserializer: (e) => e.data.text(),
        })

        while(!is_dev_loaded) {
            setTimeout(() => {}, 500);
        }

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
            if (currentSessions.find(session => session.user_session_id.id == data.user_session_id.id)) {
                return false;
            }
            currentSessions = currentSessions.filter(session => session.user_session_id.id != data.user_session_id.id);
            currentSessions.push(data);
            return true;
        })
        .with({ SessionDisconnected: P.select() }, (data) => {
            console.log("Disconnected Session: ", data, currentSessions);

            //

            currentSessions = currentSessions.filter(session => session.user_session_id.id != data.id);
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
        {#if is_dev_loaded}
            {#if gameData != null && gameData.categories != null}
                {#each gameData.categories as category}
                    <JeopardyCategory {category} />
                {/each}
            {/if}
        {:else}
            <h1>Loading...</h1>
        {/if}
    </div>
</div>
{#key currentSessions}
    <Players {currentSessions} />
{/key}

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