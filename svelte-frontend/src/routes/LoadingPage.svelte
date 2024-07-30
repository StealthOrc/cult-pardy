<script lang="ts">
    import Cookies from "js-cookie";
	import { onMount } from "svelte";
    import { cookieStore, dev_loaded,type cookies } from "$lib/stores/cookies";
    import {dev} from "$app/environment";
	import { session_data } from "$lib/api/ApiRequests";

    let loaded = false
    let cookies : cookies; 
    console.log("LOADING!!!")

    onMount(async () => {
        dev_loaded.subscribe(value => {
            loaded = value;
        })
        cookieStore.subscribe(value => {
            cookies = value;
        });
    //await new Promise(r => setTimeout(r, 500));
        if (dev) {
            let sessiondata = await session_data();
            if (sessiondata) {
                if (cookies.userSessionId.id != sessiondata.user_session_id.id) {
                    console.log("USER SESSION ID CHANGED");
                    Cookies.set("user-session-id", sessiondata.user_session_id.id);
                    cookieStore.update(value => {
                        value.userSessionId.id = sessiondata.user_session_id.id;
                        return value;
                    });
                }
                if (cookies.sessionToken != sessiondata.session_token.token) {
                    console.log("SESSION TOKEN CHANGED");
                    Cookies.set("session-token", sessiondata.session_token.token);
                    cookieStore.update(value => {
                        value.sessionToken = sessiondata.session_token.token;
                        return value;
                    });
                }
                console.log("DATA", sessiondata);
            }
            dev_loaded.set(true);
        }
        while(!loaded) {
            await new Promise(r => setTimeout(r, 5000));
        }
    })

 </script>

<h1>Loading...</h1>
        