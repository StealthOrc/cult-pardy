<!-- JeopardyCategory.svelte -->
<script lang="ts">
	import type { DtoCategory, DtoQuestion, Vector2D }  from 'cult-common';
    import JeopardyQuestion from './JeopardyQuestion.svelte';
	import type { WebSocketSubject } from 'rxjs/webSocket';

    export let category: DtoCategory;
    export let ws: WebSocketSubject<any> | null;
    export let current : DtoQuestion | null;
</script>

<div class="jeopardy-category">
    <h2>{category.title}</h2>
    {#each category.questions as question: Question}
        
        <JeopardyQuestion {question} {ws} {current}/>
    {/each}
</div>

<style>
    .jeopardy-category {
        text-align: center;
    }
</style>