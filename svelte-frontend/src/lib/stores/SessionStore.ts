import { dev } from "$app/environment";
import type { DTOSession, UserSessionId } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 




export const CurrentSessionsStore = createCurrentSessionsStore();


if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.CurrentSessionsStore.store = CurrentSessionsStore.store;
            }
        });
    }
}

function createCurrentSessionsStore() {

    const store = writable<DTOSession[]>([]);

    function addSession(dtoSession: DTOSession) {
        store.update((curr) => {
            if (curr.find((s) => s.user_session_id.id === dtoSession.user_session_id.id) != undefined){
                return curr.sort(doSort);
            }
            curr.push(dtoSession);
            return curr.sort(doSort);
        });
    }



    function removeSessionById(sessionId: UserSessionId) {
        store.update((curr) => {
            const found: DTOSession | undefined = curr.find((s) => s.user_session_id.id === sessionId.id);
            if (found == undefined) 
                return curr.sort(doSort);
            curr.splice(curr.indexOf(found), 1);
            return curr.sort(doSort);
        });
    }

    function getSessionById(sessionId: UserSessionId) : DTOSession {
        let found: DTOSession | undefined = undefined;
        store.update((curr) => {
            found = curr.find((s) => s.user_session_id.id === sessionId.id);
            return curr;
        });
        if (found == undefined) {
            console.error(`SessionStore: Session with id ${sessionId.id} not found`);
            //TODO: throw error to handle with toast
            return {
                user_session_id: {
                    id: "unknown" 
                },
                score: 0,
                discord_user: null,
                is_admin: false
            };
        }
        return found;
    }
    
    function setSessions(sessions: DTOSession[]) {
        store.set(sessions.sort(doSort));
    }

    function subscribe(this: void, run: Subscriber<DTOSession[]>): Unsubscriber {
        return store.subscribe(run);
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
        store,
        addSession,
        removeSessionById,
        getSessionById,
        setSessions,
        subscribe,
    }
}
