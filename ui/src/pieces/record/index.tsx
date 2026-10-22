// The Service Record: rank, RR, recent ranked games and the act's numbers on one piece, drawn
// the way each style's own game draws its career screen (Halo 3's Service Record, the CS 1.6
// Statistics dialog, a terminal printout, Minecraft's Statistics, a Fortnite season page, the
// VALORANT beta's Progression, the 2022 career page). Auto picks the style's own.
import { Match, Show, Switch, createMemo } from "solid-js";
import type { PieceProps } from "~/studio/registry";
import { doc } from "~/studio/store";
import type { Skin } from "~/studio/types";
import { careerView } from "~/state/views";
import { RankCard, RrHistory, Status, useCareer } from "../career";
import { recordStats } from "./shared";
import { HaloRecord } from "./halo";
import { Cs16Record } from "./cs16";
import { TerminalRecord } from "./terminal";
import { MinecraftRecord } from "./minecraft";
import { FortniteRecord } from "./fortnite";
import { BetaRecord } from "./beta";
import { ValnowRecord } from "./valnow";
import styles from "./record.module.css";

export type RecordLayout = "modern" | "halo" | "cs16" | "terminal" | "minecraft" | "fortnite" | "beta" | "valnow";

/** The record's look when set to Auto: what the style's own game would show. */
const AUTO_RECORD: Record<Skin, RecordLayout> = {
  modern: "modern",
  beta: "beta",
  current: "valnow",
  glass: "modern",
  valnow: "valnow",
  csgo: "modern",
  cs2: "modern",
  cs16: "cs16",
  terminal: "terminal",
  fortnite: "fortnite",
  minecraft: "minecraft",
  halo: "halo",
  pipboy: "halo",
  league: "modern",
};

export const RECORD_LAYOUTS: readonly { value: string; label: string }[] = [
  { value: "auto", label: "Auto" },
  { value: "modern", label: "Rank card + history" },
  { value: "halo", label: "Halo 3 Service Record" },
  { value: "cs16", label: "CS 1.6 Statistics" },
  { value: "terminal", label: "Terminal printout" },
  { value: "minecraft", label: "Minecraft Statistics" },
  { value: "fortnite", label: "Fortnite career" },
  { value: "beta", label: "Beta Progression" },
  { value: "valnow", label: "VALORANT 2022 career" },
];

export function ServiceRecord(props: PieceProps) {
  useCareer();
  const layout = (): RecordLayout => {
    const chosen = props.piece.props["layout"];
    return typeof chosen === "string" && chosen !== "auto" ? (chosen as RecordLayout) : AUTO_RECORD[doc.theme.skin];
  };
  return (
    <Switch>
      <Match when={layout() === "modern"}>
        <div class={styles.modern}>
          <div class={styles.modernRank}>
            <RankCard {...props} />
          </div>
          <div class={styles.modernHistory}>
            <RrHistory {...props} />
          </div>
        </div>
      </Match>
      <Match when={true}>
        <Show when={careerView.value()} fallback={<Status what="Service record" />}>
          {(view) => {
            const stats = createMemo(() => recordStats(view()));
            return (
              <Switch>
                <Match when={layout() === "halo"}>
                  <HaloRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "cs16"}>
                  <Cs16Record view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "terminal"}>
                  <TerminalRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "minecraft"}>
                  <MinecraftRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "fortnite"}>
                  <FortniteRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "beta"}>
                  <BetaRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
                <Match when={layout() === "valnow"}>
                  <ValnowRecord view={view()} stats={stats()} isEditing={props.isEditing} />
                </Match>
              </Switch>
            );
          }}
        </Show>
      </Match>
    </Switch>
  );
}
