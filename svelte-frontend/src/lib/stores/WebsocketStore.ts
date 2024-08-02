
import type { UserSessionId, WebsocketSessionEvent } from "cult-common";
import { webSocket, WebSocketSubject } from "rxjs/webSocket";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


// eslint-disable-next-line no-var
export var WebsocketStore : ReturnType<typeof createWebsocketStore> | null = null;


export function newWebSocketStore(lobbyId: string, userSessionId: UserSessionId, sessionToken: string): ReturnType<typeof createWebsocketStore> {
    if (WebsocketStore != null) {
        return WebsocketStore;
    }
    console.log("Creating new websocket store");
    const ws = createWebsocketStore(lobbyId, userSessionId, sessionToken);
    WebsocketStore = ws;
    return ws;
}






function get_ws(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) : WebSocketSubject<WebsocketSessionEvent> {
     return webSocket({
        url: `ws://localhost:8000/ws?lobby-id=${lobbyId}&user-session-id=${userSessionId.id}&session-token=${sessionToken}`,
        //use binaryType: 'arraybuffer' if you are sending binary data
        binaryType: 'arraybuffer',
        deserializer: (e) => e.data,
        serializer: (value: WebsocketSessionEvent) => {
            const json = JSON.stringify(value);
            const encoder = new TextEncoder();
            const binaryData = encoder.encode(json);
            const buffer = new ArrayBuffer(binaryData.length);
            const view = new Uint8Array(buffer);
            view.set(binaryData);
            return buffer;
        }
    });
}




function createWebsocketStore(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) {

    const lobby_id = lobbyId;
    const user_session_id = userSessionId;
    const session_token = sessionToken;

    const ws = writable<WebSocketSubject<WebsocketSessionEvent>>(get_ws(lobby_id, user_session_id, session_token));

    function new_ws() {
        ws.set(get_ws(lobby_id, user_session_id, session_token));
    }

    function subscribe(this: void, run: Subscriber<WebSocketSubject<WebsocketSessionEvent>>): Unsubscriber {
        return ws.subscribe(run);
    }

    function stop() {
        ws.update((ws) => {
            ws.complete();
            return ws;
        });
    }


    return {
        ws,
        stop,
        new_ws,
        subscribe,
    }
}
