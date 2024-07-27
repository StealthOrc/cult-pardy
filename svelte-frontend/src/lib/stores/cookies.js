import { writable } from "svelte/store";

const cookies = writable({
    "user-session-id": "", 
    "session-token": "" 
});

export function updateCookies() {
    cookies.set(getCookies());
}

function getCookies() {
    return document.cookie
        .split(";")
        .map(cookie => cookie.split("="))
        .reduce((obj, [key, value]) => {
            obj[key.trim()] = value;
            return obj;
        }, {});
}

export default cookies;