import type { DTOSession, UserSessionId, WebsocketPing } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 

export type Session = {
    user_session_id: UserSessionId;
    dto_Session: DTOSession;
    ping : number;
}




export function createCurrentSessionsStore() {

    const currentSessions = writable<Session[]>([]);

    function addSession(dtoSession: DTOSession) {
        currentSessions.update((curr) => {
            if (curr.find((s) => s.user_session_id.id === dtoSession.user_session_id.id) != undefined){
                return curr.sort(doSort);
            }
            const session : Session = {
                user_session_id: dtoSession.user_session_id,
                dto_Session: dtoSession,
                ping: 0
            }
            curr.push(session);
            return curr.sort(doSort);
        });
    }

    function updateSessionPing(sessionId: UserSessionId, ping: number) {
        currentSessions.update((curr) => {
            const found: Session | undefined = curr.find((s) => s.user_session_id.id === sessionId.id);
            if (found == undefined) return curr.sort(doSort);
            found.ping = ping;
            return curr.sort(doSort);
        });
    }

    function updateSessionsPing(pings: WebsocketPing[]) {
        currentSessions.update((curr) => {
            pings.forEach((ping) => {
                const found: Session | undefined = curr.find((s) => s.user_session_id.id === ping.user_session_id.id);
                if (found != undefined) {
                    found.ping = ping.ping;
                }
            });
            return curr.sort(doSort);
        });
    }


    function removeSessionById(sessionId: UserSessionId) {
        currentSessions.update((curr) => {
            const found: Session | undefined = curr.find((s) => s.user_session_id.id === sessionId.id);
            if (found == undefined) return curr.sort(doSort);
            curr.splice(curr.indexOf(found), 1);
            return curr.sort(doSort);
        });
    }

    function setSessions(sessions: DTOSession[]) {
       const newSessions: Session[] = sessions.map((s) => {
            return {
                user_session_id: s.user_session_id,
                dto_Session: s,
                ping: 0
            }
        });
        currentSessions.set(newSessions.sort(doSort));
    }

    function subscribe(this: void, run: Subscriber<Session[]>): Unsubscriber {
        return currentSessions.subscribe(run);
    }

    function doSort(a: Session, b: Session) {
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
        updateSessionsPing,
        updateSessionPing,
        removeSessionById,
        setSessions,
        subscribe,
    }
}
