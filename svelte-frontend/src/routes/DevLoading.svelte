<script lang="ts">
    import Cookies from "js-cookie";
	import { onMount } from "svelte";
    import { cookieStore, dev_loaded,updateCookies,type cookies } from "$lib/stores/cookies";
    import {dev} from "$app/environment";
	import { session_data } from "$lib/api/ApiRequests";

    let is_dev_loaded = false
    dev_loaded.subscribe(value => {
        is_dev_loaded = value;
    })
    let cookies : cookies; 
    cookieStore.subscribe(value => {
            cookies = value;
    });
    console.log("LOADING!!!")
    let loaded = false;

    onMount(async () => {
        if (dev) {
            let sessiondata = await session_data();
            updateCookies(sessiondata);
            dev_loaded.set(true);
        }
        while(!is_dev_loaded) {
            await new Promise(r => setTimeout(r, 50));
        }
        loaded = true;
    })

 </script>

<div class=" h-dvh w-dvw flex flex-col items-center justify-center gap-2">
    {#key is_dev_loaded}
            <div class="loader ease-linear rounded-full border-8 border-t-8 border-gray-200 h-64 w-64"></div>
            <h1>Loading..</h1>
    {/key}
</div>
        