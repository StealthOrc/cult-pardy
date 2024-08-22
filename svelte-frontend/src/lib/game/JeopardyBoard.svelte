<script lang="ts">
    export const prerender = false;
    import JeopardyCategory from './JeopardyCategory.svelte';
    import { getContext, onMount, setContext } from 'svelte';
    import {WebSocketSubject} from "rxjs/webSocket";
    import { CookieStore, type SessionCookies} from "$lib/stores/cookies.js";
    import { match, P } from 'ts-pattern';
	import type { BoardEvent, DtoJeopardyBoard, DTOSession, MediaState, SessionEvent, WebsocketEvent, WebsocketServerEvents, WebsocketSessionEvent } from 'cult-common';
	import Players from './Players.svelte';
	import {CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { SessionPingsStore } from '$lib/stores/SessionPings';
	import { newWebSocketStore, type WebsocketStoreType} from '$lib/stores/WebsocketStore';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { inflate } from 'fflate';
	import { CONST } from '$lib/const';
	import { SvelteDate } from 'svelte/reactivity';
	import type { MediaPlayerContext } from '$lib/types';
	import { mediaPlayerContextStore } from '$lib/stores/MediaPlayerStore';
	import { mediaStateStore } from '$lib/stores/MediaStateStore';
	import { get_global_time, timeout } from '$lib/lib';

    type Props = { 
        lobbyId: string;
    }

    let { lobbyId = "main" }: Props = $props();	

 
;

    let wsStore = newWebSocketStore(lobbyId, $CookieStore);	
    let ws = $wsStore;


    let loc= location.host;

    let gameData: DtoJeopardyBoard | null = $state(null);
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

    function handleEvent(event: WebsocketServerEvents): boolean {
        match(event)
        //BoardEvents
        .with({ Board: P.select() }, (boardEvent) => handleBoardEvent(boardEvent))
        //SessionEvents
        .with({Session: P.select() }, (sessionEvent) => handleSessionEvent(sessionEvent))
        //TextEvents
        .with({ Text: P.select() }, (textEvent) => console.log('Websocket textEvent not implemented:', textEvent))
        //ErrorEvents
        .with({ Error: P.select() }, (errorEvent) => {console.error('Websocket errorEvent:', errorEvent)})
        //WebsocketEvents
        .with({ Websocket: P.select() }, (websocketEvent) => handleWebsocketEvent(websocketEvent))
        .with({ ActionState: P.select()}, (data) => {
            // { Media: ActionMediaEvent } | { SyncForward: number } | { SyncBackward: number };
            match(data)
            .with({ Media: P.select()}, (data) => {
                match(data)
                .with({ChangeState: P.select()}, (data) => {
                    mediaStateStore.setMediaState(data);
                })
                .otherwise((data) => {
                console.error("undhandled ActionStateEvent: ",data) 
                })
            })
            .with({ SyncForward: P.select()}, (data) => {
                mediaStateStore.addForward(data);
            })
            .with({ SyncBackward: P.select()}, (data) => {
                mediaStateStore.addBackward(data);
            })
            return true;
        })
        .exhaustive();
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
            JeopardyBoardStore.setCurrent(data[0]);
            JeopardyBoardStore.setActionState(data[1]);
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
            // search inside currentSessions for an object with the same user_session_id as data, if not: add data
            CurrentSessionsStore.addSession(data); 
            return true;
        })
        .with({ SessionDisconnected: P.select() }, (data) => {
            console.log("Disconnected Session: ", data, currentSessions);
            CurrentSessionsStore.removeSessionById(data);
            SessionPingsStore.removeBySessionId(data);
            return true;
        })
        .with({ SessionsPing : P.select() }, (data) => {
            console.log("SHOUT NOT BE TRIGGERT!!!")
            SessionPingsStore.updateSessionsPing(data);
            return true;
        })  
        .with({SessionPing: P.select()}, (data) => {
            SessionPingsStore.updateWebsocketPing(data);
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
        .with({ WebsocketID: P.select() }, (data) => {
            console.log("WebsocketPing: ", data);
            wsStore.update_websocket_id(data);
            return true;
        })
        .exhaustive();
        return true;
    }

    function requestPlayerPlay(): boolean {
        console.log("requestPlayerStart");
        if (ws == null) {
            return false;
        }
        let playEvent: WebsocketSessionEvent = { VideoEvent: "Play" };
        ws.webSocketSubject.next(playEvent);
        return true;
    }

    function requestPlayerPause(currPlayTime: number): boolean {
        console.log("requestPlayerPause");
        if (ws == null) {
           return false; 
        }
        let pauseEvent: WebsocketSessionEvent = { VideoEvent: {Pause: currPlayTime} };
        ws.webSocketSubject.next(pauseEvent);
        return true;
    }

    function requestPlayerChang(state:MediaState): boolean {
        console.log("requestPlayerChang");
        if (ws == null) {
           return false; 
        }
        let pauseEvent: WebsocketSessionEvent = { VideoEvent: {ChangeState: state} };
        ws.webSocketSubject.next(pauseEvent);
        return true;
    }

    function requestPlayerSyncForward(value: number): boolean {
        console.log("requestPlayerSyncForward");
        if (ws == null) {
            return false;
        }
        let syncEvent: WebsocketSessionEvent = { SyncForwardRequest: value} ;
        ws.webSocketSubject.next(syncEvent);
        return true;
    }

    function requestPlayerSyncBackward(): boolean {
        console.log("requestPlayerSyncBackward");
        if (ws == null) {
            return false;
        }
        
        let syncEvent: WebsocketSessionEvent =  "SyncBackwardRequest" ;
        ws.webSocketSubject.next(syncEvent);
        return true;
    }

    function requestPlayerChangeState(state: MediaState): boolean {
        console.log("requestPlayerChangeState");
        if (ws == null) {
            return false;
        }
        let changeStateEvent: WebsocketSessionEvent = { VideoEvent: {ChangeState: state} };
        ws.webSocketSubject.next(changeStateEvent);
        return true;
    }


    onMount(async () => {
        if (ws != null) {
            ws.webSocketSubject.subscribe({
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
                                const updated = handleEvent(parsed);
                                if (updated) {
                                    updateGridColumns();
                                }
                            }

                        });
                    };
                },
                error: (error) => {
                    console.log(error);
                    wsStore.stop();
                }
            });

            for (let i = 0; i < CONST.num_time_sync_cycles; i++) {
		        await timeout(500);
		        ws.webSocketSubject.next( "SyncBackwardRequest" );
                await timeout(500);
                ws.webSocketSubject.next({SyncForwardRequest: get_global_time(0)});
	        }
            console.log("JeopardyBoard: Websocket connected");
            //print all $mediaStateStore
            console.log($mediaStateStore)
        }
    
    });
</script>

{#if gameData != null && gameData.categories != null}
    <div class="jeopardy-container">
        <div class=" text-center ">ID : {ws.websocket_id.id}</div>
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

