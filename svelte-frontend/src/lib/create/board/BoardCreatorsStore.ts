import { dev } from "$app/environment";
import type { JeopardyBoard, Question } from "cult-common";

import { writable } from "svelte/store";

export const JeopardyBoardCreatorStore = createJeopardyBoardCreatorStore();



if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.JeopardyBoardCreatorStore.store = JeopardyBoardCreatorStore.store;
            }
        });
    }
}


function getDefaultBoard() : JeopardyBoard {
    return{
        
        title: "Jeopardy Game",
        categories: [
          { title: "Category 1", questions: [] },
          { title: "Category 2", questions: [] }
        ]
    }
}





function createJeopardyBoardCreatorStore() {

    const store = writable<JeopardyBoard>(getDefaultBoard());


    function setBoard(board: JeopardyBoard) {
        store.set(board);        
    }

    function setTitle(title: string) {
        store.update((board) => {
            board.title = title;
            return board;
        });       
    }

    function addCategory(title: string) {
        store.update((board) => {
            board.categories.push({ title, questions: [] });
            return board;
        });       
    }

    function removeCategory(index: number) {
        store.update((board) => {
            board.categories.splice(index, 1);
            return board;
        });       
    }

    function setCategoryTitle(index: number, title: string) {
        store.update((board) => {
            board.categories[index].title = title;
            return board;
        });       
    }

    function addQuestion(categoryIndex: number, question: Question) {
        store.update((board) => {
            board.categories[categoryIndex].questions.push(question);
            return board;
        });       
    }

    function removeQuestion(categoryIndex: number, questionIndex: number) {
        store.update((board) => {
            board.categories[categoryIndex].questions.splice(questionIndex, 1);
            return board;
        });       
    }

    function setQuestion(categoryIndex: number, questionIndex: number, question: Question) {
        store.update((board) => {
            board.categories[categoryIndex].questions[questionIndex] = question
            return board;
        });       
    }



    function subscribe(this: void, run: (value: JeopardyBoard) => void) {
        return store.subscribe(run);
    }

    return {
        store,
        setBoard,
        setTitle,
        addCategory,
        removeCategory,
        setCategoryTitle,
        addQuestion,
        removeQuestion,
        setQuestion,
        subscribe

    }

}