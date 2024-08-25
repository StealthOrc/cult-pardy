
import { dev } from "$app/environment";
import type {  WebsocketSessionId } from "cult-common";

import { writable, type Subscriber, type Unsubscriber, type Writable} from "svelte/store"; 


// eslint-disable-next-line no-var
export var WebsocketSessionStore  = createWebsocketSessionStore();


if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.WebsocketSessionStore = WebsocketSessionStore;
            }
        });
    }
}

export type WebSocketType = {
    id : WebsocketSessionId;
    blob_loaded : boolean;
}


export type WebsocketSessionStoreType = {
    store: Writable<Map<WebsocketSessionId, WebSocketType>>;
    add: (websocket_id: WebsocketSessionId) => void;
    remove: (websocket_id: WebsocketSessionId) => void;
    update_blob_loaded: (websocket_id: WebsocketSessionId) => void;
    subscribe: (this: void, run: Subscriber<Map<WebsocketSessionId, WebSocketType>>) => Unsubscriber;
}




function createWebsocketSessionStore() : WebsocketSessionStoreType {

    const store = writable<Map<WebsocketSessionId, WebSocketType>>(new Map());

    function add(websocket_id: WebsocketSessionId) {
        store.update((curr) => {
            const newWebsocket = {id: websocket_id, blob_loaded: false};
            curr.set(websocket_id, newWebsocket);
            return curr;
        })
    }

    function remove(websocket_id: WebsocketSessionId) {
        store.update((curr) => {
            curr.delete(websocket_id);
            return curr;
        })
    }

    function update_blob_loaded(websocket_id: WebsocketSessionId) {
        store.update((curr) => {
            const websocket = curr.get(websocket_id);
            if (websocket != undefined) {
                websocket.blob_loaded = true;
                curr.set(websocket_id, websocket);
            }
            return curr;
        })
    }

    function subscribe(run: Subscriber<Map<WebsocketSessionId, WebSocketType>>): Unsubscriber {
        return store.subscribe(run);
    }


    return {
        store,
        add,
        remove,
        update_blob_loaded,
        subscribe
    }
}
