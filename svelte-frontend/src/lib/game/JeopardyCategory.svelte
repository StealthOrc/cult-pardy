<!-- JeopardyCategory.svelte -->
<script lang="ts">
	import type { DtoCategory, Vector2D }  from 'cult-common';
    import JeopardyQuestion from './JeopardyQuestion.svelte';
	import type { WebSocketSubject } from 'rxjs/webSocket';

    export let category: DtoCategory;
    export let ws: WebSocketSubject<any> | null;
    export let currentQuestion : Vector2D | null;
</script>

<div class="jeopardy-category">
    <h2>{category.title}</h2>
    {#each category.questions as question: Question}
        
        <JeopardyQuestion {question} {ws} {currentQuestion}/>
    {/each}
</div>

<style>
    .jeopardy-category {
        text-align: center;
    }
</style>