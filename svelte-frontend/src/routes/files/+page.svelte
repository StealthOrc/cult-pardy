<script lang="ts">
    import { api_file_list, api_files } from '$lib/api/ApiRequests';
    import { getFileType } from '$lib/const';
	import BlobDisplay from '$lib/game/blobdisplay/BlobDisplay.svelte';
    import type { DTOFileData, Media } from 'cult-common';
    import { onMount } from 'svelte';

    let fileCount = 0;
    let pageSize = 5; // Number of files per page
    let currentPage = 0; // Start from page 0
    let totalPages = 0;
    let fileList: DTOFileData[] = [];

	let media : Media | undefined = undefined;

    // Fetch the total file count from the API
    async function fetchFileCount() {
        try {
            const response = await api_files();
            if (response.ok) {
                const data = await response.json();
                fileCount = data.file_count;
                totalPages = Math.ceil(fileCount / pageSize) - 1; // Adjust for 0-based indexing
            } else {
                console.error("Error fetching file count.");
            }
        } catch (error) {
            console.error("Error fetching file count: ", error);
        }
    }

    // Fetch the file list for the current page from the API
    async function fetchFileList(page: number) {
        try {
            const response = await api_file_list(pageSize, page);
            if (response.ok) {
                const data = await response.json();
                fileList = data.files;
            } else {
                console.error("Error fetching file list.");
            }
        } catch (error) {
            console.error("Error fetching file list: ", error);
        }
    }

    // Navigate to a specific page
    function goToPage(page: number) {
        if (page < 0 || page > totalPages) return;
        currentPage = page;
        fetchFileList(currentPage);
    }

    onMount(() => {
        fetchFileCount();
        fetchFileList(currentPage);
    });

    // Convert and format the date
    function formatDate(dateString: string): string {
        const isoArray = dateString.split(' +');
        if (isoArray.length <= 1) {
            return 'Invalid Date';
        }
        const isoString = isoArray[0] + "Z";
        const date = new Date(isoString);

        // Check if the date is valid
        if (isNaN(date.getTime())) return 'Invalid Date';

        // Format the date to 'dd MMM yyyy, HH:mm:ss'
        const day = String(date.getUTCDate()).padStart(2, '0');
        const month = String(date.getUTCMonth() + 1).padStart(2, '0'); // Months are zero-based
        const year = date.getUTCFullYear();
        const hours = String(date.getUTCHours()).padStart(2, '0');
        const minutes = String(date.getUTCMinutes()).padStart(2, '0');
        const seconds = String(date.getUTCSeconds()).padStart(2, '0');

        return `${day}.${month}.${year}, ${hours}:${minutes}:${seconds}`;
    }

    // Handle view button click
    function viewFile(file: DTOFileData) {
		media = file.media;
    }
</script>

<!-- Pagination controls -->
<div class="flex items-center justify-between p-4 bg-gray-100 border-t border-gray-200">
	{#if !media}
    <button 
        on:click={() => goToPage(currentPage - 1)} 
        disabled={currentPage === 0} 
        class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-300"
    >
        Previous
    </button>
    <span class="text-gray-700">Page {currentPage + 1} of {totalPages + 1}</span>
    <button 
        on:click={() => goToPage(currentPage + 1)} 
        disabled={currentPage === totalPages} 
        class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-300"
    >
        Next
    </button>
	{:else}
	<button 
		on:click={() => media = undefined} 
		class="bg-blue-500 text-white px-4 py-2 rounded"
	>
		Back
	</button>
	{/if}
</div>

<!-- File list display -->
<div class="p-4">
	
	{#if media != undefined}
	<div class="fixed flex justify-center items-center left-0 w-full h-full z-10" role="dialog">
		<BlobDisplay media={media}/>
	</div>
	{:else}


    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {#each fileList as file}
            <div class="flex flex-col items-center bg-white border border-gray-200 rounded-lg shadow-md p-4">
                <img src={getFileType(file.file_name)} alt={file.file_name} class="w-16 h-16 mb-4" />
                <div class="text-center">
                    <p class="font-bold text-gray-900">Filename: <span class="font-normal">{file.file_name}</span></p>
                    <p class="text-gray-600">Uploaded on: <span class="font-normal">{formatDate(file.upload_date)}</span></p>
                    <p class="text-gray-600">File type: <span class="font-normal">{file.metadata.file_type}</span></p>
                    <p class="text-gray-600">Uploader: <span class="font-normal">{file.metadata.uploader.id}</span></p>
                    <button 
                        on:click={() => viewFile(file)} 
                        class="mt-2 bg-green-500 text-white px-4 py-2 rounded"
                    >
                        View
                    </button>
                </div>
            </div>
        {/each}
    </div>
	{/if}
</div>
