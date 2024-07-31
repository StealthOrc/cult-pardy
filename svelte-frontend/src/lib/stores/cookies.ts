
import Cookies from "js-cookie";
import type { UserSessionId} from 'cult-common';
import { writable, type Writable } from 'svelte/store';
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

const cookies: cookies = {
    userSessionId: <UserSessionId>({ id: "" }),
    sessionToken: <string>(""),
};

export type cookies = {
    userSessionId: UserSessionId;
    sessionToken: string;
};

export function getCookies(): cookies {
    cookies.userSessionId.id =  Cookies.get("user-session-id") || "";
    cookies.sessionToken = Cookies.get("session-token") || "";
    return cookies;
}

export function updateCookies(sessionData: SessionData | null) {
    if (sessionData == null) {
        return;
    }
    if (cookies.userSessionId.id != sessionData.user_session_id.id) {
        Cookies.set("user-session-id", sessionData.user_session_id.id);
        cookieStore.update(value => {
            value.userSessionId.id = sessionData.user_session_id.id;
            return value;
        });
    }
    if (cookies.sessionToken != sessionData.session_token.token) {
        Cookies.set("session-token", sessionData.session_token.token);
        cookieStore.update(value => {
            value.sessionToken = sessionData.session_token.token;
            return value;
        });
    }
}


export const cookieStore = writable(getCookies());

export const dev_loaded : Writable<boolean> = writable(dev ? false : true);
