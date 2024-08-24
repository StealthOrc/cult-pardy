
import { dev } from "$app/environment";
import type {  UserSessionId, WebsocketSessionEvent, WebsocketSessionId } from "cult-common";
import { webSocket, WebSocketSubject } from "rxjs/webSocket";
import { writable, type Subscriber, type Unsubscriber, type Writable} from "svelte/store"; 
import { deflateSync } from "fflate";


// eslint-disable-next-line no-var
export var WebsocketStore : WebsocketStore = createWebsocketStore();


if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.WebsocketStore = WebsocketStore;
            }
        });
    }
}



function get_ws(lobbyId: string, userSessionId:UserSessionId, sessionToken:string) : WebsocketStoreType {
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
    webSocketSubject: WebSocketSubject<WebsocketSessionEvent>
    websocket_id: WebsocketSessionId;
}



export type WebsocketStore = {
    store: Writable<WebsocketStoreType>;
    stop: () => void;
    new_ws: (lobbyId: string, userSessionId:UserSessionId, sessionToken:string) => WebsocketStoreType;
    update_websocket_id: (id: WebsocketSessionId) => void;
    subscribe: (this: void, run: Subscriber<WebsocketStoreType>) => Unsubscriber;
}

function createWebsocketStore(): WebsocketStore {

    const store = writable<WebsocketStoreType>();

    function new_ws(lobbyId: string, userSessionId:UserSessionId, sessionToken:string) : WebsocketStoreType {
        store.update((ws) => {
            if (ws != undefined && ws.webSocketSubject != undefined) {
                ws.webSocketSubject.unsubscribe();
                ws.webSocketSubject.complete();
            }
            return ws;
        });

        const ws = get_ws(lobbyId, userSessionId, sessionToken);
        store.set(ws);
        return ws;
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
