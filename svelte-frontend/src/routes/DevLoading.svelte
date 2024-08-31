<script lang="ts">

	import { onMount } from "svelte";
	import { session_data, SessionData } from "$lib/api/ApiRequests";
	import { CookieStore, dev_loaded, is_loading, type SessionCookies } from "$lib/stores/cookies";



    let is_dev_loaded = false
    dev_loaded.subscribe(value => {
        is_dev_loaded = value;
    })
    let cookies : SessionCookies | null = null; 
    CookieStore.subscribe(value => {
            cookies = value;
    });
    console.log("LOADING!!!")
    let loading = false;

    is_loading.subscribe(value => {
        loading = value;
    })

    onMount(async () => {
        if (dev_loaded) {
            if (loading) {
                return;
            }
            is_loading.set(true);
            try {
                console.log("Request 1")
                let sessiondata : SessionData | null = await session_data();
                let tries = 0;
                    while (sessiondata === null) {
                        tries++;
                        console.log("Request" + tries);
                        sessiondata = await session_data();
                        await new Promise(r => setTimeout(r, 5000));
                    }
                    CookieStore.update_with_sessionData(sessiondata);
                    dev_loaded.set(true);
                    is_loading.set(false);
            } catch (e) {
                console.log("Backend not loaded");
            }
        }
    })

        



 </script>
<class class="bg-cult-gradient h-full w-full flex items-center justify-center">
    <div class="flex items-center text-white">
      <span class="text-3xl mr-4">Loading</span>
      <svg class="animate-spin h-8 w-8" fill="none"
        viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
        </path>
      </svg>
    </div>
</class>
        