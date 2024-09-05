<script lang="ts">
    import { api_file_list, api_files } from '$lib/api/ApiRequests';
	import { getFileType } from '$lib/const';
    import type { DTOFileData } from 'cult-common';
    import { onMount } from 'svelte';

    // Import the enum and function

    let fileCount = 0;
    let pageSize = 5; // Number of files per page
    let currentPage = 0; // Start from page 0
    let totalPages = 0;
    let fileList: DTOFileData[] = [];

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
</script>

<!-- Pagination controls -->
<div class="flex items-center justify-between p-4 bg-gray-100 border-t border-gray-200">
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
</div>

<!-- File list display -->
<ul class="p-4 space-y-4">
    {#each fileList as file}
        <li class="flex items-center p-4 bg-white border border-gray-200 rounded-lg shadow-md">
            <img src={getFileType(file.metadata.file_type)} alt={file.file_name} class="w-10 h-21 mr-4" />
            <div>
                <p class="font-bold text-gray-900">Filename: <span class="font-normal">{file.file_name}</span></p>
                <p class="text-gray-600">Uploaded on: <span class="font-normal">{file.upload_date}</span></p>
                <p class="text-gray-600">File type: <span class="font-normal">{file.metadata.file_type}</span></p>
                <p class="text-gray-600">Uploader: <span class="font-normal">{file.metadata.uploader.id}</span></p>
            </div>
        </li>
    {/each}
</ul>
