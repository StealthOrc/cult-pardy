<script lang="ts">
	import { onMount } from "svelte";
	import { session_data, SessionData } from "$lib/api/ApiRequests";
	import { CookieStore, dev_loaded, type SessionCookies } from "$lib/stores/cookies";
	import { on } from "svelte/events";




    let is_dev_loaded = false
    dev_loaded.subscribe(value => {
        is_dev_loaded = value;
    })
    let cookies : SessionCookies | null = null; 
    CookieStore.subscribe(value => {
            cookies = value;
    });
    console.log("LOADING!!!")
    let is_loading = false;

    onMount(async () => {
        if (dev_loaded) {
            if (is_loading) {
                return;
            }
            let sessiondata : SessionData | null = await session_data();
            while (sessiondata === null) {
                sessiondata = await session_data();
                await new Promise(r => setTimeout(r, 10000));
            }
            is_loading = true;
            CookieStore.update_with_sessionData(sessiondata);
            dev_loaded.set(true);
        }
    })

        



 </script>
<class class="absolute bg-white bg-opacity-60 z-10 h-full w-full flex items-center justify-center">
    <div class="flex items-center">
      <span class="text-3xl mr-4">Loading</span>
      <svg class="animate-spin h-8 w-8 text-gray-800" fill="none"
        viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
        </path>
      </svg>
    </div>
</class>
        