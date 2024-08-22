

import { dev } from "$app/environment";
import type { UserSessionId, WebsocketPing } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


export const SessionPingsStore = createCurrentPingsStore();



if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.SessionPingsStore.store = SessionPingsStore.store;
            }
        });
    }
}


function createCurrentPingsStore() {

    const store = writable<WebsocketPing[]>([]);


    function updateSessionsPing(pings: WebsocketPing[]) {
        store.set(pings);
    }

    function removeBySessionId(user_session_id: UserSessionId) {
        store.update((curr) => {
            const found: WebsocketPing | undefined = curr.find((s) => s.user_session_id.id === user_session_id.id);
            if (found == undefined) return curr;
            curr.splice(curr.indexOf(found), 1);
            return curr;
        });
    }
    function updateWebsocketPing(websocketPing: WebsocketPing) {
        store.update((curr) => {
            const index = curr.findIndex((s) => s.user_session_id.id === websocketPing.user_session_id.id);
    
            if (index === -1) {
                curr.push(websocketPing);
            } else {
                curr[index] = { ...curr[index], ping: websocketPing.ping };
            }
    
            return curr;
        });
    }


    function subscribe(this: void, run: Subscriber<WebsocketPing[]>): Unsubscriber {    
        return store.subscribe(run);
    }

    function get_ping_update_by_session_id(this: void, user_session_id: UserSessionId,run: Subscriber<WebsocketPing>): Unsubscriber {
        return store.subscribe((pings) => {
            const ping = pings.find((ping) => ping.user_session_id.id === user_session_id.id);
            if (ping != undefined) {
                run(ping);
            }
        });

    }

    return {
        store,
        removeBySessionId,
        updateSessionsPing,
        subscribe,
        updateWebsocketPing,
        get_ping_update_by_session_id,
    }
}
