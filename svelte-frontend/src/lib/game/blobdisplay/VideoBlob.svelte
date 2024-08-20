<script lang="ts">
	import { CONST } from "$lib/const";
	import { get_global_time } from "$lib/lib";
	import { mediaPlayerContextStore } from "$lib/stores/MediaPlayerStore";
	import { mediaStateStore, type MediaPlayerSessionType } from "$lib/stores/MediaStateStore";
	import type { BoardContext, MediaPlayerContext } from "$lib/types";
	import type { MediaState } from "cult-common";
	import { getContext, onMount, setContext } from "svelte";
	import JeopardyBoard from "../JeopardyBoard.svelte";
	import { JeopardyBoardStore } from "$lib/stores/JeopardyBoardStore";
	import JeopardyCategory from "../JeopardyCategory.svelte";
	import { match, P } from "ts-pattern";

    export let video: Blob
    export let currUserIsAdmin: boolean = false;


    let media: MediaPlayerSessionType;
    mediaStateStore.subscribe(value => {
        media = value;
    })


    enum EventStatusEnum {
        NONE = "NONE",
        PLAY = "PLAY",
        PAUSE = "PAUSE",
        SEEKING = "SEEKING",
        SEEKING_PLAY = "SEEKING_PLAY",
        PAUSE_SEEKING = "PAUSE_SEEKING",
    }

    enum EventSource {
        USER = "USER",
        WEBSOCKET = "WEBSOCKET",
    }




    let event_status = {
        websocket : EventStatusEnum.NONE,
        user: EventStatusEnum.NONE,
    }



    const boardCtx: BoardContext = getContext(CONST.BOARDCTX);
    setContext(CONST.MEDIAPLAYERCTX, {
        changeState: (state: MediaState) => doMediaStateChange(state)
    }); 
    mediaPlayerContextStore.set(getContext(CONST.MEDIAPLAYERCTX));


    function seekToTime(time: number) {
        return new Promise<void>((resolve, reject) => {
            const timer = setTimeout(() => {
                player.removeEventListener('seeked', onSeeked);
                reject(new Error('Seek operation timed out after 10 seconds'));
            }, 10000);

            function onSeeked() {
                clearTimeout(timer);
                player.removeEventListener('seeked', onSeeked);
                resolve();
            }

            player.addEventListener('seeked', onSeeked);
            player.currentTime = time;
        });
    }




    async function handleEvent(eventType: EventStatusEnum, state: MediaState) {
        const proposed_time = state.playing     ? (get_global_time(media.correction) - state.global_timestamp) / 1000 + state.video_timestamp  : state.video_timestamp;
        if (event_status.user === eventType) {
            if (Math.abs(proposed_time - player.currentTime) > CONST.PAUSED_THRESH && player.currentTime !== proposed_time) {
                // STILL NEED TO DO SOMETHING HERE
                event_status.user = EventStatusEnum.SEEKING;
                await seekToTime(proposed_time);
            }
            update_status(EventSource.USER);
            return true;
        }
        return false;
    }


    async function doMediaStateChange(state: MediaState) {
        try {
            if (await handleEvent(EventStatusEnum.SEEKING, state)) return;
            if (await handleEvent(EventStatusEnum.PLAY, state)) return;
            if (await handleEvent(EventStatusEnum.PAUSE, state)) return;

            if (state.playing) {
                determinePlayAndSeekPlay(state);
            } else {
                determinePauseAndPauseSeek(state);
            }
        } catch (error) {
            console.error('Error changing media state:', error);
        }
    }
    
    let player: HTMLVideoElement;


    function pause() {
        console.log("TEK PAUSE EVENT TRIGGERED", event_status);
        if (event_status.user == EventStatusEnum.PAUSE) {
            update_status(EventSource.USER);
            console.log("TEK IGNORE PAUSE USER", event_status);
            return;
        }
        if (event_status.websocket == EventStatusEnum.PAUSE) {
            update_status(EventSource.WEBSOCKET);
            console.log("TEK Pause triggered by WebSocket", event_status);
            return;
        }

        // STATE: USER: NONE | WEBSOCKET: NONE

        // STATE: USER: NONE | WEBSOCKET: NONE
        // print if USER != NONE && WEBSOCKET != PAUSE

        if (event_status.user != EventStatusEnum.NONE) {
            console.log("TEK Pause OWN STATUS NOT NONE!", event_status);
        }

        event_status.user = EventStatusEnum.PAUSE;
        const state: MediaState = {
            video_timestamp: player.currentTime,
            last_updated: get_global_time(media.correction),
            global_timestamp: get_global_time(media.correction),
            playing : false,
        }
        console.log("TEK SENDING PAUSE TO WS", event_status);
        boardCtx.changeMediaState(state);
    }

    function play(){
        if (!player)
            return;
        console.log("TEK PLAY EVENT TRIGGERED", event_status);
        if (event_status.user == EventStatusEnum.PLAY) {
            update_status(EventSource.USER);
            console.log("TEK IGNORE PLAY USER", event_status);
            return;
        }
        
        if (event_status.websocket == EventStatusEnum.PLAY) {
            update_status(EventSource.WEBSOCKET);
            console.log("TEK Play triggered by WebSocket", event_status);
            return;
        }

        if (event_status.user != EventStatusEnum.NONE) {
            console.log("TEK play OWN STATUS NOT NONE!", event_status);
        }

        event_status.user = EventStatusEnum.PLAY;
        const state: MediaState = {
            video_timestamp: player.currentTime,
            last_updated: get_global_time(media.correction),
            global_timestamp: get_global_time(media.correction),
            playing : true,
        }
        console.log("TEK SENDING PLAY TO WS!!!!!", event_status,player.currentTime);
        boardCtx.changeMediaState(state);
    }

    function seek() {
        console.log("_SEEEK!!",  player.currentTime, event_status);


        console.log("TEK SEEK EVENT TRIGGERED", event_status, player);
        if (!player)
            return;
        console.log("TEK SEEK EVENT TRIGGERED", event_status);
        if (event_status.user == EventStatusEnum.PAUSE_SEEKING || event_status.user == EventStatusEnum.SEEKING_PLAY ) { // || event_status.user == EventStatusEnum.SEEKING
            update_status(EventSource.USER);
            console.log("TEK IGNORE SEEK USER", event_status);
            return;
        }
        if (event_status.websocket == EventStatusEnum.PAUSE_SEEKING || event_status.websocket == EventStatusEnum.SEEKING_PLAY || event_status.websocket == EventStatusEnum.SEEKING) {
            update_status(EventSource.WEBSOCKET);
            console.log("TEK Seek triggered by WebSocket", event_status);
            return;
        }

        if (event_status.user != EventStatusEnum.NONE) {
            console.log("TEK SEEK OWN STATUS NOT NONE!", event_status);
        }

            
        let playing = !player.paused;

        event_status.user = EventStatusEnum.SEEKING;

        const state: MediaState = {
            video_timestamp: player.currentTime,
            last_updated: get_global_time(media.correction),
            global_timestamp: get_global_time(media.correction),
            playing,
        }

        console.log("TEK SENDING SEEK TO WS", event_status);
        boardCtx.changeMediaState(state);
    }


    const validStatuses = {
        [EventStatusEnum.PLAY]: EventStatusEnum.NONE,
        [EventStatusEnum.PAUSE]: EventStatusEnum.NONE,
        [EventStatusEnum.SEEKING]: EventStatusEnum.NONE,
        [EventStatusEnum.SEEKING_PLAY]: EventStatusEnum.PLAY,
        [EventStatusEnum.PAUSE_SEEKING]: EventStatusEnum.SEEKING,
        [EventStatusEnum.NONE]: EventStatusEnum.NONE,
    };



    function update_status(event_source: EventSource) {
        if (event_source === EventSource.USER) {
            event_status.user = validStatuses[event_status.user] || EventStatusEnum.NONE;
            console.log("TEK UPDATING STATUS USER", event_status);
        } else if (event_source === EventSource.WEBSOCKET) {
            event_status.websocket = validStatuses[event_status.websocket] || EventStatusEnum.NONE;
            console.log("TEK UPDATING STATUS WS", event_status);
        }
    }

    function new_status(old_status:EventStatusEnum): EventStatusEnum {
        return validStatuses[old_status] || EventStatusEnum.NONE;
    }

    function reset_status(event_source: EventSource) {
        if (event_source === EventSource.USER) {
            event_status.user = EventStatusEnum.NONE;
        } else if (event_source === EventSource.WEBSOCKET) {
            event_status.websocket = EventStatusEnum.NONE;
        }
    }
        



    function onEnded(event: any) {
        if (!player)
            return;
        try {
            player.load();
        } catch (error) {
            console.error('Error resetting video:', error);
        }
        const state: MediaState = {
            video_timestamp: player.currentTime,
            last_updated: get_global_time(media.correction),
            global_timestamp: get_global_time(media.correction),
            playing : false,
            
        }
        boardCtx.changeMediaState(state);
    }

    
    async function determinePlayAndSeekPlay(mediaState: MediaState) {
        if (!player) return;
        let proposed_time = (get_global_time(media.correction) - mediaState.global_timestamp) / 1000 + mediaState.video_timestamp; // Video timestamp from global timestamp from the server 
        let isSeeking = Math.abs(proposed_time - player.currentTime) > CONST.PAUSED_THRESH;
        if (event_status.websocket != EventStatusEnum.NONE) {
                    console.log("TEK WS NOT NONE", event_status); //WEBSOCKET NEED TO BE NONE
        }

        let isPlaying = !player.paused;

        //  144.274977 -  144.32711999999998 

        if (isSeeking && player.currentTime != proposed_time) {
            if (isPlaying) event_status.websocket = EventStatusEnum.SEEKING;
            else event_status.websocket = EventStatusEnum.SEEKING_PLAY;
            console.log("TEK DETERMINE PLAY SEEKING", event_status, proposed_time, player.currentTime, isSeeking);
            await seekToTime(proposed_time);
        } else {
            if (isPlaying) event_status.websocket = EventStatusEnum.NONE;
            else event_status.websocket = EventStatusEnum.PLAY;
            console.log("TEK DETERMINE PLAY", event_status);
        }

        console.log("TEK DETERMINE PLAY!", event_status);
        if (!isPlaying) player.play()
    }


    async function determinePauseAndPauseSeek(mediaState: MediaState) {
        if (!player) return;

        let isPause = player.paused;
        let proposed_time =  mediaState.video_timestamp; // Video timestamp from the video itself
        let isSeeking = (Math.abs(player.currentTime - proposed_time) > CONST.PAUSED_THRESH);  

        if (isSeeking) {
            if (isPause) event_status.websocket = EventStatusEnum.SEEKING; // SEEKING -> NONE
            else event_status.websocket = EventStatusEnum.PAUSE_SEEKING; // PAUSE_SEEKING -> SEEKING -> NONE
        } else {
            if (isPause) event_status.websocket = EventStatusEnum.NONE; // NONE 
            else event_status.websocket = EventStatusEnum.PAUSE; // PAUSE -> NONE
        }
        console.log("TEK DETERMINE PAUSE", event_status);
        if (!isPause) player.pause()

        if (isSeeking) {
            await seekToTime(proposed_time);
        }
    }



    onMount(async () => {
        player = document.getElementById("player") as HTMLVideoElement;
        if (player == null) return;
        
        match(JeopardyBoardStore.getActionState())
        .with({MediaPlayer: P.select()}, (data) => {
            if (!player) return
            if (data.playing) determinePlayAndSeekPlay(data);
            else determinePauseAndPauseSeek(data);
            mediaStateStore.setMediaState(data);
        })


    })


</script>





<video 
bind:this={player}
on:play={(event) => {
    console.log("VB play EVENT");
    play();
}} on:pause={(event) => {
    console.log("VB pause EVENT");
    pause();
}} on:ended={onEnded} 
on:seeked={(event) => {
    console.log("VB seeked EVENT", player)
    seek()
}} id="player" src={URL.createObjectURL(video)} controls={currUserIsAdmin} muted>
    <track kind="captions" />
</video>