// The whole menu: scene backdrop, floating top nav, the active scene (a Studio stage), the
// scene-change wipe, toasts, the Esc menu, and in edit mode the Studio panels. Match-flow
// events move between scenes (agent select when a match is found, home afterwards).
import { Show, createEffect, on, onCleanup, type ParentProps } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import { onAppEvent } from "~/ipc/events";
import { postMatch } from "~/state/postMatch";
import { pregame } from "~/state/pregame";
import { game } from "~/state/session";
import { pushToast } from "~/state/toasts";
import { addFromImageFile, addFromUrl, setCurrentStage } from "~/studio/actions";
import { Inspector } from "~/studio/Inspector";
import { StudioBar } from "~/studio/StudioBar";
import { useEditorShortcuts } from "~/studio/shortcuts";
import { doc, isEditing } from "~/studio/store";
import { FriendsDrawer } from "~/pieces/friends";
import { ClickFx } from "./ClickFx";
import { CommandPalette } from "./CommandPalette";
import { EscMenu } from "./EscMenu";
import { Intro } from "./Intro";
import { MatchFound } from "./MatchFound";
import { SceneBackdrop } from "./SceneBackdrop";
import { SceneWipe } from "./SceneWipe";
import { Toasts } from "./Toasts";
import { useWindowMode } from "./windowMode";
import { TopNav } from "./TopNav";
import { sceneForPath } from "./scenes";
import styles from "./MenuShell.module.css";

export function MenuShell(props: ParentProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const scene = () => sceneForPath(location.pathname);
  let sceneArea: HTMLElement | undefined;

  createEffect(() => setCurrentStage(scene()));
  // The style's cursor gives way to the Studio's move and resize cursors while editing.
  createEffect(() => document.documentElement.toggleAttribute("data-studio", isEditing()));
  onCleanup(() => document.documentElement.removeAttribute("data-studio"));
  useEditorShortcuts();
  useWindowMode();

  // Match flow decides the scene; a pregame view appearing is a backup signal.
  const stopFlow = onAppEvent("match_flow", (effect) => {
    if (effect.kind === "enter_scene") navigate(`/${effect.scene === "career" ? "home" : effect.scene}`);
  });
  onCleanup(stopFlow);
  createEffect(
    on(
      () => pregame() !== null,
      (isInPregame, wasInPregame) => {
        if (isInPregame && scene() !== "agent-select") navigate("/agent-select");
        // Leaving agent select into the match is handled below; back to the menus is a dodge.
        if (!isInPregame && wasInPregame && scene() === "agent-select" && game()?.phase !== "ingame") navigate("/home");
      },
    ),
  );

  // When the match starts the menu hides; whoever brings it back mid-match lands on Live.
  createEffect(
    on(
      () => game()?.phase,
      (phase, previous) => {
        if (phase === "ingame" && previous !== "ingame" && !isEditing()) navigate("/live");
        // Backup for the match flow's own return: the match is over, so the live overlay
        // must not stay up in the menus with nothing to press.
        if (phase === "menus" && previous === "ingame" && scene() === "live" && !isEditing()) navigate("/home");
      },
    ),
  );

  // A new match summary opens itself once, when it arrives while the player is in the menus.
  let shownMatchId = "";
  createEffect(
    on(
      () => postMatch(),
      (view) => {
        if (!view || view.is_seen || view.match_id === shownMatchId) return;
        if (game()?.phase !== "menus" || pregame() !== null || isEditing()) return;
        shownMatchId = view.match_id;
        navigate("/post-match");
      },
    ),
  );

  const dropPoint = (event: DragEvent) => {
    const bounds = sceneArea?.getBoundingClientRect();
    if (!bounds) return { x: 0.5, y: 0.5 };
    return { x: (event.clientX - bounds.left) / bounds.width, y: (event.clientY - bounds.top) / bounds.height };
  };

  const onDrop = (event: DragEvent) => {
    if (!isEditing()) return;
    event.preventDefault();
    const at = dropPoint(event);
    const file = event.dataTransfer?.files[0];
    if (file) {
      void addFromImageFile(scene(), file, at).then((isAdded) => {
        if (!isAdded) pushToast("info", "Not added", "Drop an image under 1.5 MB, or paste a link.");
      });
      return;
    }
    const url = event.dataTransfer?.getData("text/uri-list") || event.dataTransfer?.getData("text/plain") || "";
    if (!addFromUrl(scene(), url, at)) pushToast("info", "Not added", "Drop an image or a link.");
  };

  return (
    <div class={styles.shell} data-editing={isEditing() ? "" : undefined}>
      <SceneBackdrop scene={scene()} />
      <TopNav />
      <main
        ref={sceneArea}
        class={styles.scene}
        data-part="scene"
        onDragOver={(event) => isEditing() && event.preventDefault()}
        onDrop={onDrop}
      >
        {props.children}
      </main>
      <SceneWipe />
      <MatchFound />
      <Show when={isEditing()}>
        <Inspector />
        <StudioBar />
      </Show>
      {/* The VALORANT and League styles keep their friends in their own right-edge panels. */}
      <Show when={doc.theme.skin !== "current" && doc.theme.skin !== "league"}>
        <FriendsDrawer />
      </Show>
      <EscMenu />
      <CommandPalette />
      <Toasts />
      <ClickFx />
      <Intro />
    </div>
  );
}
