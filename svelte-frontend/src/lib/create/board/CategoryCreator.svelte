<script lang="ts">
  import type { Category, Question, QuestionType } from 'cult-common';
  import QuestionCreator from './QuestionCreator.svelte';
  import type { Writable } from 'svelte/store';
  import JeopardyBoardCreator from './JeopardyBoardCreator.svelte';
  import { JeopardyBoardCreatorStore } from './BoardCreatorsStore';

  export let category: Category;
  export let index: number;

  function addQuestion() {
    let question : QuestionType = "Question";


    let newQuestion : Question = {
      question: "New Question",
      value: 100,
      answer: "New Answer",
      question_type: question,
    };
    JeopardyBoardCreatorStore.addQuestion(index, newQuestion);
  }

  function update(event: Event) {
    console.log(event);
    let input = event.target as HTMLInputElement;
    category.title = input.value;
    JeopardyBoardCreatorStore.setCategoryTitle(index,input.value);
  }


  
</script>

<div class="p-4 border rounded w-full">
  <h2 class="text-xl mb-2">Category {index + 1}: 
    <input type="text" on:change={update} bind:value={category.title} placeholder="Enter Category Title" class="border px-2 py-1 rounded w-full mb-2" />
  </h2>
  <button on:click={addQuestion} class="bg-green-500 text-white px-4 py-2 rounded">Add Question</button>
  <div class="mt-2">
    {#each category.questions as question, qIndex}
      <QuestionCreator {question} {index} {qIndex} />
    {/each}
  </div>
</div>
