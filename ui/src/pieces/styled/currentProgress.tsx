// VALORANT (current client) Progress and mode-select pieces. Progress copies the client's
// page: daily checkpoints and weekly missions on the left, the battlepass reward up top and
// the objectives row below. Riot's contract numbers are not read here, so the figures are
// props the player sets (they default to a plausible mid-act account).
import { For, Index, Show, createSignal, onCleanup, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Lock } from "lucide-solid";
import type { PieceProps } from "~/studio/registry";
import { playCue } from "~/menu/sound";
import { catalog } from "~/state/catalog";
import { storeView } from "~/state/views";
import { Art } from "~/ui/Art";
import { usePlay } from "./usePlay";
import { colorVars, textProp } from "./current";
import { PassGlyph, KcCoin, VpCoin } from "./valIcons";
import { type TextKey, groupNumber, queueName, vt } from "./valText";
import styles from "./currentProgress.module.css";

function num(props: PieceProps, key: string, fallback: number): number {
  const value = props.piece.props[key];
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function str(props: PieceProps, key: string, fallback = ""): string {
  const value = props.piece.props[key];
  return typeof value === "string" && value.trim() ? value.trim() : fallback;
}

/** `12:27:05` / `5:13:41:40` until the next daily (00:00 UTC) or weekly (Tuesday) reset. */
function untilReset(now: number, isWeekly: boolean): string {
  const next = new Date(now);
  next.setUTCHours(24, 0, 0, 0);
  if (isWeekly) while (next.getUTCDay() !== 2) next.setUTCDate(next.getUTCDate() + 1);
  let seconds = Math.max(0, Math.floor((next.getTime() - now) / 1000));
  const days = Math.floor(seconds / 86_400);
  seconds -= days * 86_400;
  const pad = (value: number) => value.toString().padStart(2, "0");
  const clock = `${pad(Math.floor(seconds / 3600))}:${pad(Math.floor((seconds % 3600) / 60))}:${pad(seconds % 60)}`;
  return days > 0 ? `${days}:${clock}` : clock;
}

interface Weekly {
  text: TextKey;
  done: number;
  goal: number;
}

export function ValProgress(props: PieceProps) {
  const [now, setNow] = createSignal(Date.now());
  const timer = window.setInterval(() => setNow(Date.now()), 1000);
  onCleanup(() => window.clearInterval(timer));
  onMount(() => {
    if (!storeView.value() && !storeView.isLoading()) void storeView.load();
  });
  const weekly = (): Weekly[] => [
    { text: "useAbilities", done: num(props, "abilities", 35), goal: 200 },
    { text: "dealDamage", done: num(props, "damage", 1274), goal: 18_000 },
    { text: "useUltimate", done: num(props, "ultimates", 0), goal: 15 },
  ];
  const tier = () => num(props, "tier", 16);
  const tierMax = () => num(props, "tierMax", 50);
  const xpLeft = () => num(props, "xpLeft", 7450);
  const rewardImage = () => str(props, "rewardImage") || storeView.value()?.daily[0]?.image || null;
  const agent = () => str(props, "agent", "Brimstone");
  return (
    <div class={styles.page} style={colorVars(props, ["accent", "panel"])}>
      <section class={styles.missions}>
        <h2 class={styles.heading}>{vt("missions")}</h2>
        <div class={styles.box}>
          <header class={styles.boxHead}>
            <b>{vt("dailyMissions")}</b>
            <span>
              {vt("refreshesIn")} {untilReset(now(), false)}
            </span>
          </header>
          <p class={styles.boxNote}>{vt("dailyReward")}</p>
          <div class={styles.checkpoints}>
            <Index each={[1, 2, 3, 4]}>
              {(point) => (
                <span class={styles.checkpoint} data-done={point() <= num(props, "checkpoints", 0) ? "" : undefined}>
                  <b>{point()}</b>
                </span>
              )}
            </Index>
          </div>
          <div class={styles.checkpointBars}>
            <i />
            <i />
            <i />
          </div>
          <header class={styles.boxHead} data-weekly="">
            <b>{vt("weeklyMissions")}</b>
            <span>
              {vt("refreshesIn")} {untilReset(now(), true)}
            </span>
          </header>
          <For each={weekly()}>
            {(mission) => (
              <div class={styles.weekly}>
                <i class={styles.weeklyMark} />
                <div class={styles.weeklyBody}>
                  <span>{vt(mission.text)}</span>
                  <span class={styles.bar}>
                    <i style={{ width: `${Math.min(100, (mission.done / mission.goal) * 100)}%` }} />
                  </span>
                  <span class={styles.weeklyNumbers}>
                    <span>
                      {groupNumber(mission.done)}/{groupNumber(mission.goal)}
                    </span>
                    <b>
                      +{groupNumber(18_500)} {vt("xp")}
                    </b>
                  </span>
                </div>
              </div>
            )}
          </For>
        </div>
      </section>
      <section class={styles.pass}>
        <h2 class={styles.heading}>{vt("battlepass")}</h2>
        <div class={styles.passBox}>
          <span class={styles.season}>{str(props, "season", vt("season"))}</span>
          <span class={styles.ends}>
            {vt("endsIn")} <b>{num(props, "daysLeft", 76)} {vt("days")}</b>
          </span>
          <Art src={rewardImage()} class={styles.reward} fallback={<span />} />
          <span class={styles.premium}>
            <Lock size={16} /> {vt("premiumNeeded")}
          </span>
          <span class={styles.rewardName}>{str(props, "reward", "Ghost Datalink")}</span>
          <div class={styles.tierRow}>
            <PassGlyph size={22} />
            <span>
              {vt("tier")} {tier()} / {tierMax()}
            </span>
            <span class={styles.xpLeft}>
              {vt("xpLeft")}: {groupNumber(xpLeft())}
            </span>
          </div>
          <span class={styles.tierBar}>
            <i style={{ width: `${Math.min(100, (tier() / Math.max(1, tierMax())) * 100)}%` }} />
          </span>
          <button class={styles.upgrade} disabled>
            <span>
              <VpCoin size={18} /> 1000
            </span>
            <small>{vt("upgradePass")}</small>
          </button>
        </div>
        <h2 class={styles.heading}>{vt("objectives")}</h2>
        <div class={styles.objectives}>
          <div class={styles.objective}>
            <b>{vt("battlepass")}</b>
            <small>
              {vt("tier").toUpperCase()}: {tier()} / 20
            </small>
            <span class={styles.segments}>
              <Index each={[0, 1, 2, 3, 4]}>{(index) => <i data-on={index() < 1 ? "" : undefined} />}</Index>
            </span>
            <span class={styles.objectiveIcon} data-free="">
              {vt("free")}
            </span>
          </div>
          <For each={[5000, 2500]}>
            {(cost) => (
              <div class={styles.objective} data-locked="">
                <b>
                  {vt("rewards")} {agent()}
                </b>
                <small>
                  {vt("unlockWith")} <KcCoin size={12} /> {cost}
                </small>
                <span class={styles.objectiveIcon}>
                  <Lock size={20} />
                </span>
              </div>
            )}
          </For>
        </div>
      </section>
    </div>
  );
}

// ---- mode select ---------------------------------------------------------------------------

export function ValModes(props: PieceProps) {
  const play = usePlay();
  const navigate = useNavigate();
  // A different map behind every mode, spread across the pool so neighbours never match.
  const splash = (index: number) => {
    const maps = catalog().maps.filter((map) => map.splash && !map.is_team_deathmatch);
    return maps.length ? (maps[(index * 5 + 1) % maps.length]?.splash ?? null) : null;
  };
  return (
    <div class={styles.modes} style={colorVars(props, ["accent"])}>
      <h2 class={styles.modesTitle}>{textProp(props, "title", "pickMode")}</h2>
      <div class={styles.modeGrid}>
        <For each={play.queues()}>
          {(queue, index) => (
            <button
              class={styles.modeTile}
              data-active={play.current()?.id === queue.id ? "" : undefined}
              disabled={!play.isOwner()}
              onMouseEnter={() => playCue("hover")}
              onClick={() => {
                play.pickQueue(queue);
                if (props.piece.props["backOnPick"] !== false) navigate("/home");
              }}
            >
              <Art src={splash(index())} class={styles.modeImage} fallback={<span class={styles.modeEmpty} />} />
              <span class={styles.modeShade} />
              <span class={styles.modeName}>{queueName(queue.id, queue.label)}</span>
            </button>
          )}
        </For>
      </div>
      <Show when={play.queues().length === 0}>
        <p class={styles.modesEmpty}>{vt("offline")}</p>
      </Show>
    </div>
  );
}
