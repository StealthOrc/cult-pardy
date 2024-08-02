
import type { UserSessionId, WebsocketPing } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


export const SessionPingsStore = createCurrentPingsStore();


function createCurrentPingsStore() {

    const current_pings = writable<WebsocketPing[]>([]);


    function updateSessionsPing(pings: WebsocketPing[]) {
        current_pings.set(pings);
    }

    function removeBySessionId(user_session_id: UserSessionId) {
        current_pings.update((curr) => {
            const found: WebsocketPing | undefined = curr.find((s) => s.user_session_id.id === user_session_id.id);
            if (found == undefined) return curr;
            curr.splice(curr.indexOf(found), 1);
            return curr;
        });
    }


    function subscribe(this: void, run: Subscriber<WebsocketPing[]>): Unsubscriber {    
        return current_pings.subscribe(run);
    }

    function get_ping_update_by_session_id(this: void, user_session_id: UserSessionId,run: Subscriber<WebsocketPing>): Unsubscriber {
        return current_pings.subscribe((pings) => {
            const ping = pings.find((ping) => ping.user_session_id.id === user_session_id.id);
            if (ping != undefined) {
                run(ping);
            }
        });

    }

    return {
        current_pings,
        removeBySessionId,
        updateSessionsPing,
        subscribe,
        get_ping_update_by_session_id,
    }
}
