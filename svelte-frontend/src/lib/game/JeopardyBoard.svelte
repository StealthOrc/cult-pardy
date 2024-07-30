<script lang="ts">
    import Cookies from "js-cookie";
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { onMount } from 'svelte';
    import {webSocket, WebSocketSubject} from "rxjs/webSocket";
    import { cookieStore, dev_loaded, getCookies, type cookies } from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, SessionEvent, WebsocketServerEvents } from 'cult-common';
	import Players from './Players.svelte';
	import { dev } from '$app/environment';
	import { session_data } from '$lib/api/ApiRequests';

    //import * as dd from "cult_common";
    
    
	//import type { DtoJeopardyBoard, DTOSession, WebsocketServerEvents } from 'cult_common';

    export let lobbyId: string = "main";	

    let cookies : cookies;


    let dto = {
        "test-data": "test-data",
    };
    
    let is_dev_loaded = false


    let gameData: DtoJeopardyBoard;
    let currrentSessions: DTOSession[];
;

    let socket : WebSocketSubject<any>;

    function updateGridColumns() {
        console.log("gameData:", gameData);

        if (gameData == null || gameData.categories == null) {
            return;
        }
        console.log("gameData.categories:", gameData.categories);


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
        //Currently no added events
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
            console.log("Event found: ", data);
            currrentSessions = data;
            return true;
        }).otherwise(() => {
            console.log("Event not found: ",sessionEvent)
        });
        return true;
    }
</script>


<div class="jeopardy-container">
    <div class="jeopardy-board">
        {#if dev_loaded}
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
<Players {currrentSessions} />

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