<!-- JeopardyCategory.svelte -->
<script>
    import JeopardyQuestion from './JeopardyQuestion.svelte';

    export let category;
    console.log("cat + " + category);
    export let questions;
    console.log("questions + " + questions);
</script>

<div class="jeopardy-category">
    <h2>{category}</h2>
    {#each questions as question}
        <JeopardyQuestion {question} />
    {/each}
</div>

<style>
    .jeopardy-category {
        text-align: center;
    }
</style>