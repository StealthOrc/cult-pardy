import type { DTOSession, UserSessionId } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store";

export function createCurrentSessionsStore() {

    const currentSessions = writable<DTOSession[]>([]);

    function addSession(session: DTOSession) {
        currentSessions.update((curr) => {
            if (curr.find((s) => s.user_session_id.id === session.user_session_id.id) != undefined){
                return curr.sort(doSort);
            }
            curr.push(session);
            return curr.sort(doSort);
        });
    }

    function removeSessionById(sessionId: UserSessionId) {
        currentSessions.update((curr) => {
            const found: DTOSession | undefined = curr.find((s) => s.user_session_id.id === sessionId.id);
            if (found == undefined) return curr.sort(doSort);
            curr.splice(curr.indexOf(found), 1);
            return curr.sort(doSort);
        });
    }

    function setSessions(sessions: DTOSession[]) {
        currentSessions.set(sessions.sort(doSort));
    }

    function subscribe(this: void, run: Subscriber<DTOSession[]>): Unsubscriber {
        return currentSessions.subscribe(run);
    }

    function doSort(a: DTOSession, b: DTOSession) {
        if (a.user_session_id.id < b.user_session_id.id) {
            return -1;
        }
        if (a.user_session_id.id > b.user_session_id.id) {
            return 1;
        }
        return 0;
    }
    
    return {
        currentSessions,
        addSession,
        removeSessionById,
        setSessions,
        subscribe,
    }
}
