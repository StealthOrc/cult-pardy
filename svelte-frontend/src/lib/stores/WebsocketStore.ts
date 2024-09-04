import { dev } from "$app/environment";
import type { UserSessionId, WebsocketSessionEvent, WebsocketSessionId } from "cult-common";
import { webSocket, WebSocketSubject } from "rxjs/webSocket";
import { writable, type Subscriber, type Unsubscriber, type Writable } from "svelte/store";
import { deflateSync } from "fflate";

export let WebsocketStore: WebsocketStoreType;

function getWebsocketStore(): WebsocketStoreType {
    if (!WebsocketStore) {
        WebsocketStore = createWebsocketStore();
    }
    return WebsocketStore;
}

WebsocketStore = getWebsocketStore();

if (dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule) => {
            console.log("HOT RELOADING WEBSOCKET STORE", newModule, WebsocketStore);
            if (newModule) {
                console.log("HOT RELOADING WEBSOCKET STORE", newModule.WebsocketStore, WebsocketStore);
                WebsocketStore.store = newModule.WebsocketStore.store; // Preserve the store state
            }
        });
    }
}

function get_ws(lobbyId: string, userSessionId: UserSessionId, sessionToken: string): WebsocketStoreDataType {
    const host = location.host;
    console.log("HOST", host);
    const ws = webSocket({
        url: `ws://${host}/ws?lobby-id=${lobbyId}&user-session-id=${userSessionId.id}&session-token=${sessionToken}`,
        binaryType: 'arraybuffer',
        deserializer: (e) => e.data,
        serializer: (value: WebsocketSessionEvent) => {
            const json = JSON.stringify(value);
            const encoder = new TextEncoder();
            const binaryData = encoder.encode(json);
            const buffer = new ArrayBuffer(binaryData.length);
            const view = new Uint8Array(buffer);
            view.set(binaryData);
            const deflated = deflateSync(view);
            return deflated;
        }
    });
    return { webSocketSubject: ws, websocket_id: { id: "NEW_WS" } };
}

export type WebsocketStoreDataType = {
    webSocketSubject: WebSocketSubject<WebsocketSessionEvent>;
    websocket_id: WebsocketSessionId;
};

export type WebsocketStoreType = {
    store: Writable<WebsocketStoreDataType>;
    stop: () => void;
    new_ws: (lobbyId: string, userSessionId: UserSessionId, sessionToken: string) => void;
    update_websocket_id: (id: WebsocketSessionId) => void;
    subscribe: (this: void, run: Subscriber<WebsocketStoreDataType>) => Unsubscriber;
};

function createWebsocketStore(): WebsocketStoreType {
    const store = writable<WebsocketStoreDataType>();

    function new_ws(lobbyId: string, userSessionId: UserSessionId, sessionToken: string) {
        store.update((ws) => {
            if (ws?.webSocketSubject && !ws.webSocketSubject.isStopped) {
                console.log("WS IS RUNNING", ws);
            } else {
                console.log("WS IS NOT RUNNING", ws);
                ws = get_ws(lobbyId, userSessionId, sessionToken);
            }
            return ws;
        });
    }

    function update_websocket_id(id: WebsocketSessionId) {
        store.update((ws) => {
            if (ws) {
                ws.websocket_id = id;
            }
            return ws;
        });
    }

    function subscribe(this: void, run: Subscriber<WebsocketStoreDataType>): Unsubscriber {
        return store.subscribe(run);
    }

    function stop() {
        store.update((ws) => {
            if (ws?.webSocketSubject) {
                ws.webSocketSubject.unsubscribe();
                ws.webSocketSubject.complete();
                ws.websocket_id = { id: "STOPPED_WS" };
            }
            return ws;
        });
    }

    return {
        store,
        update_websocket_id,
        stop,
        new_ws,
        subscribe,
    };
}
