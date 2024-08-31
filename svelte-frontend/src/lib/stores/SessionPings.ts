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
            for (let i = 0; i < curr.length; i++) {
                if (curr[i].user_session_id.id === user_session_id.id) {
                    curr.splice(i, 1);
                    break;
                }
            }
            return curr;
        });
    }
    


    function updateWebsocketPing(websocketPing: WebsocketPing) {
        console.log("updateWebsocketPing", websocketPing);
        store.update((curr) => {
            let updated = false;
            for (let i = 0; i < curr.length; i++) {
                if (curr[i].user_session_id.id === websocketPing.user_session_id.id) {
                    curr[i] = websocketPing;
                    updated = true;
                    break;
                }
            }  
            if (!updated) {
                curr.push(websocketPing);
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
