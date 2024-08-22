
import { dev } from "$app/environment";
import type { UserSessionId, WebsocketSessionEvent, WebsocketSessionId } from "cult-common";
import { webSocket, WebSocketSubject } from "rxjs/webSocket";
import { writable, type Subscriber, type Unsubscriber, type Writable} from "svelte/store"; 
import type { SessionCookies } from "./cookies";
import { deflateSync } from "fflate";


// eslint-disable-next-line no-var
var WebsocketStore : WebsocketStore;


export function newWebSocketStore(lobbyId: string, cookies:SessionCookies): WebsocketStore {
    if (WebsocketStore != null) {
        return WebsocketStore;
    }
    console.log("Creating new websocket store");
    const ws = createWebsocketStore(lobbyId, cookies.userSessionId, cookies.sessionToken);
    WebsocketStore = ws;
    return ws;
}


export function get_websocketStore() {
    return WebsocketStore;
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



function get_ws(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) : WebsocketStoreType {
    const host = location.host
    console.log("HOST", host)
    const ws = webSocket({
        url: `ws://${host}/ws?lobby-id=${lobbyId}&user-session-id=${userSessionId.id}&session-token=${sessionToken}`,
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
    return {webSocketSubject: ws, websocket_id: {id: ""}};
}

export type WebsocketStoreType = {
    webSocketSubject: WebSocketSubject<WebsocketSessionEvent>;
    websocket_id: WebsocketSessionId;
}



export type WebsocketStore = {
    store: Writable<WebsocketStoreType>;
    stop: () => void;
    new_ws: () => void;
    update_websocket_id: (id: WebsocketSessionId) => void;
    subscribe: (this: void, run: Subscriber<WebsocketStoreType>) => Unsubscriber;
}

function createWebsocketStore(lobbyId: string, userSessionId: UserSessionId, sessionToken: string): WebsocketStore {
    const lobby_id = lobbyId;
    const user_session_id = userSessionId;
    const session_token = sessionToken;

    const store = writable<WebsocketStoreType>(get_ws(lobby_id, user_session_id, session_token));

    function new_ws() {
        store.set(get_ws(lobby_id, user_session_id, session_token));
    }

    function update_websocket_id(id: WebsocketSessionId) {
        store.update((ws) => {
            ws.websocket_id = id;
            return ws;
        });
    }

    function subscribe(this: void, run: Subscriber<WebsocketStoreType>): Unsubscriber {
        return store.subscribe(run);
    }


    function stop() {
        store.update((ws) => {
            ws.webSocketSubject.unsubscribe();
            ws.webSocketSubject.complete();
            ws.websocket_id = {id: ""};
            return ws;
        });
    }



    return {
        store,
        update_websocket_id,
        stop,
        new_ws,
        subscribe,
    }
}
