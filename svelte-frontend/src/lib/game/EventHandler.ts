
import { match, P } from 'ts-pattern';
import type { BoardEvent, SessionEvent, WebsocketEvent, WebsocketServerEvents} from 'cult-common';

import {CurrentSessionsStore } from '$lib/stores/SessionStore';
import { SessionPingsStore } from '$lib/stores/SessionPings';
import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
import { mediaStateStore } from '$lib/stores/MediaStateStore';
import {  WebsocketStore } from '../stores/WebsocketStore';


export function handleEvent(event: WebsocketServerEvents): boolean {
    
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
    console.log("BoardEvent: ", boardEvent);
    match(boardEvent)
    .with({ CurrentBoard: P.select() }, (data) => {
        console.log("Event found: ", data);
        JeopardyBoardStore.setBoard(data);
        return true;
    })
    .with({ CurrentQuestion: P.select() }, (data) => {
        JeopardyBoardStore.setCurrent(data[0]);
        JeopardyBoardStore.setActionState(data[1]);
        return true;
    })    
    .otherwise(() => {
        console.log("Event not found: ",boardEvent)
    });
    return true;
}

// Handle SessionEvents
function handleSessionEvent(sessionEvent: SessionEvent): boolean {
    console.log("SessionEvent: ", sessionEvent);
    match(sessionEvent)
    .with({ CurrentSessions: P.select() }, (data) => {
        CurrentSessionsStore.setSessions(data);
        return true;
    })
    .with({ SessionJoined: P.select() }, (data) => {
        // search inside currentSessions for an object with the same user_session_id as data, if not: add data
        CurrentSessionsStore.addSession(data); 
        return true;
    })
    .with({ SessionDisconnected: P.select() }, (data) => {
        CurrentSessionsStore.removeSessionById(data);
        SessionPingsStore.removeBySessionId(data);
        return true;
    })
    .with({ SessionsPing : P.select() }, (data) => {
        SessionPingsStore.updateSessionsPing(data);
        return true;
    })  
    .with({SessionPing: P.select()}, (data) => {
        console.log("SessionPing: ", data);
        SessionPingsStore.updateWebsocketPing(data);
        return true;
    }) 
    .exhaustive();
    return true;
}

//handle websocket joined and disconnected event
function handleWebsocketEvent(websocketEvent: WebsocketEvent): boolean {
    //match joined and disconnected
    console.log("WebsocketEvent: ", websocketEvent);
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
        const store = WebsocketStore;
        store.update_websocket_id(data);
        return true;
    })
    .exhaustive();
    return true;
}