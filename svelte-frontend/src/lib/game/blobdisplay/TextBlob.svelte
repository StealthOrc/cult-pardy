<script lang="ts">
    export let text: Blob;
    let textContent: string = '';

    // Function to read the blob and set its content
    const readTextBlob = (blob: Blob) => {
        const reader = new FileReader();
        reader.onload = () => {
            if (typeof reader.result === 'string') {
                textContent = reader.result;
            }
        };
        reader.readAsText(blob);
    };

    // Read the textBlob when the component mounts or textBlob changes
    readTextBlob(text);
</script>

<div class="flex flex-col items-center p-6 bg-gray-100 rounded-lg shadow-lg">
    <pre class="whitespace-pre-wrap text-left w-full max-w-lg bg-white p-4 rounded-lg border border-gray-300">
        {textContent || 'No text available'}
    </pre>
</div>
