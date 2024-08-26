<script lang="ts">
    import type { JeopardyBoard } from 'cult-common';
    import CategoryCreator from './CategoryCreator.svelte';
    import { writable } from 'svelte/store';
	import { JeopardyBoardCreatorStore } from './BoardCreatorsStore';

    let board : JeopardyBoard | undefined = undefined;
    JeopardyBoardCreatorStore.subscribe(value => {
      console.log("TEST?", value); 
      board = value;
    });

    function addCategory() {
      JeopardyBoardCreatorStore.addCategory("New Category");
    }
  </script>
  

  {#if board}
  <div class="p-4">
    <h1 class="text-2xl mb-4">{ board.title }</h1>
    <input type="text" placeholder="Enter Board Title" bind:value={board.title} class="border px-2 py-1 rounded w-full mb-4" />
    <button on:click={addCategory} class="bg-blue-500 text-white px-4 py-2 rounded">Add Category</button>
    <div class="flex flex-col gap-4 mt-4">
      {#each board.categories as category, index}
        <CategoryCreator category={category} {index} />
      {/each}
    </div>
    <pre class="mt-6 bg-gray-100 p-4 rounded">
      {JSON.stringify(board, null, 2)}
    </pre>
  </div>
  {/if}
  