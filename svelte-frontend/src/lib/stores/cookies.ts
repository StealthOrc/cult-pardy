
import Cookies from "js-cookie";
import type { LobbyId, UserSessionId} from 'cult-common';
import { writable, type Subscriber, type Unsubscriber, type Writable } from 'svelte/store';
import { dev } from "$app/environment";
import { SessionData } from '../api/ApiRequests';


// fuction that returns the cookies and a type that represents the cookies
// the type is a writable store that contains the userSessionId and the sessionToken
// the userSessionId is a string that is the user session id
// the sessionToken is a string that is the session token
// the cookies are read from the cookies in the browser
// the cookies are updated when the function is called
// the cookies are updated by calling the update function on the userSessionId and sessionToken
// the update function is called with a function that gets the value of the cookie from the browser
// the value of the cookie is the value of the cookie from the browser or an empty string

const default_cookies: SessionCookies = {
    userSessionId: <UserSessionId>({ id: "" }),
    sessionToken: <string>(""),
};

export type SessionCookies = {
    userSessionId: UserSessionId;
    sessionToken: string;
};

function getCookies(): SessionCookies {
    default_cookies.userSessionId.id =  Cookies.get("user-session-id") || "";
    default_cookies.sessionToken = Cookies.get("session-token") || "";
    console.log("Cookies", default_cookies);
    return default_cookies;
}



export const CookieStore : CookieStoreType = createCookieStore();

export const dev_loaded : Writable<boolean> = writable(dev ? false : true);
export const is_loading : Writable<boolean> = writable(false);
export const lobby_store : Writable<LobbyId> = writable();


export type CookieStoreType = {
    store: Writable<SessionCookies>;
    update: (newCookies: SessionCookies) => void;
    update_with_sessionData: (sessionData: SessionData) => void;
    setCookies: (newCookies: SessionCookies) => void;
    update_userSessionId: (userSessionId: UserSessionId) => void;
    update_sessionToken: (sessionToken: string) => void;
    subscribe: (run: Subscriber<SessionCookies>) => Unsubscriber;
}


function createCookieStore() : CookieStoreType {

    const store : Writable<SessionCookies> = writable<SessionCookies>(getCookies());


    function setCookies(newCookies: SessionCookies) {
        store.set(newCookies);
    }

    function update(newCookies: SessionCookies) {
        store.update((cookies) => {
            if (cookies.userSessionId.id !== newCookies.userSessionId.id) {
                cookies.userSessionId.id = newCookies.userSessionId.id;
                Cookies.set("user-session-id", newCookies.userSessionId.id);
            }
            if (cookies.sessionToken !== newCookies.sessionToken) {
                cookies.sessionToken = newCookies.sessionToken;
                Cookies.set("session-token", newCookies.sessionToken);
            }
            return cookies;
        });
    }

    function update_with_sessionData(sessionData: SessionData) {
        console.log("SD: id", sessionData.user_session_id.id, "token", sessionData.session_token.token);


        store.update((cookies) => {
            console.log("curre id", cookies.userSessionId.id, "token", cookies.sessionToken);

            if (cookies.userSessionId.id !== sessionData.user_session_id.id) {
                cookies.userSessionId.id = sessionData.user_session_id.id;
                Cookies.set("user-session-id", sessionData.user_session_id.id);
            }
            if (cookies.sessionToken !== sessionData.session_token.token) {
                cookies.sessionToken = sessionData.session_token.token;
                Cookies.set("session-token", sessionData.session_token.token);
            }
            return cookies;
        });
    }



    function update_userSessionId(userSessionId: UserSessionId) {
        store.update((cookies) => {
            cookies.userSessionId = userSessionId;
            return cookies;
        });
    }

    function update_sessionToken(sessionToken: string) {
        store.update((cookies) => {
            cookies.sessionToken = sessionToken;
            return cookies;
        });
    }


    function subscribe(this: void, run: Subscriber<SessionCookies>): Unsubscriber {
        return store.subscribe(run);
    }

    

    return {
        store,
        update,
        update_with_sessionData,
        setCookies,
        update_userSessionId,
        update_sessionToken,
        subscribe,
    }
}