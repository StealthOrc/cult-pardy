<script lang="ts">
	import { CookieStore, type  SessionCookies } from '$lib/stores/cookies';
	import { authorization } from '$lib/api/ApiRequests';
    import { type ApiResponse,type DiscordUser } from 'cult-common';
	import { onMount } from 'svelte';

    export let discord_user: DiscordUser | null;
    let cookies : SessionCookies | null = null;
    CookieStore.subscribe(value => {
            cookies = value;
    });
    let isAdmin : ApiResponse | null
    let loaded = false;
    console.log(discord_user)

    onMount( async () => {
        isAdmin = await authorization();
        loaded = true;
    })

    function getUserName() {
        if (cookies == null) {
            return "Unknown";
        }
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
        return `https://cdn.discordapp.com/avatars/${discord_id.id}/${avatar_id}.png?size=640`;
    }
</script>

{#if loaded && isAdmin && cookies}  
    <div class="fixed top-5 w-full flex justify-center items-center z-10">
        <div class="relative bg-cultGrey p-4 rounded-lg flex items-center space-x-4 shadow-lg">
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
{/if}
