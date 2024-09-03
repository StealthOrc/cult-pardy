import { dev } from "$app/environment";
import { get_global_time } from "$lib/lib";
import type { MediaStatus } from "cult-common";
import { writable, type Subscriber, type Unsubscriber } from "svelte/store";

export const mediaStateStore = createMediaStateStore();

export type MediaPlayerSessionType = {
    media_status: MediaStatus | null,
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
        media_status: null,
        over_estimates: [],
        under_estimates: [],
        over_estimate: 0,
        under_estimate: 0,
        correction: 0,
    });

    function subscribeMediaStatus(this: void, run: Subscriber<MediaStatus>): Unsubscriber {
        return store.subscribe((data) => {
            if (data.media_status != null) {
                run(data.media_status);
            }
        });
    }

    function resetMedia() {
        store.update((curr) => {
            curr.media_status = null;
            return curr;
        });
    }

    function setMediaStatus(media_status: MediaStatus) {
        store.update((curr) => {
            curr.media_status = media_status;
            return curr;
        });
    }






    function addForward(time: number) {
        store.update((curr) => {
            curr.over_estimates.push(time);
            curr.over_estimate = median(curr.over_estimates);
            curr.correction = (curr.under_estimate +curr.over_estimate) / 2;
            return curr
        });
    }

    function addBackward(time: number) {
        store.update((curr) => {
            curr.under_estimates.push(time - get_global_time(0));
            curr.under_estimate = median(curr.under_estimates);
            curr.correction = (curr.under_estimate + curr.over_estimate) / 2;
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
        setMediaStatus,
        subscribe,
        resetMedia,
        subscribeMediaState: subscribeMediaStatus,
        addForward,
        addBackward,
        clearEstimates,
        clearOverEstimates,
        clearUnderEstimates,
            
    }
}
