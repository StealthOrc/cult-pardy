<script lang="ts">
    import { base } from "$app/paths";

    //get from static folder
    function getAvatar() {
        return `${base}/static/bild.png`
        ;
    }



    let lobbyid = '';
</script>
<div class=" h-dvh w-dvw flex flex-col items-center justify-center">

    <img src="{getAvatar()}" alt="Avatar" class="player-avatar">


    <input bind:value={lobbyid} class="border-2 border-blue-600 placeholder-blue-600 p-2 rounded m-2" type="text" name="lobby-id" id="lobby-id" placeholder="Lobby ID"/>
    <button on:click={() => window.location.href = `${base}/game/${lobbyid}`} class="bg-blue-600 hover:bg-blue-800 text-white font-bold py-2 px-4 rounded">Join Game</button>
    <button on:click={() => window.location.href = `${base}/game/main`} class="bg-blue-600 hover:bg-blue-800 text-white font-bold py-2 px-4 rounded">Join /main/ Game</button>
</div>