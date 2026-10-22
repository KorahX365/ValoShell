// Videos shipped with the installed game (home-screen loops first). Loaded once: the backend
// also opens their folder to the `asset:` protocol, so the urls only work after this call.
import { convertFileSrc } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import type { GameVideo } from "~/bindings/GameVideo";
import { call } from "~/ipc/commands";
import { isTauri } from "~/ipc/runtime";

const [gameVideos, setGameVideos] = createSignal<GameVideo[]>([]);
const [isSearched, setSearched] = createSignal(false);

void call("get_game_videos", {})
  .then(setGameVideos, () => undefined)
  .finally(() => setSearched(true));

export { gameVideos, isSearched as areGameVideosSearched };

/** The chosen video, or the newest home-screen loop when none is chosen (or it is gone). */
export function pickGameVideo(path: string): GameVideo | undefined {
  const videos = gameVideos();
  return videos.find((video) => video.path === path) ?? videos[0];
}

/** Playable url for a video file of the game (the demo backend lists plain urls). */
export function gameVideoUrl(video: GameVideo): string {
  return isTauri ? convertFileSrc(video.path) : video.path;
}
