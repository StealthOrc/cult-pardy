<!-- JeopardyCategory.svelte -->
<script lang="ts">
    import JeopardyQuestion from './JeopardyQuestion.svelte';

    export let category: Category;
    console.log("cat + " + category);
</script>

<div class="jeopardy-category">
    <h2>{category.title}</h2>
    {console.log("category: " + category)}
    {#each category.questions as question: Question}
        
        <JeopardyQuestion {question} />
    {/each}
</div>

<style>
    .jeopardy-category {
        text-align: center;
    }
</style>