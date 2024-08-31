<script lang="ts">

	import type { Media } from 'cult-common';
	import { onMount } from 'svelte';
	import { BlobType, downloadBlob, getBlobType, type FileDownloadProgress } from './blodUtils';
	import ImageBlob from './ImageBlob.svelte';
	import VideoBlob from './VideoBlob.svelte';
	import AudioBlob from './AudioBlob.svelte';
	import TextBlob from './TextBlob.svelte';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { CookieStore, lobby_store } from '$lib/stores/cookies';

	export let media: Media;

	let blob: Blob | null = null;
	let fileDownloadProgress: FileDownloadProgress | null = null;
	let blobType: BlobType = BlobType.UNKNOWN;

	const onProgress = (progress: FileDownloadProgress) => {
        if (blob != undefined)  return;
		if (progress.blob) {
            blobType = getBlobType(progress.blob);
			blob = progress.blob;
		} else {
			fileDownloadProgress = progress;
		}
	};

	async function loadBlob() {
		if (!blob && media.media_token) {
		
			await downloadBlob(media.name, $lobby_store, media.media_token,onProgress);	
		}
		if (typeof media.media_type === "object" && "Video" in media.media_type) {
        return media.media_type.Video; // Access the VideoType
    	}
	}

    function isAdmin(): boolean {
        return $CurrentSessionsStore.filter(s => s.user_session_id.id === $CookieStore.userSessionId.id && s.is_admin).length > 0;
    }

	onMount(loadBlob);
</script>

{#if blob && typeof media.media_type === "object"}
	{#if blobType === BlobType.IMAGE}
		<ImageBlob image={blob} />
	{:else if blobType === BlobType.VIDEO && "Video" in media.media_type}
		<VideoBlob video={blob} videoTypes={media.media_type.Video} currUserIsAdmin = {isAdmin()} />
	{:else if blobType === BlobType.AUDIO}
		<AudioBlob audio={blob} />
	{:else if blobType === BlobType.TEXT}
		<TextBlob text={blob} />
	{:else}
		<p>Unsupported file type</p>
	{/if}
{:else}
	<p>Loading... {fileDownloadProgress?.current || 0} % /  {100} %  | Speed {fileDownloadProgress?.speed} </p>
{/if}
