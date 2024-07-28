<!-- JeopardyCategory.svelte -->
<script lang="ts">
	import type { DtoCategory }  from 'cult-common';
    import JeopardyQuestion from './JeopardyQuestion.svelte';

    export let category: DtoCategory;
    console.log("cat + " + category);
</script>

<div class="jeopardy-category">
    <h2>{category.title}</h2>
    {#each category.questions as question: Question}
        
        <JeopardyQuestion {question} />
    {/each}
</div>

<style>
    .jeopardy-category {
        text-align: center;
    }
</style>