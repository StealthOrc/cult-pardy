import type { MediaPlayerContext } from "$lib/types";
import { writable } from "svelte/store";

export const mediaPlayerContextStore = writable<MediaPlayerContext | null>(null)
