import type { DtoJeopardyBoard, ActionState } from "cult-common"

export enum FileUploadType {
    BOARDJSON,
    MEDIA,
}

export enum VideoPlayerType {
    NONE,
    VIDEO,
    YOUTUBE,
}

export type BoardContext = {
  requestPlay: () => boolean;
  requestPause: (value: number) => boolean;
};

export type MediaPlayerContext = {
    play: () => void;
    pause: () => void;
}