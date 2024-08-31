<script lang="ts">
	import type { DtoCategory, DTOSession }  from 'cult-common';
    import JeopardyQuestion from './JeopardyQuestion.svelte';

    export let category: DtoCategory;
</script>

<div class="flex flex-col items-center">
    <div class="flex flex-1 w-24 justify-center items-center">
        <p class="font-semibold my-1 text-white text-center text-pretty">{category.title}</p>
    </div>
    {#each category.questions as question: Question}
        <JeopardyQuestion {question}/>
    {/each}
</div>
