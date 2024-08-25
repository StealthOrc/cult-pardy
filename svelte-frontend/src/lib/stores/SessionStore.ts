import { dev } from "$app/environment";
import type { DTOSession, UserSessionId } from "cult-common";
import { writable, type Subscriber, type Unsubscriber, type Writable} from "svelte/store"; 




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

export type CurrentSessionsStoreType = {
    store: Writable<Map<UserSessionIDRef, DTOSession>>;
    addSession: (dtoSession: DTOSession) => void;
    removeSessionById: (sessionId: UserSessionId) => void;
    getSessionById: (sessionId: UserSessionId) => DTOSession;
    setSessions: (sessions: DTOSession[]) => void;
    subscribe: (this: void, run: Subscriber<DTOSession[]>) => Unsubscriber;
}

export  function sortSessions(a: DTOSession, b: DTOSession) {
    if (a.user_session_id.id < b.user_session_id.id) {
        return -1;
    }
    if (a.user_session_id.id > b.user_session_id.id) {
        return 1;
    }
    return 0;
}

export type UserSessionIDRef = string;


function createCurrentSessionsStore() {


    const store = writable<Map<UserSessionIDRef, DTOSession>>(new Map());

    function addSession(dtoSession: DTOSession) {
        console.log(`Adding session: ${dtoSession.user_session_id}`);
        store.update((curr) => {
            console.log(`Current Map size before update: ${curr.size}`);
            const updated = new Map(curr);
            updated.set(dtoSession.user_session_id.id, dtoSession);
            console.log(`Current Map size after update: ${updated.size}`);
            return updated;
        });
    }

    function removeSessionById(sessionId: UserSessionId) {
        store.update((curr) => {
            curr.delete(sessionId.id);
            return curr;
        });
    }

    function getSessionById(sessionId: UserSessionId) : DTOSession {
        let found: DTOSession = {
            user_session_id: {id: ""},
            score: 0,
            is_admin: false,

        }
        store.update((curr) => {
            const temp = curr.get(sessionId.id);
            if (temp != undefined) {
                found = temp;
            }
            return curr;
        });
    
        return found;
    }
    
    function setSessions(sessions: DTOSession[]) {
        for (const session of sessions) {
            addSession(session);
        }
    }

    function subscribe(this: void, run: Subscriber<Map<UserSessionIDRef, DTOSession>>): Unsubscriber {
        return store.subscribe(run);
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
