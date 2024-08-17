
import { dev } from "$app/environment";
import type { UserSessionId, WebsocketSessionEvent } from "cult-common";
import { webSocket, WebSocketSubject } from "rxjs/webSocket";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 
import type { SessionCookies } from "./cookies";
import { deflateSync } from "fflate";


// eslint-disable-next-line no-var
export var WebsocketStore : ReturnType<typeof createWebsocketStore> | null = null;


export function newWebSocketStore(lobbyId: string, cookies:SessionCookies): ReturnType<typeof createWebsocketStore> {
    if (WebsocketStore != null) {
        return WebsocketStore;
    }
    console.log("Creating new websocket store");
    const ws = createWebsocketStore(lobbyId, cookies.userSessionId, cookies.sessionToken);
    WebsocketStore = ws;
    return ws;
}


if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.WebsocketStore = WebsocketStore;
            }
        });
    }
}



function get_ws(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) : WebSocketSubject<WebsocketSessionEvent> {
     return webSocket({
        url: `ws://0.0.0.0:8000/ws?lobby-id=${lobbyId}&user-session-id=${userSessionId.id}&session-token=${sessionToken}`,
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
            const u8 = new Uint8Array(buffer);
            const deflated = deflateSync(u8);
            return deflated;
        }
    });
}



function createWebsocketStore(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) {
    const lobby_id = lobbyId;
    const user_session_id = userSessionId;
    const session_token = sessionToken;

    const store = writable<WebSocketSubject<WebsocketSessionEvent>>(get_ws(lobby_id, user_session_id, session_token));

    function new_ws() {
        store.set(get_ws(lobby_id, user_session_id, session_token));
    }

    function subscribe(this: void, run: Subscriber<WebSocketSubject<WebsocketSessionEvent>>): Unsubscriber {
        return store.subscribe(run);
    }

    function stop() {
        store.update((ws) => {
            ws.complete();
            return ws;
        });
    }


    return {
        store,
        stop,
        new_ws,
        subscribe,
    }
}
