<script lang="ts">
    export const prerender = false;
    import { onMount } from 'svelte';
	import type { DtoQuestion } from 'cult-common';
	import YouTubePlayerPlus from 'youtube-player-plus';
	import type { YTPP_Options } from 'youtube-player-plus/types';
    
	export let current : DtoQuestion;
    export let youtube_id : string;

	let playerElement : HTMLElement | null = null;
	let player : YouTubePlayerPlus | null = null;

    function createYouTubePlayer() {
		console.log("Creating YouTube Player");
        if (current == null || playerElement == null) {
            return false;
        }
		console.log("Creating YouTube Player");
        let result = false;
		if (playerElement == null) {
				return false;
		}
        let options : YTPP_Options = {
                autoplay: true,
                controls: false,
                keyboard: false,
                loop: false,
                annotations: false,
                modestBranding: false,
                relatedVideos: false,
                playsInline: false,
        }

        player = new YouTubePlayerPlus(playerElement, options)
        player.load(youtube_id)
        player.setVolume(100)
        result = true;
        return result;
    }

	onMount(() => {
		playerElement = document.getElementById("ytplayer");
		createYouTubePlayer();
	});
</script>

<div id="ytplayer"></div>
{#if player}
	<div> 
		<h1>${current.value}</h1>
		<button on:click={() => player?.play()} class="cult-btn-menu">Play</button>
		<button on:click={() =>  {
			console.log("test", player);
			if (player != null) {
				player.pause();
			}
		}} class="cult-btn-menu">Pause</button>
		<button on:click={() => player?.stop()} class="cult-btn-menu">Stop</button>
	</div>
{/if}