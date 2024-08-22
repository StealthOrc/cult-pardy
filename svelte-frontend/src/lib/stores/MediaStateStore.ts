import { dev } from "$app/environment";
import { get_global_time } from "$lib/lib";
import type { MediaState } from "cult-common";
import { writable, type Subscriber, type Unsubscriber } from "svelte/store";



export const mediaStateStore = createMediaStateStore();

export type MediaPlayerSessionType = {
    mediaState: MediaState | null,
    over_estimates :number[],
    under_estimates :number[],
    over_estimate: number,
    under_estimate: number,
    correction: number,
}



if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.mediaStateStore.store = mediaStateStore.store;
            }
        });
    }
}


function createMediaStateStore() {

    const store = writable<MediaPlayerSessionType>({
        mediaState: null,
        over_estimates: [],
        under_estimates: [],
        over_estimate: 0,
        under_estimate: 0,
        correction: 0,
    });

    function subscribeMediaState(this: void, run: Subscriber<MediaState>): Unsubscriber {
        return store.subscribe((data) => {
            if (data.mediaState != null) {
                run(data.mediaState);
            }
        });
    }

    function setMediaState(mediaState: MediaState) {
        store.update((curr) => {
            curr.mediaState = mediaState;
            return curr;
        });
    }






    function addForward(time: number) {
        store.update((curr) => {
            console.log("addForward");
            curr.over_estimates.push(time);
            curr.over_estimate = median(curr.over_estimates);
            curr.correction = (curr.under_estimate +curr.over_estimate) / 2;
            console.log("over_estimates", curr.over_estimates)
            console.log("over_estimate", curr.over_estimate)
            console.log("under_estimate", curr.under_estimate)
            console.log(`%c Updated val for over_estimate is ${curr.over_estimate}`, "color:green");
            console.log(`%c New correction time is ${curr.correction} miliseconds`, 'color:red; font-size:12px');
            return curr
        });
    }

    function addBackward(time: number) {
        store.update((curr) => {
            console.log("addBackward");
            curr.under_estimates.push(time - get_global_time(0));
            curr.under_estimate = median(curr.under_estimates);
            curr.correction = (curr.under_estimate + curr.over_estimate) / 2;
            console.log("under_estimates", curr.under_estimates)
            console.log("under_estimate", curr.under_estimate)
            console.log("over_estimate", curr.over_estimate)
            console.log(`%c Updated val for under_estimate is ${curr.under_estimate}`, "color:green");
            console.log(`%c New correction time is ${curr.correction} miliseconds`, 'color:red; font-size:12px');
            return curr
        });
    }



    function clearEstimates() {
        store.update((curr) => {
            curr.over_estimates = [];
            curr.under_estimates = [];
            return curr;
        });
    }

    function clearOverEstimates() {
        store.update((curr) => {
            curr.over_estimates = [];
            return curr;
        });
    }

    function clearUnderEstimates() {
        store.update((curr) => {
            curr.under_estimates = [];
            return curr;
        });
    }




    function subscribe(this: void, run: Subscriber<MediaPlayerSessionType>): Unsubscriber {
        return store.subscribe(run);
    }

    

    function median(values: number[]): number {
        if (values.length === 0) {
            return 0;
        }
        values.sort((x, y) => x - y);
        const half = Math.floor(values.length / 2);
    
        if (values.length % 2 !== 0) {
            return values[half];
        }
        return (values[half - 1] + values[half]) / 2.0;
    }
    

    return {
        store,
        setMediaState,
        subscribe,
        subscribeMediaState,
        addForward,
        addBackward,
        clearEstimates,
        clearOverEstimates,
        clearUnderEstimates,
            
    }
}
