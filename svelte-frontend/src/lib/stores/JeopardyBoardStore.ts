
import { dev } from "$app/environment";
import type { DtoJeopardyBoard, DtoQuestion } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


export const JeopardyBoardStore = createJeopardyBoardStore();



if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.JeopardyBoardStore.store = JeopardyBoardStore.store;
            }
        });
    }
}



function createJeopardyBoardStore() {

    const store = writable<DtoJeopardyBoard|null>(null);


    function setBoard(board: DtoJeopardyBoard) {
        store.set(board);        
    }

    function setCurrent(current: DtoQuestion) {
        store.update((board) => {
            if (board == null) {
                return board;
            }
            board.current = current;
            return board;
        });       
    }


    function subscribe(this: void, run: Subscriber<DtoJeopardyBoard | null>): Unsubscriber {
        return store.subscribe(run);
    }

    

    return {
        store,
        setCurrent,
        setBoard,
        subscribe,
    }
}

