// Game-style navigation. Modern, CS2 and Fortnite use the floating top bar; the other menu
// styles draw their own game's nav: the current VALORANT client's icon bar and social panel, CS:GO's segmented bar, the Beta's thin spaced labels with
// the emblem, a side menu (VALORANT 2022, CS 1.6, Halo 3), Minecraft's stone buttons and a
// terminal prompt line, and the League client's PLAY crest with its social column. All share the tabs,
// Q/E cycling, wallet and system buttons.
import { For, Match, Show, Switch, onCleanup, onMount, type JSX } from "solid-js";
import { A, useLocation, useNavigate } from "@solidjs/router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { House, Minus, Paintbrush, Power, Settings, Users } from "lucide-solid";
import { isTauri } from "~/ipc/runtime";
import { doc, isEditing, setEditMode } from "~/studio/store";
import { tabLabel } from "~/studio/skins";
import { isDrawerOpen, onlineCount, setDrawerOpen } from "~/state/friends";
import { connection, game } from "~/state/session";
import { wallet } from "~/state/wallet";
import { TABS, getAdjacentTab, sceneForPath, type SceneInfo } from "./scenes";
import { CurrentNav } from "./CurrentNav";
import { GlassNav } from "./GlassNav";
import { LeagueNav } from "./LeagueNav";
import { openEscMenu } from "./EscMenu";
import { playCue } from "./sound";
import { isTyping } from "./typing";
import styles from "./TopNav.module.css";
import nav from "./NavVariants.module.css";

const ICON_SIZE = 17;

export function TopNav() {
  const location = useLocation();
  const navigate = useNavigate();

  const cycleTab = (step: -1 | 1) => {
    playCue("tab");
    navigate(`/${getAdjacentTab(sceneForPath(location.pathname), step).id}`);
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (isTyping(event.target) || event.repeat || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key === "q" || event.key === "Q") cycleTab(-1);
    if (event.key === "e" || event.key === "E") cycleTab(1);
  };
  onMount(() => window.addEventListener("keydown", onKeyDown));
  onCleanup(() => window.removeEventListener("keydown", onKeyDown));

  const skin = () => doc.theme.skin;
  return (
    <Switch fallback={<BarNav />}>
      <Match when={skin() === "glass"}>
        <GlassNav />
      </Match>
      <Match when={skin() === "current"}>
        <CurrentNav />
      </Match>
      <Match when={skin() === "csgo"}>
        <CsgoNav />
      </Match>
      <Match when={skin() === "beta"}>
        <BetaNav />
      </Match>
      <Match when={skin() === "valnow" || skin() === "cs16" || skin() === "halo"}>
        <SideNav />
      </Match>
      <Match when={skin() === "minecraft"}>
        <McNav />
      </Match>
      <Match when={skin() === "terminal"}>
        <TerminalNav />
      </Match>
      <Match when={skin() === "pipboy"}>
        <PipNav />
      </Match>
      <Match when={skin() === "league"}>
        <LeagueNav />
      </Match>
    </Switch>
  );
}

// ---- shared parts -----------------------------------------------------------------------

const isLive = () => game()?.phase === "pregame" || game()?.phase === "ingame";
const label = (tab: SceneInfo) => tabLabel(doc.theme.skin, tab.id, tab.label);

const LIVE_TAB: SceneInfo = { id: "live", label: "Live", isTab: false };

function TabLink(props: { tab: SceneInfo; class: string | undefined; activeClass: string | undefined; children?: JSX.Element }) {
  return (
    <A
      href={`/${props.tab.id}`}
      class={props.class ?? ""}
      data-part="tab"
      activeClass={props.activeClass ?? ""}
      onMouseEnter={() => playCue("hover")}
      onClick={() => playCue("tab")}
    >
      {props.children ?? label(props.tab)}
    </A>
  );
}

function SystemButtons() {
  return (
    <>
      <IconButton
        label="Friends"
        isActive={isDrawerOpen()}
        onClick={() => {
          playCue(isDrawerOpen() ? "back" : "confirm");
          setDrawerOpen(!isDrawerOpen());
        }}
      >
        <Users size={ICON_SIZE} />
        <Show when={onlineCount() > 0}>
          <span class={styles.badge}>{onlineCount()}</span>
        </Show>
      </IconButton>
      <IconButton
        label={isEditing() ? "Close Studio (Esc)" : "Studio: edit everything (Ctrl+E)"}
        isActive={isEditing()}
        onClick={() => {
          playCue(isEditing() ? "back" : "confirm");
          setEditMode(!isEditing());
        }}
      >
        <Paintbrush size={ICON_SIZE} />
      </IconButton>
      <IconButton label="Settings (Esc)" onClick={openEscMenu}>
        <Settings size={ICON_SIZE} />
      </IconButton>
      <Show when={isTauri}>
        <IconButton label="Minimise" onClick={() => void getCurrentWindow().minimize()}>
          <Minus size={ICON_SIZE} />
        </IconButton>
        <IconButton label="Hide to tray (Ctrl+Alt+V brings it back)" onClick={() => void getCurrentWindow().hide()}>
          <Power size={ICON_SIZE} />
        </IconButton>
      </Show>
    </>
  );
}

function StatusLight() {
  return <span class={styles.status} data-state={connection().state} title={connection().state.replace(/_/g, " ")} />;
}

// ---- Modern / CS2: floating bar ---------------------------------------------------------

function BarNav() {
  return (
    <header class={styles.nav} data-part="nav" data-tauri-drag-region>
      <div class={styles.brand} data-tauri-drag-region>
        <span class={styles.mark} aria-hidden="true" />
        <StatusLight />
      </div>
      <nav class={styles.tabs} aria-label="Menu">
        <kbd class={styles.key} aria-hidden="true">
          Q
        </kbd>
        <For each={TABS}>{(tab) => <TabLink tab={tab} class={styles.tab} activeClass={styles.active} />}</For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={`${styles.tab} ${styles.liveTab}`} activeClass={styles.active}>
            <span class={styles.liveDot} />
            Live
          </TabLink>
        </Show>
        <kbd class={styles.key} aria-hidden="true">
          E
        </kbd>
      </nav>
      <div class={styles.side}>
        <Wallet />
        <span class={styles.divider} aria-hidden="true" />
        <SystemButtons />
      </div>
    </header>
  );
}

// ---- CS:GO: segmented bar with home and power cells -------------------------------------

function CsgoNav() {
  const home = () => TABS.find((tab) => tab.id === "home") ?? LIVE_TAB;
  const tabs = () => TABS.filter((tab) => tab.id !== "home");
  return (
    <header class={nav.csgo} data-part="nav" data-tauri-drag-region>
      <div class={nav.csgoCorner} data-tauri-drag-region>
        <StatusLight />
      </div>
      <nav class={nav.csgoCells} aria-label="Menu">
        <TabLink tab={home()} class={`${nav.csgoCell} ${nav.csgoIconCell}`} activeClass={nav.csgoActive}>
          <House size={26} />
        </TabLink>
        <For each={tabs()}>{(tab) => <TabLink tab={tab} class={nav.csgoCell} activeClass={nav.csgoActive} />}</For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.csgoCell} activeClass={nav.csgoActive}>
            Live
          </TabLink>
        </Show>
        <button class={`${nav.csgoCell} ${nav.csgoIconCell}`} title="Menu (Esc)" onMouseEnter={() => playCue("hover")} onClick={openEscMenu}>
          <Power size={26} />
        </button>
      </nav>
      <div class={nav.csgoSide}>
        <Wallet />
        <SystemButtons />
      </div>
    </header>
  );
}

// ---- VALORANT Beta: thin spaced labels on a hairline, emblem below ----------------------

function BetaNav() {
  return (
    <header class={nav.beta} data-part="nav" data-tauri-drag-region>
      <div class={nav.betaCorner} data-tauri-drag-region>
        <StatusLight />
      </div>
      <nav class={nav.betaTabs} aria-label="Menu">
        <For each={TABS}>{(tab) => <TabLink tab={tab} class={nav.betaTab} activeClass={nav.betaActive} />}</For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.betaTab} activeClass={nav.betaActive}>
            Live
          </TabLink>
        </Show>
      </nav>
      <div class={nav.betaSide}>
        <Wallet />
        <SystemButtons />
      </div>
      <svg class={nav.betaEmblem} viewBox="0 0 160 120" aria-hidden="true">
        <path d="M0 0 L66 62 M160 0 L94 62" />
        <path class={nav.betaEmblemFill} d="M80 22 L104 74 L92 74 L80 48 L68 74 L56 74 Z" />
        <path d="M80 82 L84 88 L80 94 L76 88 Z M80 94 L80 120" />
      </svg>
    </header>
  );
}

// ---- side menu: VALORANT 2022 (big list, red PLAY), CS 1.6 (plain list), Halo 3 (bars) --

type SideVariant = "list" | "classic" | "halo";

function SideNav() {
  const variant = (): SideVariant => (doc.theme.skin === "cs16" ? "classic" : doc.theme.skin === "halo" ? "halo" : "list");
  const isClassic = () => variant() === "classic";
  const hasBigPlay = () => variant() === "list";
  const listed = () => (hasBigPlay() ? TABS.filter((tab) => tab.id !== "play") : TABS);
  const playTab = () => TABS.find((tab) => tab.id === "play") ?? LIVE_TAB;
  return (
    <>
      <header class={nav.sideBar} data-part="nav" data-tauri-drag-region>
        <div class={nav.sideBarStatus} data-tauri-drag-region>
          <StatusLight />
        </div>
        <Wallet />
        <SystemButtons />
      </header>
      <nav class={nav.side} data-variant={variant()} aria-label="Menu">
        <Show when={hasBigPlay()}>
          <TabLink tab={playTab()} class={nav.sidePlay} activeClass={nav.sideActive} />
        </Show>
        <Show when={!hasBigPlay()}>
          <span class={nav.sideLogo}>ValoShell</span>
        </Show>
        <For each={listed()}>{(tab) => <TabLink tab={tab} class={nav.sideItem} activeClass={nav.sideActive} />}</For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.sideItem} activeClass={nav.sideActive}>
            Live
          </TabLink>
        </Show>
        <Show when={isClassic()}>
          <button class={nav.sideItem} onClick={openEscMenu}>
            Quit
          </button>
        </Show>
        <Show when={variant() === "halo"}>
          <footer class={nav.sideHints}>
            <span>
              <i data-button="a">A</i> Select
            </span>
            <span>
              <i data-button="b">B</i> Back
            </span>
          </footer>
        </Show>
      </nav>
    </>
  );
}

// ---- Minecraft: a row of stone buttons --------------------------------------------------

function McNav() {
  return (
    <header class={nav.mc} data-part="nav" data-tauri-drag-region>
      <div class={nav.mcCorner} data-tauri-drag-region>
        <StatusLight />
      </div>
      <nav class={nav.mcTabs} aria-label="Menu">
        <For each={TABS}>{(tab) => <TabLink tab={tab} class={nav.mcTab} activeClass={nav.mcActive} />}</For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.mcTab} activeClass={nav.mcActive}>
            Live
          </TabLink>
        </Show>
      </nav>
      <div class={nav.mcSide}>
        <Wallet />
        <SystemButtons />
      </div>
    </header>
  );
}

// ---- terminal: one prompt line ---------------------------------------------------------

function TerminalNav() {
  return (
    <header class={nav.term} data-part="nav" data-tauri-drag-region>
      <span class={nav.termPrompt} data-tauri-drag-region>
        valoshell@{connection().state === "connected" ? "valorant" : "offline"}:~$
      </span>
      <nav class={nav.termTabs} aria-label="Menu">
        <For each={TABS}>
          {(tab, index) => (
            <TabLink tab={tab} class={nav.termTab} activeClass={nav.termActive}>
              [{index() + 1}] {label(tab).toLowerCase()}
            </TabLink>
          )}
        </For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.termTab} activeClass={nav.termActive}>
            [*] live
          </TabLink>
        </Show>
      </nav>
      <div class={nav.termSide}>
        <Wallet />
        <SystemButtons />
      </div>
    </header>
  );
}

// ---- Pip-Boy: tab names on a rule that lifts into a bracket round the open one -----------

function PipNav() {
  return (
    <header class={nav.pip} data-part="nav" data-tauri-drag-region>
      <nav class={nav.pipTabs} aria-label="Menu">
        <For each={TABS}>
          {(tab) => (
            <TabLink tab={tab} class={nav.pipTab} activeClass={nav.pipActive}>
              {label(tab).toUpperCase()}
            </TabLink>
          )}
        </For>
        <Show when={isLive()}>
          <TabLink tab={LIVE_TAB} class={nav.pipTab} activeClass={nav.pipActive}>
            LIVE
          </TabLink>
        </Show>
      </nav>
      <div class={nav.pipSide}>
        <StatusLight />
        <Wallet />
        <SystemButtons />
      </div>
    </header>
  );
}

function Wallet() {
  const currencies = () => {
    const balances = wallet();
    return [
      { id: "vp", label: "VP", value: balances?.valorant_points },
      { id: "rp", label: "RP", value: balances?.radianite_points },
      { id: "kc", label: "KC", value: balances?.kingdom_credits },
    ];
  };
  return (
    <div class={styles.wallet} data-part="wallet" data-sensitive>
      <For each={currencies()}>
        {(currency) => (
          <span class={styles.currency}>
            <span class={styles.currencyLabel}>{currency.label}</span>
            <span class="num">{currency.value === undefined ? "-" : currency.value.toLocaleString()}</span>
          </span>
        )}
      </For>
    </div>
  );
}

function IconButton(props: { label: string; isActive?: boolean; onClick: () => void; children: JSX.Element }) {
  return (
    <button
      class={styles.iconButton}
      data-part="icon-button"
      data-active={props.isActive ? "" : undefined}
      aria-label={props.label}
      title={props.label}
      onMouseEnter={() => playCue("hover")}
      onClick={() => props.onClick()}
    >
      {props.children}
    </button>
  );
}
