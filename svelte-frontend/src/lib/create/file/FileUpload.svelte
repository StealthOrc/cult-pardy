<script lang="ts">
    import { handleBoardFileUpload, handleMediaFileUpload  } from './fileUploadUtils';
    import type { FileUploadProgress } from './fileUploadUtils';
    import { FileUploadType } from '$lib/types';

    export let title: string = 'Upload File';
    export let uploadType: FileUploadType = FileUploadType.MEDIA;

    let file: File | null = null;
    let progress: number = 0;
    let uploadStatus: string = '';
    let progressBar: HTMLProgressElement;
    let progressText: HTMLSpanElement;
    let isUploading: boolean = false;
    let acceptTypes: string = "";
    switch (uploadType) {
        case FileUploadType.MEDIA:
            acceptTypes = "image/*,video/*";
            break;
        case FileUploadType.BOARDJSON:
            acceptTypes = ".json";
            break;
    }

    function handleFileChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files?.length) {
            file = input.files[0];
            uploadStatus = 'Ready to upload';
            isUploading = false; // Reset uploading state
        }
    }

    async function uploadFile() {
        if (file) {
            try {
                uploadStatus = 'Uploading...';
                isUploading = true; // Set uploading state to true
                let max = 100;
                const onProgress = (progress: FileUploadProgress) => {
                    progressBar.value = progress.current;
                    progressBar.max = max;
                    progressText.innerText = `${progress.current} % / ${max} % Speed ${progress.speed}`;
                };
                onProgress({ current: 0, speed: "N/A" });
                switch (uploadType) {
                    case FileUploadType.MEDIA:
                        await handleMediaFileUpload(file, onProgress);
                        break;
                    case FileUploadType.BOARDJSON:
                        await handleBoardFileUpload(file, onProgress);
                        break;
                }
                uploadStatus = 'Upload complete!';
            } catch (error) {
                uploadStatus = 'Upload failed!';
                console.error(error);
            } finally {
                isUploading = false; // Reset uploading state regardless of outcome
            }
        }
    }
</script>

<div class="flex flex-col items-center p-6 cult-surface">
    <h1 class="text-2xl font-semibold mb-4 text-white">{title}</h1>
    <input type="file" accept={acceptTypes} on:change={handleFileChange}  class="mb-4 p-2 border border-gray-300 rounded-lg bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 transition duration-200"/>
    <button  on:click={uploadFile}   class="px-6 py-3 cult-btn-menu" disabled={isUploading}  >
        Upload
    </button>
    <progress class="w-full max-w-md mt-4 h-2 bg-gray-200 rounded-full"  max="100"  bind:this={progressBar}  aria-label="Upload Progress"  ></progress>
    <span bind:this={progressText} class="mt-2 text-sm text-cultGrey-light">
        0/0
    </span>
    <div class="mt-4 text-lg font-medium text-gray-700">{uploadStatus}</div>
</div>
