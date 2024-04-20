<script>
    export let question;
    let showOverlay = false;

    function handleClick() {
        showOverlay = true;
    }

    function handleClose() {
        showOverlay = false;
    }
    function handleKeyDown(event) {
        if (event.key === 'Escape') {
            handleClose();
        }
    }

</script>

<div class="jeopardy-question">
    <button on:click={handleClick}>${question.value}</button>
    {#if showOverlay}
        <div class="overlay" on:click={handleClose} on:keydown={handleKeyDown}>
            <div class="overlay-content">
                <h1>${question.value}</h1>
                <p>{question.question_text} ?</p>
            </div>
        </div>
    {/if}
</div>


<style>
    .jeopardy-question {
        margin: 5px;
        position: relative;
    }

    .jeopardy-question button {
        width: 100px;
        height: 60px;
        font-size: 24px;
        background-color: #4CAF50;
        color: white;
        border: none;
        border-radius: 5px;
        cursor: pointer;
        transition: background-color 0.3s ease;
    }

    .jeopardy-question button:hover {
        background-color: #45a049;
    }

    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2;
    }

    .overlay-content {
        background-color: white;
        padding: 20px;
        border-radius: 10px;
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
        max-width: 80%;
        max-height: 80%;
        overflow-y: auto;
    }
</style>