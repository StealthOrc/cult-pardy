<script lang="ts">
	import type { DtoQuestion, Media, MediaType, NumberScope, WebsocketSessionEvent } from 'cult-common';
	import { onMount } from 'svelte';
	import { BlobType, downloadBlob, getBlobType, test, test2, type FileDownloadProgress } from './blodUtils';
	import ImageBlob from './ImageBlob.svelte';
	import VideoBlob from './VideoBlob.svelte';
	import AudioBlob from './AudioBlob.svelte';
	import TextBlob from './TextBlob.svelte';
	import { match, P } from 'ts-pattern';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { CookieStore, lobby_store } from '$lib/stores/cookies';
	import { WebsocketSessionStore } from '$lib/stores/WebsocketSessionStore';
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
	export let media: Media;

	let blob: Blob | null = null;
	let fileDownloadProgress: FileDownloadProgress | null = null;
	let blobType: BlobType = BlobType.UNKNOWN;

	const onProgress = (progress: FileDownloadProgress) => {
        if (blob != undefined)  return;
		if (progress.blob) {
            blobType = getBlobType(progress.blob);
			blob = progress.blob;
			$WebsocketStore.webSocketSubject.next("MediaDownloadComplete");

		} else {
			fileDownloadProgress = progress;
		}
	};

	async function loadBlob() {
		if (!blob) {
			await downloadBlob(media.name, $lobby_store, media.media_token,onProgress);	
		}
		if (typeof media.media_type === "object" && "Video" in media.media_type) {
        	return media.media_type.Video; // Access the VideoType
    	}
	}

    function isAdmin(): boolean {
        return CurrentSessionsStore
            .getSessionById({ id: $CookieStore.userSessionId.id})
            .is_admin;
    }
	onMount(loadBlob);


	function allSessionLoaded() : boolean {
		console.log("media", !media.media_loaded);
		console.log("media", media.media_loaded.length);
		console.log("media", $CurrentSessionsStore.size);


		
		if (!media.media_loaded) {
        	return false;
    	}
		if (media.media_loaded.length === 0) {
			return false;
		}

		for (let session of $CurrentSessionsStore.values()) {
			console.log("session", session);
			console.log("session", session.user_session_id);
			console.log("session", media.media_loaded);
			console.log("session", media.media_loaded.includes(session.user_session_id));
			let found = media.media_loaded.find((element) => element.id === session.user_session_id.id);
			if (!found) {
				return false;
			}
		}
		return true;
	}

</script>




{#if blob && typeof media.media_type === "object" && allSessionLoaded()}
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
