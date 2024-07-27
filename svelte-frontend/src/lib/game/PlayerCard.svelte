<script lang="ts">
    import type { DTOSession, DiscordID} from '$lib/cult_common';
    export let currrentSessions :DTOSession[]


    let default_avatar = "https://cdn-icons-png.flaticon.com/512/149/149071.png"
    
    console.log("currrentSessions:", currrentSessions)


    function getAvatar(DTOSession: DTOSession) {
        if (DTOSession.discord_user === null) {
            return default_avatar
        }
        let avatar_id = DTOSession.discord_user.avatar_id
        let discord_id = DTOSession.discord_user.discord_id
        return `https://cdn.discordapp.com/avatars/${discord_id}/${avatar_id}.png?size=64*8`
    }

    function getUserName(DTOSession: DTOSession) {
        if (DTOSession.discord_user === null) {
            return DTOSession.user_session_id
        }
        return DTOSession.discord_user.username
    }

</script>

<div class="player-container">
    {#each currrentSessions as player}
        <div class="player-card">
            <img src="{getAvatar(player)}" alt="Avatar" class="player-avatar">
            <h1 class="player-username">{getUserName(player)}</h1> 
            <h3 class="player-score">{player.score}</h3>
        </div>
    {/each}
</div>


<style>
    .player-container { 
        display: flex;
        justify-content: center;
        align-items: flex-end;
        position: fixed;
        bottom: 20px;
        width: 100%;
        z-index: 1;
    }

    .player-card {
        background-color: #f1f1f1;
        border: 1px solid #ddd;
        border-radius: 5px;
        padding: 10px;
        margin: 0 10px;
        box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
        display: flex;
        align-items: center;
        gap: 10px; 
    }

    .player-avatar {
        width: 50px;
        height: 50px;
        border-radius: 50%; 
    }

    .player-username {
        margin: 0; 
        font-size: 18px; 
        flex-grow: 1; 
    }

    .player-score {
        margin: 0;
        font-size: 18px; 
        color: #666; 
    }

    .player-icon {
        font-size: 24px;
        margin-right: 10px;
    }

    .player-card h3 {
        margin: 0;
        font-size: 18px;
    }
</style>