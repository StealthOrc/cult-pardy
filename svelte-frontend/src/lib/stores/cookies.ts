
import Cookies from "js-cookie";
import type { UserSessionId, WebsocketSessionId} from 'cult-common';
import { writable } from 'svelte/store';


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
    sessionToken: <WebsocketSessionId>({ id: "" }),
};

export type cookies = {
    userSessionId: UserSessionId;
    sessionToken: WebsocketSessionId;
};

export function getCookies() {
    console.log("Updating cookies");
    cookies.userSessionId.id =  Cookies.get("user-session-id") || "";
    cookies.sessionToken.id = Cookies.get("session-token") || "";
    console.log("Updated cookies");
    return cookies;
}


export const cookieStore = writable({
    cookies: cookies,
});