<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { getContext, onMount, setContext } from 'svelte';
    import {WebSocketSubject} from "rxjs/webSocket";
    import { CookieStore, lobby_store, type SessionCookies} from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, LobbyId, MediaState, SessionEvent, WebsocketEvent, WebsocketServerEvents, WebsocketSessionEvent } from 'cult-common';
	import Players from './Players.svelte';
	import {CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { SessionPingsStore } from '$lib/stores/SessionPings';
	import {WebsocketStore, type WebsocketStoreType} from '$lib/stores/WebsocketStore';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { inflate } from 'fflate';
	import { CONST } from '$lib/const';
	import { get_global_time, timeout } from '$lib/lib';
	import { handleEvent } from './EventHandler';

    type Props = { 
        lobbyId: string;
    }

    let { lobbyId = "main" }: Props = $props();	

    let lobby_id :LobbyId = {id: lobbyId};
    lobby_store.set(lobby_id);

 
    const wsType : WebsocketStoreType = WebsocketStore.new_ws(lobbyId, $CookieStore.userSessionId, $CookieStore.sessionToken);
    const ws = wsType.webSocketSubject;


    let gameData: DtoJeopardyBoard | null = $state(null);
    JeopardyBoardStore.subscribe(value => {
        gameData = value;
        updateGridColumns();
    })

    function updateGridColumns() {
        if (gameData == null || gameData.categories == null) {
            return;
        }
        const categoriesSize = Object.keys(gameData.categories).length;
        const gridColumnTemplate = `repeat(${categoriesSize}, 1fr)`;
        document.documentElement.style.setProperty('--grid-columns', gridColumnTemplate);
    }


    onMount(async () => {
        if (ws != null) {
            ws.subscribe({
                next: (message) => {
                    if (message instanceof ArrayBuffer) {
                        let u8 = new Uint8Array(message);
                        inflate(u8, (err, infalte) => {
                            if (err) {
                                console.error('Deflation error:', err);
                            } else {
                                const decoder = new TextDecoder();
                                let json : string = decoder.decode(infalte);
                                const parsed : WebsocketServerEvents = JSON.parse(json);
                                handleEvent(parsed);
                            }

                        });
                    };
                },
                error: (error) => {
                    console.log(error);
                    JeopardyBoardStore.store.set(null);
                    SessionPingsStore.store.set([]);
                    CurrentSessionsStore.store.set([]);
                    console.log("Websocket error");
                    WebsocketStore.stop();
                    
                },
                complete: () => {
                    JeopardyBoardStore.store.set(null);
                    SessionPingsStore.store.set([]);
                    CurrentSessionsStore.store.set([]);
                    console.log("Websocket completed");
                    WebsocketStore.stop();
                }
            });

            for (let i = 0; i < CONST.num_time_sync_cycles; i++) {
		        ws.next( "SyncBackwardRequest" );
                timeout(20);
                ws.next({SyncForwardRequest: get_global_time(0)});
                timeout(20);
	        }
            console.log("JeopardyBoard: Websocket connected");
        }
    });
</script>

{#if gameData != null && gameData.categories != null}
    <div class="jeopardy-container">
        <div class=" text-center ">ID : {wsType.websocket_id.id}</div>
        <div class="jeopardy-board">
                {#each gameData.categories as category}
                    <JeopardyCategory {category}/>
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

