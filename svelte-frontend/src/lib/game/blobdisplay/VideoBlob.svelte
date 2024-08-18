<script lang="ts">
	import { CONST } from "$lib/const";
	import { mediaPlayerStore } from "$lib/stores/MediaPlayerStore";
	import type { BoardContext, MediaPlayerContext } from "$lib/types";
	import { getContext, onMount, setContext } from "svelte";

    export let video: Blob
    export let currUserIsAdmin: boolean = false;
    
    const boardCtx: BoardContext = getContext(CONST.BOARDCTX);
    setContext(CONST.MEDIAPLAYERCTX, {
        play: () => doPlay(),
        pause: () => doPause(),
    });
    mediaPlayerStore.set(getContext(CONST.MEDIAPLAYERCTX));

    function doPlay() {
        console.log("doPlay");
        if (!player)
            return;
        try {
            isPlayAllowed = true;
            player.play();
        } catch (error) {
            console.error(error);
        }
    };

    function doPause() {
        console.log("doPause, player: ", player);
        if (!player)
            return;
        try {
            console.log("doPause2");
            isPauseAllowed = true;
            player.pause();
        } catch (error) {
            console.error(error);
        }
    };
    
    let player: HTMLVideoElement | null = null;
    //true, if the video initiated the play/pause
    let isPlayAllowed: boolean = false;
    let isPauseAllowed: boolean = false;

    function onPlay(event: any) {
        console.log("onPlay", isPlayAllowed);
        if (!player)
            return;
        console.log("onPlay: time", player.currentTime);
        //if the play is coming from websocket then dont pause and dont send websocket message
        if (isPlayAllowed) 
        {
            isPlayAllowed = false;
            return;
        }
        //requesting play until then pause
        console.log("pausing because wait for ws");
        isPauseAllowed = true;
        player.pause();
        console.log("requesting start");
        boardCtx.requestPlay();
    }

    function onPause(event: any) {
        console.log("onPause", isPauseAllowed, event);
        if (!player)
            return;
        console.log("onPause: time", player.currentTime);
        if (isPauseAllowed) 
        {
            isPauseAllowed = false;
            return;
        }
        //requesting pause at current time in microseconds
        isPlayAllowed = true;
        player.play();
        let secs = Math.floor(player.currentTime * 1000000);
        console.log("requesting pause: ", secs);
        boardCtx.requestPause(secs);
    }

    function onEnded(event: any) {
        if (!player)
            return;
        isPauseAllowed = true;
        player.fastSeek(0);
    }

    onMount(() => {
        player = document.getElementById("player") as HTMLVideoElement;
    })

</script>


<video on:play={onPlay} on:pause={onPause} on:ended={onEnded} id="player" src={URL.createObjectURL(video)} controls={currUserIsAdmin}>
    <track kind="captions" />
</video>