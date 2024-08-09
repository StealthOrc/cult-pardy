<script lang="ts">
    import { handleFileUpload, upload_file2  } from './fileUploadUtils';
    import type { FileUploadProgress } from './fileUploadUtils';

    let file: File | null = null;
    let progress: number = 0;
    let uploadStatus: string = '';
    let progressBar: HTMLProgressElement;
    let progressText: HTMLSpanElement;
    let isUploading: boolean = false;

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
                const onProgress = (progress: FileUploadProgress) => {
                    progressBar.value = progress.loaded;
                    progressBar.max = progress.total;
                    progressText.innerText = `${progress.loaded}/${progress.total}`;
                };

                


                await handleFileUpload(file, onProgress);

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

<div class="flex flex-col items-center p-6 bg-gray-100 rounded-lg shadow-lg">
    <h1 class="text-2xl font-semibold mb-4 text-gray-800">File Upload</h1>
    <input type="file"   accept="image/*, video/*"   on:change={handleFileChange}  class="mb-4 p-2 border border-gray-300 rounded-lg bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 transition duration-200"/>
    <button  on:click={uploadFile}   class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition duration-200" disabled={isUploading}  >
        Upload
    </button>
    <progress  class="w-full max-w-md mt-4 h-2 bg-gray-200 rounded-full"  max="100"  bind:this={progressBar}  aria-label="Upload Progress"  ></progress>
    <span bind:this={progressText} class="mt-2 text-sm text-gray-600">
        0/0
    </span>
    <div class="mt-4 text-lg font-medium text-gray-700">{uploadStatus}</div>
</div>
