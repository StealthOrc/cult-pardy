<script lang="ts">
	import { cookieStore, getCookies, type cookies} from '$lib/stores/cookies';
    import { DiscordUser, type ApiResponse } from 'cult-common';
	import { onDestroy, onMount } from 'svelte';
	import { on } from 'svelte/events';

    export let discord_user: DiscordUser | null;


    let cookies : cookies;

    const subscribe = cookieStore.subscribe(value => {
        cookies = value.cookies;
    });

   
    let isAdmin : ApiResponse = {success: false};

    onMount( async () => {
        subscribe()
        let response = await fetch(`api/authorization`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            },
           'credentials': 'include',
            }
        )
        console.log(response);
        if (response.ok) {
            try {

                const setCookieHeaders = response.headers

                console.log('Set-Cookie headers:', setCookieHeaders);


                isAdmin = await response.json();
            } catch (e) {
                console.error(e);
            }
        } 
    })


    function getUserName() {
        if (!discord_user) {
            return cookies.userSessionId.id;
        }
        return discord_user.username
    }

    function getAvatar() {
        if (!discord_user) {
            return "https://cdn-icons-png.flaticon.com/512/149/149071.png";
        }

        let avatar_id = discord_user.avatar_id;
        let discord_id = discord_user.discord_id;
        return `https://cdn.discordapp.com/avatars/${discord_id}/${avatar_id}.png?size=640`;
    }
</script>

<div class="fixed top-5 w-full flex justify-center items-center z-10">
    <div class="relative bg-gray-700 p-4 rounded-lg flex items-center space-x-4 shadow-lg">
        <!-- Shadow Overlay -->
        <div class="absolute inset-0 bg-gray-900 opacity-50 rounded-lg z-[-1]"></div>
        {#key discord_user}
        <img src="{getAvatar()}" alt="Avatar" class="w-16 h-16 rounded-full">
        <div class="text-white text-lg flex flex-col items-center">
            {#if isAdmin.success}
                <span class="text-red-500 text-sm font-bold mb-1">[ADMIN]</span>
            {/if}
            <p class="text-center">{getUserName()}</p>
            {#if discord_user}
                <p class="text-xs">{cookies.userSessionId.id}</p> <!-- Use custom class for smaller text -->
            {/if}
        </div>
        {/key}
    </div>
</div>

