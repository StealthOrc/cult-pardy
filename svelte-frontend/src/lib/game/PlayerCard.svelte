<script lang="ts" type="module">
    import { onMount } from 'svelte';
    import init, { DTOSession, DiscordUser, avatar_image_url2} from 'cult-common';
    import * as wasm from 'cult-common';
    export let session :DTOSession
    let default_avatar : string= "https://cdn-icons-png.flaticon.com/512/149/149071.png"



    function getAvatar() {
        if (session.discord_user === null) {
            return default_avatar
        }
        let avatar_id = session.discord_user.avatar_id
        let discord_id = session.discord_user.discord_id
        return `https://cdn.discordapp.com/avatars/${discord_id}/${avatar_id}.png?size=64*10`;
    }
    

    function getUserName(DTOSession: DTOSession) {
        if (DTOSession.discord_user === null) {
            return DTOSession.user_session_id
        }
        return DTOSession.discord_user.username
    }

</script>

<div class="player-card">
    <img src="{getAvatar()}" alt="Avatar" class="player-avatar">
    <h1 class="player-username">{getUserName(session)}</h1> 
    <h3 class="player-score">{session.score}</h3>
</div>



<style>
    .player-card {
        background-color: #f1f1f1;
        border: 1px solid #ddd;
        border-radius: 5px;
        padding: 10px;
        margin: 10px;
        box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        max-width: 200px; /* Adjust based on your design */
        box-sizing: border-box; /* Include padding and border in the width */
        overflow: hidden; /* Prevent overflow */
    }

    .player-avatar {
        width: 50px;
        height: 50px;
        border-radius: 50%;
        flex-shrink: 0; /* Prevent avatar from shrinking */
    }

    .player-details {
        display: flex;
        flex-direction: column;
        justify-content: center;
        flex-grow: 1; /* Allow the details container to grow */
        overflow: hidden; /* Ensure content stays within bounds */
    }

    .player-username {
        margin: 0;
        font-size: 18px;
        font-weight: bold;
        white-space: nowrap; /* Prevent text wrapping */
        overflow: hidden;
        text-overflow: ellipsis; /* Truncate text with ellipsis */
    }

    .player-score {
        margin: 0;
        font-size: 18px;
        color: #666;
    }
</style>