<script lang="ts">

	import { CONST } from "$lib/const";
	import { get_global_time } from "$lib/lib";
	import { mediaPlayerContextStore } from "$lib/stores/MediaPlayerStore";
	import { mediaStateStore, type MediaPlayerSessionType } from "$lib/stores/MediaStateStore";
	import type { BoardContext, MediaPlayerContext } from "$lib/types";
	import { type MediaState, type WebsocketSessionEvent, type NumberScope, type Media, type MediaType, type VideoType, type MediaStatus, type ActionState, type MediaEvent, type VideoEvent} from "cult-common";
	import { getContext, onMount, setContext } from "svelte";
	import JeopardyBoard from "../JeopardyBoard.svelte";
	import { JeopardyBoardStore } from "$lib/stores/JeopardyBoardStore";
	import JeopardyCategory from "../JeopardyCategory.svelte";
	import { match, P } from "ts-pattern";
	import type { EventHandler } from "svelte/elements";
	import { WebsocketStore } from "$lib/stores/WebsocketStore";
	import type { Action } from "@sveltejs/kit";

    export let video: Blob
    export let currUserIsAdmin: boolean = false;
    export let videoTypes: VideoType[] = [];
    let player: HTMLVideoElement | null = null;
    let ranges : NumberScope[] = [];
    let muted = false;
    let play_rate = 1;

    let ov : HTMLElement | null = null;
    onMount(() => {
        ov = document.getElementById("ov") as HTMLElement;
        player = document.getElementById("player") as HTMLVideoElement;
        if (player == null) return;

        for (let type of videoTypes) {
            match(type)
            .with({ TimeSlots: P.select() }, (data) => {
                ranges = data;
            })
            .with("Mute", () => {
                if (player) player.muted = true;
                muted = true;
            })
            .with({ Slowmotion: P.select() }, (data) => {
                play_rate = data / 100;
                if (player) player.playbackRate = play_rate;
            })
        }
        ignore = false;
        let status = $mediaStateStore.media_status;
        if(status != null){
            doMediaStateChange(status);
        }

    })
    let ignore = true
    let mediasession: MediaPlayerSessionType;
    mediaStateStore.subscribe(value => {
        mediasession = value;
        if(value == null || value.media_status == null || ignore) {
            return;
        }
        doMediaStateChange(value.media_status);
    })


    enum EventStatusEnum {
        NONE = "NONE",
        PLAY = "PLAY",
        PAUSE = "PAUSE",
        START_SEEKING = "START_SEEKING",//MAYBE
        START_SEEKING_PLAY = "START_SEEKING_PLAY",//MAYBE
        PAUSE_START_SEEKING = "PAUSE_START_SEEKING",//MAYBE
        SEEKING = "SEEKING",
        SEEKING_PLAY = "SEEKING_PLAY",
    }


    let status = EventStatusEnum.NONE;
    let eventSourceLog = Array<EventStatusEnum>();


    let store = $WebsocketStore;
    

    enum StateUpdateType {
        PLAY = "PLAY",
        PAUSE = "PAUSE",
        SEEK = "SEEK",
        USER = "USER",
        WEBSOCKET = "WEBSOCKET",
        STARTING_SEEKING = "START_SEEKING",
        STOPING_SEEKING = "STOP_SEEKING",
        WEbSOCKET_SEEKING = "WEBSOCKET_SEEKING",
        
    }

  


    function handleEvent(state: MediaStatus): boolean {
        if (player == null) return false;
        if (state.interaction_id.id === store.websocket_id.id) {
            let proposed_time;
            let isSeeking;
            if (state.playing){
                let proposed_time = getProposedTime(state);
                isSeeking = Math.abs(proposed_time - player.currentTime) > CONST.PLAYING_THRESH;
            } else {
                proposed_time =  state.video_timestamp; // Video timestamp from the video itself
                isSeeking = (Math.abs(player.currentTime - proposed_time) > CONST.PAUSED_THRESH);
            }
            if(isSeeking){
                console.log("SEEKING", state);
                return false;
            }

            return false;
        }
        return true;
    }


    async function doMediaStateChange(state: MediaStatus) {
        try {
            if (!player) return;
                if (!handleEvent(state)) return;
                if (state.playing) {
                    determinePlayAndSeekPlay(state);
                } else {
                    determinePauseAndPauseSeek(state);
                }
        } catch (error) {
            console.error('Error changing media state:', error);
        }
    }
    


    async function pause() {
        if (!player)
            return;
        if (status == EventStatusEnum.PAUSE || status == EventStatusEnum.PAUSE_START_SEEKING) {
            update_status(StateUpdateType.PAUSE)
            return;
        }
        if(status != EventStatusEnum.NONE){
            return;
        }

        const state = currentMediaState(false);
        requestPlayerChangeState(state);
    }

    async function play(){
        console.log("playing", status);
        if (!player)
            return;
        if (player.playbackRate != play_rate)
            player.playbackRate = play_rate;

        if (status == EventStatusEnum.PLAY) {
            update_status(StateUpdateType.PLAY)
            return;
        }
        if(status != EventStatusEnum.NONE){
            return;
        }
        
        const state = currentMediaState(true);
        requestPlayerChangeState(state);
    }

    async function seek() {
        console.log("START SEEKING", status);
        if (!player)
            return;
        if(player.paused){
            update_status(StateUpdateType.SEEK);
            return;
        }


        if (status == EventStatusEnum.START_SEEKING || status == EventStatusEnum.START_SEEKING_PLAY) {
            update_status(StateUpdateType.STARTING_SEEKING);
            return;
        }
        if (status == EventStatusEnum.SEEKING || status == EventStatusEnum.SEEKING_PLAY) {
            return;
        }
        if (status != EventStatusEnum.NONE) {
            return;
        }

    }


    const validStatuses = {
        [EventStatusEnum.PLAY]: EventStatusEnum.NONE,
        [EventStatusEnum.PAUSE]: EventStatusEnum.NONE,
        [EventStatusEnum.SEEKING]: EventStatusEnum.NONE,
        [EventStatusEnum.PAUSE_START_SEEKING]: EventStatusEnum.START_SEEKING,
        [EventStatusEnum.START_SEEKING_PLAY]: EventStatusEnum.SEEKING_PLAY,
        [EventStatusEnum.START_SEEKING]: EventStatusEnum.SEEKING,
        [EventStatusEnum.SEEKING_PLAY]: EventStatusEnum.PLAY,
        [EventStatusEnum.NONE]: EventStatusEnum.NONE,
    };



    async function update_status(updae: StateUpdateType) {
        let old_status = status;
        let new_status = get_new_status(old_status);
        set_status(new_status,updae);
    }

    function get_new_status(old_status:EventStatusEnum): EventStatusEnum {
        let test =  validStatuses[old_status] || EventStatusEnum.NONE;
        return test
    }


    function set_status(new_status: EventStatusEnum, update: StateUpdateType) {
        console.log("UPDATE STATUS FROM", status, "TO", new_status , "WITH", update);
        status = new_status;
        eventSourceLog.push(new_status);
    }   
        



    function onEnded(event: any) {
        if (!player)
            return;
        try {
            player.load();
        } catch (error) {
            console.error('Error resetting video:', error);
        }
        const state = currentMediaState(false);
        requestPlayerChangeState(state);
    }



    async function seekToTime(time: number): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            if (!player) return;
            const timer = setTimeout(() => {
                if (!player) return;
                player.removeEventListener('seeked', onSeeked);
                reject(new Error('Seek operation timed out after 5 seconds'));
            }, 2000);
            function onSeeked() {
                if (!player) return;
                clearTimeout(timer);
                player.removeEventListener('seeked', onSeeked);
                resolve();
            }

            let delta = Math.abs(player.currentTime - time);


            if (player.paused && delta < CONST.PAUSED_THRESH) {
                resolve();
                console.log("SEEKING SKIP PAUSED_THRESH", delta);
                return;
            } else if (!player.paused && delta < CONST.PLAYING_THRESH) {
                resolve();
                console.log("SEEKING SKIP PLAYING_THRESH", delta);
                return;
            }



            player.addEventListener('seeked', onSeeked);
            player.currentTime = time;
            console.log("SEEKING TO", time);
        });
    }


    
    async function determinePlayAndSeekPlay(mediaState: MediaStatus) {
        if (!player) return;
        console.log("Playrate", play_rate);
        console.log("Test", ((get_global_time(mediasession.correction) - mediaState.global_timestamp) / 1000 +  mediaState.video_timestamp))


        let proposed_time = getProposedTime(mediaState);
        console.log("PROPOSED TIME", proposed_time);
        let isSeeking = Math.abs(proposed_time - player.currentTime) > CONST.PLAYING_THRESH;



        let isPlaying = !player.paused;
        if(status != EventStatusEnum.SEEKING){
            if(isSeeking ){
                set_status(isPlaying ? EventStatusEnum.START_SEEKING : EventStatusEnum.START_SEEKING_PLAY, StateUpdateType.WEbSOCKET_SEEKING); // SEEKING# SEEKING_PLAY
                await seekToTime(proposed_time);
            } else {
                set_status(isPlaying ? EventStatusEnum.NONE : EventStatusEnum.PLAY,StateUpdateType.WEBSOCKET); // PLAY# NONE
            }
        }   
        if (!isPlaying) player.play()

    }
    function getProposedTime(mediaState: MediaStatus): number {
        const play_rate = getPlayRate();
        
        let proposed_time = ((get_global_time(mediasession.correction) -  mediaState.global_timestamp) / 1000 +  mediaState.video_timestamp);


        return proposed_time;
    }
    function getPlayRate(): number {
        return play_rate || 1;
    }

    async function determinePauseAndPauseSeek(mediaState: MediaStatus) {
        if (!player) return;    
        let isPause = player.paused;
        let proposed_time =   mediaState.video_timestamp; // Video timestamp from the video itself
        let isSeeking = (Math.abs(player.currentTime - proposed_time) > CONST.PAUSED_THRESH);  
        if(status != EventStatusEnum.SEEKING){
            if (isSeeking) {
                set_status(isPause ? EventStatusEnum.START_SEEKING : EventStatusEnum.PAUSE_START_SEEKING, StateUpdateType.WEbSOCKET_SEEKING); // SEEKING# PAUSE_SEEKING
            } else {
                set_status(isPause ? EventStatusEnum.NONE : EventStatusEnum.PAUSE,StateUpdateType.WEBSOCKET); // NONE# PAUSE
            }
        }
        if (!isPause) player.pause()
        if (isSeeking) await seekToTime(proposed_time);
        
    }



    
    function currentMediaState(playing:boolean):MediaStatus {
        return {
                video_timestamp: player?.currentTime || 0,
                last_updated: get_global_time(mediasession.correction),
                global_timestamp: get_global_time(mediasession.correction),
                playing,
                interaction_id: {
                    id: store.websocket_id.id
            }
        }   
    }


    function endSeeking() {
        console.log("SEEKED");
        if (!player)
            return;
        if (status == EventStatusEnum.SEEKING || status == EventStatusEnum.SEEKING_PLAY || status == EventStatusEnum.START_SEEKING) {
            update_status(StateUpdateType.STOPING_SEEKING);
            return;
        }


        let playing = !player.paused;
        const state = currentMediaState(playing);
        requestPlayerChangeState(state);



    }


    function requestPlayerChangeState(state: MediaStatus): boolean {
        let changeState : VideoEvent = {  ChangeState:  state };
        let videoEvent : MediaEvent = {  VideoEvent:  changeState };
        let changeStateEvent: WebsocketSessionEvent = { MediaEvent:videoEvent } ;
        store.webSocketSubject.next(changeStateEvent);
        return true;
    }

    function containsTime(time: number): boolean {
        if (ranges.length == 0) return true;
        return ranges.some((range) => range.start <= time && time <= range.end);
    }
    
    function closesTimeStart(time: number): number {
    if (ranges.length === 0) return 0;

    return ranges.reduce((prev, curr) => {
        const prevDistance = Math.abs(prev.start - time);
        const currDistance = Math.abs(curr.start - time);
        return currDistance < prevDistance ? curr : prev;
    }).start; 
    }

    function closesTimeEnd(time: number): number {
    if (player == null) return 0;
    if (ranges.length === 0) return player.duration;

    return ranges.reduce((prev, curr) => {
        const prevDistance = Math.abs(prev.start - time);
        const currDistance = Math.abs(curr.start - time);
        return currDistance < prevDistance ? curr : prev;
    }).end; 
    }

    function closesTimeRange(time: number): NumberScope {
    if (player == null) return {start: 0, end: 0};
    if (ranges.length === 0) return {start: 0, end: player.duration};
    return ranges.reduce((prev, curr) => {
        const prevDistance = Math.abs(prev.start - time);
        const currDistance = Math.abs(curr.start - time);
        return currDistance < prevDistance ? curr : prev;
        });
    }


    function moveTime(time: number): void {
        if (!player) return;
        if (!player.paused) {
            player.pause();
        }
        player.currentTime = time;
    }


</script>

<video 
bind:this={player}
on:play={play} on:pause={pause} on:ended={onEnded} 
on:seeking={seek}
on:seeked={endSeeking}
on:timeupdate={async () => {
    if (!player) return;
    if (!containsTime(player.currentTime)) {
        /*if(player.paused){
            status = EventStatusEnum.START_SEEKING;
        } else {        
            status = EventStatusEnum.PAUSE_START_SEEKING;
            player.pause();
        }*/
        if (!player.paused) {
            player.pause();
        }
        console.log("SEEKING TO closesTimeStart" , closesTimeStart(player.currentTime));
        await seekToTime(closesTimeStart(player.currentTime));
        }
    }
}
on:volumechange={() => {
    if (!player) return;
    if (muted) {
        player.muted = true;
    }
}}



id="player" src={URL.createObjectURL(video)} controls={currUserIsAdmin} muted style="width: 640px; height: 360px;" disablepictureinpicture controlslist="nodownload noplaybackrate">
    <track kind="captions" />
</video>



{#if currUserIsAdmin && player != null && ov != null}
    {#if  ranges.length > 0}
        <div class="fixed flex flex-col gap-2 left-0 top-1/2 transform -translate-y-1/2 p-4 bg-cultGrey text-white shadow-lg rounded-lg" id="ov" bind:this={ov}>
            {#each ranges as range, i}
                <button class="cult-btn-menu" on:click={(e) => {moveTime(range.start)}}>
                    [Range{i + 1}]:[{range.start} - {range.end}]
                </button>
            {/each}
        </div>
    {/if}
{/if}

