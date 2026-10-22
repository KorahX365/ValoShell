// The floating inspector: edit the selected piece, add pieces, restyle the theme, manage
// layers. The panel itself can be thrown around: it springs to the nearest side.
import { For, Match, Show, Switch as Branch, createMemo, createSignal, onCleanup, onMount, type JSX } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  ArrowDownToLine,
  ArrowUpToLine,
  Copy,
  Eye,
  EyeOff,
  Layers,
  Lock,
  LockOpen,
  Palette,
  Plus,
  RotateCcw,
  SlidersHorizontal,
  Trash2,
} from "lucide-solid";
import { playCue } from "~/menu/sound";
import { catalog } from "~/state/catalog";
import { areGameVideosSearched, gameVideos, pickGameVideo } from "~/state/gameVideos";
import { ColorField, Range, Segmented, Switch, TextField, controls } from "~/ui/controls";
import { addPieceOfKind, currentStage, duplicatePiece } from "./actions";
import { THEME_PRESETS } from "./defaults";
import { SKINS, applySkin, pieceFamily, skinLabel } from "./skins";
import { FigureControls, FramingSection, FxSection, SoundSection, TransitionSection } from "./ThemeExtras";
import { animateChange } from "./transitions";
import { applyLook, copyLook, deleteLook, lookNames, refreshLooks, saveLook } from "./looks";
import { PIECES, type PieceDef, type PropField, allowedOn, pieceDef } from "./registry";
import { GENTLE, type SpringHandle, VelocityTracker, project, runSpring } from "./spring";
import { doc, mutate, removePiece, resetEverything, restack, selectedId, setSelectedId, stagePieces, updatePiece, updateTheme } from "./store";
import { SHADER_OPTIONS } from "~/menu/backdrop/shaders";
import type { BackdropKind, DisplayFont, MenuLanguage, Piece, PropValue, ShaderId, Skin, Surface } from "./types";
import styles from "./studio.module.css";
import { Art } from "~/ui/Art";

type Tab = "piece" | "add" | "theme" | "layers";

const PANEL_WIDTH = 330;

const BACKDROP_KINDS: readonly { value: BackdropKind; label: string }[] = [
  { value: "game", label: "Game" },
  { value: "map", label: "Map" },
  { value: "agent", label: "Agent" },
  { value: "shader", label: "Shader" },
  { value: "aurora", label: "Glow" },
  { value: "image", label: "Image" },
  { value: "video", label: "Video" },
  { value: "color", label: "Flat" },
];
const PANEL_MARGIN = 16;
const LANGUAGES: readonly { value: MenuLanguage; label: string }[] = [
  { value: "auto", label: "System" },
  { value: "en", label: "English" },
  { value: "es", label: "Español" },
];

export function Inspector() {
  const [tab, setTab] = createSignal<Tab>("piece");
  const [position, setPosition] = createSignal({ x: window.innerWidth - PANEL_WIDTH - PANEL_MARGIN, y: 88 });
  let spring: SpringHandle | undefined;
  onCleanup(() => spring?.stop());

  const selected = createMemo(() => stagePieces(currentStage()).find((piece) => piece.id === selectedId()));

  const startDrag = (event: PointerEvent) => {
    if (event.button !== 0 || (event.target as HTMLElement).closest("button")) return;
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);
    spring?.stop();
    const origin = { x: event.clientX - position().x, y: event.clientY - position().y };
    const tracker = new VelocityTracker();
    const onMove = (move: PointerEvent) => {
      tracker.add(move.clientX, move.clientY);
      setPosition({ x: move.clientX - origin.x, y: move.clientY - origin.y });
    };
    const onUp = () => {
      handle.removeEventListener("pointermove", onMove);
      handle.removeEventListener("pointerup", onUp);
      const velocity = tracker.velocity();
      const from = position();
      const projectedX = from.x + project(velocity.x, 0.995);
      // Dock to whichever side the throw points at.
      const isRight = projectedX + PANEL_WIDTH / 2 > window.innerWidth / 2;
      const x = isRight ? window.innerWidth - PANEL_WIDTH - PANEL_MARGIN : PANEL_MARGIN;
      const y = Math.min(Math.max(from.y + project(velocity.y, 0.995), 80), window.innerHeight - 240);
      spring = runSpring([from.x, from.y], [x, y], [velocity.x, velocity.y], GENTLE, ([nx = x, ny = y]) =>
        setPosition({ x: nx, y: ny }),
      );
    };
    handle.addEventListener("pointermove", onMove);
    handle.addEventListener("pointerup", onUp);
  };

  const tabs: { id: Tab; label: string; icon: typeof Palette }[] = [
    { id: "piece", label: "Piece", icon: SlidersHorizontal },
    { id: "add", label: "Add", icon: Plus },
    { id: "theme", label: "Theme", icon: Palette },
    { id: "layers", label: "Layers", icon: Layers },
  ];

  return (
    <aside
      class={`${styles.inspector} ${controls.glass}`}
      style={{ transform: `translate3d(${position().x}px, ${position().y}px, 0)`, width: `${PANEL_WIDTH}px` }}
      onPointerDown={(event) => event.stopPropagation()}
    >
      <header class={styles.inspectorHead} onPointerDown={startDrag}>
        <span class={styles.grip} />
        <nav class={styles.tabs}>
          <For each={tabs}>
            {(entry) => (
              <button
                class={styles.tab}
                data-active={tab() === entry.id ? "" : undefined}
                onClick={() => {
                  playCue("tab");
                  setTab(entry.id);
                }}
              >
                <Dynamic component={entry.icon} size={14} />
                {entry.label}
              </button>
            )}
          </For>
        </nav>
      </header>
      <div class={styles.inspectorBody}>
        <Branch>
          <Match when={tab() === "piece"}>
            <Show
              when={selected()}
              fallback={
                <p class={styles.hint}>
                  Click a piece on the stage to edit it, or open <b>Add</b>. Paste any TikTok, YouTube or image link to drop it
                  straight in.
                </p>
              }
            >
              {(piece) => <PieceInspector piece={piece()} />}
            </Show>
          </Match>
          <Match when={tab() === "add"}>
            <AddPalette />
          </Match>
          <Match when={tab() === "theme"}>
            <ThemeInspector />
          </Match>
          <Match when={tab() === "layers"}>
            <LayersList />
          </Match>
        </Branch>
      </div>
    </aside>
  );
}

// ---- piece ------------------------------------------------------------------------------

function PieceInspector(props: { piece: Piece }) {
  const def = () => pieceDef(props.piece.kind);
  const stage = () => currentStage();
  const setProp = (key: string, value: PropValue) =>
    updatePiece(stage(), props.piece.id, (piece) => {
      piece.props[key] = value;
    }, `prop.${props.piece.id}.${key}`);
  const setStyle = <K extends keyof Piece["style"]>(key: K, value: Piece["style"][K]) =>
    updatePiece(stage(), props.piece.id, (piece) => {
      piece.style[key] = value;
    }, `style.${props.piece.id}.${key}`);
  const toggle = (key: "isHidden" | "isLocked") =>
    updatePiece(stage(), props.piece.id, (piece) => {
      piece[key] = !piece[key];
    });

  return (
    <div class={styles.stack}>
      <div class={styles.pieceTitle}>
        <Show when={def()}>{(current) => <Dynamic component={current().icon} size={18} />}</Show>
        <div>
          <div class={controls.heading}>{def()?.label ?? props.piece.kind}</div>
          <div class={styles.muted}>{def()?.description}</div>
        </div>
      </div>
      <div class={styles.actionRow}>
        <IconAction title="Duplicate (Ctrl+D)" onClick={() => duplicatePiece(stage(), props.piece.id)}>
          <Copy size={15} />
        </IconAction>
        <IconAction title="Bring forward ( ] )" onClick={() => restack(stage(), props.piece.id, 1)}>
          <ArrowUpToLine size={15} />
        </IconAction>
        <IconAction title="Send backward ( [ )" onClick={() => restack(stage(), props.piece.id, -1)}>
          <ArrowDownToLine size={15} />
        </IconAction>
        <IconAction title={props.piece.isLocked ? "Unlock" : "Lock in place"} onClick={() => toggle("isLocked")}>
          <Show when={props.piece.isLocked} fallback={<LockOpen size={15} />}>
            <Lock size={15} />
          </Show>
        </IconAction>
        <IconAction title={props.piece.isHidden ? "Show" : "Hide"} onClick={() => toggle("isHidden")}>
          <Show when={props.piece.isHidden} fallback={<Eye size={15} />}>
            <EyeOff size={15} />
          </Show>
        </IconAction>
        <IconAction
          title="Delete (Del)"
          isDanger
          onClick={() => {
            playCue("back");
            removePiece(stage(), props.piece.id);
          }}
        >
          <Trash2 size={15} />
        </IconAction>
      </div>
      <Show when={(def()?.fields.length ?? 0) > 0}>
        <section class={styles.section}>
          <span class="overline">Content</span>
          <For each={def()?.fields ?? []}>
            {(field) => <PropEditor field={field} value={props.piece.props[field.key]} onChange={(value) => setProp(field.key, value)} />}
          </For>
        </section>
      </Show>
      <section class={styles.section}>
        <span class="overline">Look</span>
        <Segmented<Surface>
          value={props.piece.style.surface}
          options={[
            { value: "none", label: "Bare" },
            { value: "glass", label: "Panel" },
            { value: "solid", label: "Solid" },
          ]}
          onChange={(value) => setStyle("surface", value)}
        />
        <Range
          label="Opacity"
          value={props.piece.style.opacity}
          min={0.1}
          max={1}
          step={0.01}
          format={(value) => `${Math.round(value * 100)}%`}
          onInput={(value) => setStyle("opacity", value)}
        />
        <Range
          label="Corners"
          value={props.piece.style.radius}
          min={0}
          max={48}
          step={1}
          format={(value) => `${value}px`}
          onInput={(value) => setStyle("radius", value)}
        />
        <Switch title="Accent outline" isOn={props.piece.style.isGlowing} onChange={(value) => setStyle("isGlowing", value)} />
      </section>
    </div>
  );
}

function PropEditor(props: { field: PropField; value: PropValue | undefined; onChange: (value: PropValue) => void }) {
  const asText = () => (typeof props.value === "string" ? props.value : "");
  const asNumber = () => (typeof props.value === "number" ? props.value : (props.field.min ?? 0));
  return (
    <Branch>
      <Match when={props.field.type === "toggle"}>
        <Switch title={props.field.label} isOn={props.value === true} onChange={(value) => props.onChange(value)} />
      </Match>
      <Match when={props.field.type === "range"}>
        <Range
          label={props.field.label}
          value={asNumber()}
          min={props.field.min ?? 0}
          max={props.field.max ?? 100}
          step={props.field.step ?? 1}
          onInput={(value) => props.onChange(value)}
        />
      </Match>
      <Match when={props.field.type === "select"}>
        <div class={styles.selectField}>
          <span class="overline">{props.field.label}</span>
          <Segmented value={asText()} options={props.field.options ?? []} onChange={(value) => props.onChange(value)} />
        </div>
      </Match>
      <Match when={props.field.type === "color"}>
        <div class={styles.colorProp}>
          <ColorField label={props.field.label} value={asText() || "#ffffff"} onInput={(value) => props.onChange(value)} />
          {/* An empty colour means the style's own; this puts it back. */}
          <Show when={asText()}>
            <button class={`${controls.button} ${controls.ghost} ${controls.small}`} title="Back to the style's colour" onClick={() => props.onChange("")}>
              Reset
            </button>
          </Show>
        </div>
      </Match>
      <Match when={props.field.type === "textarea"}>
        <TextField label={props.field.label} value={asText()} isMultiline onInput={(value) => props.onChange(value)} />
      </Match>
      <Match when={true}>
        <TextField
          label={props.field.label}
          value={asText()}
          placeholder={props.field.placeholder ?? ""}
          onInput={(value) => props.onChange(value)}
        />
      </Match>
    </Branch>
  );
}

function IconAction(props: { title: string; isDanger?: boolean; onClick: () => void; children: JSX.Element }) {
  return (
    <button
      class={styles.iconAction}
      data-danger={props.isDanger ? "" : undefined}
      title={props.title}
      onMouseEnter={() => playCue("hover")}
      onClick={() => props.onClick()}
    >
      {props.children}
    </button>
  );
}

// ---- add --------------------------------------------------------------------------------

interface PaletteSection {
  title: string;
  defs: PieceDef[];
}

function AddPalette() {
  const available = () => PIECES.filter((def) => allowedOn(def, currentStage()));
  const ofFamily = (family: Skin | null) => available().filter((def) => pieceFamily(def.kind) === family);
  /** Your style's pieces first, then the menu's own pieces and widgets. */
  const main = createMemo<PaletteSection[]>(() => {
    const skin = doc.theme.skin;
    const own = ofFamily(null);
    return [
      { title: `${skinLabel(skin)} pieces`, defs: skin === "modern" ? [] : ofFamily(skin) },
      { title: "Menu pieces", defs: own.filter((def) => def.category === "core") },
      { title: "Widgets", defs: own.filter((def) => def.category === "widget") },
    ].filter((section) => section.defs.length > 0);
  });
  const others = createMemo<PaletteSection[]>(() =>
    SKINS.filter((skin) => skin.id !== doc.theme.skin && skin.id !== "modern")
      .map((skin) => ({ title: skin.label, defs: ofFamily(skin.id) }))
      .filter((section) => section.defs.length > 0),
  );
  return (
    <div class={styles.stack}>
      <For each={main()}>{(section) => <PaletteGroup section={section} />}</For>
      <Show when={others().length > 0}>
        <details class={styles.fold}>
          <summary class="overline">Pieces from other styles</summary>
          <div class={styles.stack}>
            <For each={others()}>{(section) => <PaletteGroup section={section} />}</For>
          </div>
        </details>
      </Show>
      <p class={styles.hint}>Tip: paste a link (Ctrl+V) or drop an image anywhere on the stage.</p>
    </div>
  );
}

function PaletteGroup(props: { section: PaletteSection }) {
  const isPresent = (def: PieceDef) =>
    def.category === "core" && stagePieces(currentStage()).some((piece) => piece.kind === def.kind);
  return (
    <section class={styles.section}>
      <span class="overline">{props.section.title}</span>
      <div class={styles.palette}>
        <For each={props.section.defs}>
          {(def, index) => (
            <button
              class={styles.paletteItem}
              style={{ "--stagger": index() }}
              disabled={isPresent(def)}
              title={isPresent(def) ? "Already on this stage" : def.description}
              onMouseEnter={() => playCue("hover")}
              onClick={() => addPieceOfKind(currentStage(), def.kind)}
            >
              <Dynamic component={def.icon} size={20} />
              <span class={styles.paletteLabel}>{def.label}</span>
            </button>
          )}
        </For>
      </div>
    </section>
  );
}

// ---- theme ------------------------------------------------------------------------------

function LooksSection() {
  const [name, setName] = createSignal("");
  onMount(() => void refreshLooks());
  return (
    <section class={styles.looks}>
      <span class="overline">Looks</span>
      <form
        class={styles.lookForm}
        onSubmit={(event) => {
          event.preventDefault();
          void saveLook(name()).then((isSaved) => isSaved && setName(""));
        }}
      >
        <input class={controls.input} placeholder="Save this setup as…" value={name()} onInput={(event) => setName(event.currentTarget.value)} />
        <button class={`${controls.button} ${controls.small}`} type="submit" disabled={!name().trim()}>
          Save
        </button>
      </form>
      <Show when={lookNames().length > 0}>
        <ul class={styles.lookList}>
          <For each={lookNames()}>
            {(look) => (
              <li class={styles.look}>
                <button
                  class={styles.lookApply}
                  title="Apply (Ctrl+Z undoes it)"
                  onClick={() => {
                    playCue("confirm");
                    void applyLook(look);
                  }}
                >
                  {look}
                </button>
                <button class={styles.layerToggle} title="Delete" onClick={() => void deleteLook(look)}>
                  <Trash2 size={13} />
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <button class={`${controls.button} ${controls.ghost} ${controls.small}`} onClick={() => void copyLook()}>
        <Copy size={13} /> Copy this look to share
      </button>
      <p class={styles.hint}>Paste a copied look (Ctrl+V) anywhere while editing to apply it.</p>
    </section>
  );
}

function ThemeInspector() {
  const theme = () => doc.theme;
  const setColor = (key: "accent" | "accent2" | "ground" | "text", value: string) =>
    updateTheme((draft) => {
      draft[key] = value;
      draft.preset = "custom";
    }, `theme.${key}`);
  return (
    <div class={styles.stack}>
      <LooksSection />
      <section class={styles.section}>
        <span class="overline">Style</span>
        <div class={styles.skins} role="radiogroup">
          <For each={SKINS}>
            {(skin) => (
              <button
                role="radio"
                aria-checked={theme().skin === skin.id}
                class={styles.skin}
                data-active={theme().skin === skin.id ? "" : undefined}
                style={{ "--g": skin.ground, "--a": skin.accent, "--p": skin.play, "--t": skin.text }}
                onClick={() => {
                  playCue("confirm");
                  animateChange(() => mutate((draft) => applySkin(draft, skin)), "build");
                }}
              >
                <span class={styles.skinPreview} data-skin={skin.id}>
                  <span class={styles.skinNav} />
                  <span class={styles.skinPlay} />
                </span>
                <span class={styles.skinLabel}>{skin.label}</span>
                <span class={styles.skinCaption}>{skin.caption}</span>
              </button>
            )}
          </For>
        </div>
        <p class={styles.muted}>Also lays out the screens it styles (Home, Play, Career, and Store for some) with its own pieces. Ctrl+Z undoes it.</p>
        <div class={styles.selectField}>
          <span class="overline">Menu words (VALORANT styles)</span>
          <Segmented
            value={theme().language}
            options={LANGUAGES}
            onChange={(value) => updateTheme((draft) => void (draft.language = value), "theme.language")}
          />
        </div>
      </section>
      <section class={styles.section}>
        <span class="overline">Presets</span>
        <div class={styles.presets}>
          <For each={THEME_PRESETS}>
            {(preset) => (
              <button
                class={styles.preset}
                data-active={theme().preset === preset.id ? "" : undefined}
                style={{ "--a": preset.accent, "--b": preset.accent2, "--g": preset.ground }}
                title={preset.label}
                onClick={() => {
                  playCue("confirm");
                  animateChange(() =>
                    updateTheme((draft) => {
                      draft.preset = preset.id;
                      draft.accent = preset.accent;
                      draft.accent2 = preset.accent2;
                      draft.ground = preset.ground;
                      draft.text = preset.text;
                    }),
                  );
                }}
              >
                <span class={styles.presetSwatch} />
                <span class={styles.presetLabel}>{preset.label}</span>
              </button>
            )}
          </For>
        </div>
      </section>
      <section class={styles.section}>
        <span class="overline">Colours</span>
        <ColorField
          label="Accent"
          value={theme().accent}
          onInput={(value) => setColor("accent", value)}
        />
        <ColorField
          label="Highlight"
          value={theme().accent2}
          onInput={(value) => setColor("accent2", value)}
        />
        <ColorField
          label="Background"
          value={theme().ground}
          onInput={(value) => setColor("ground", value)}
        />
        <ColorField
          label="Text"
          value={theme().text}
          onInput={(value) => setColor("text", value)}
        />
      </section>
      <section class={styles.section}>
        <span class="overline">Backdrop</span>
        <div class={styles.kindGrid} role="radiogroup">
          <For each={BACKDROP_KINDS}>
            {(option) => (
              <button
                role="radio"
                aria-checked={theme().backdrop.kind === option.value}
                class={styles.kind}
                data-active={theme().backdrop.kind === option.value ? "" : undefined}
                onClick={() => {
                  playCue("tab");
                  updateTheme((draft) => void (draft.backdrop.kind = option.value));
                }}
              >
                {option.label}
              </button>
            )}
          </For>
        </div>
        <Show when={theme().backdrop.kind === "game"}>
          <Show
            when={gameVideos().length > 0}
            fallback={
              <p class={styles.muted}>
                {areGameVideosSearched()
                  ? "VALORANT install not found, so the glow is shown instead."
                  : "Looking for the game's videos…"}
              </p>
            }
          >
            <div class={styles.mapStrip}>
              <For each={gameVideos()}>
                {(video) => (
                  <button
                    class={styles.mapChip}
                    title={video.path}
                    data-active={pickGameVideo(theme().backdrop.gameVideo)?.path === video.path ? "" : undefined}
                    onClick={() => updateTheme((draft) => void (draft.backdrop.gameVideo = video.path))}
                  >
                    {video.label}
                  </button>
                )}
              </For>
            </div>
          </Show>
        </Show>
        <Show when={theme().backdrop.kind === "map"}>
          <div class={styles.mapStrip}>
            <For each={catalog().maps.filter((map) => !map.is_team_deathmatch)}>
              {(map) => (
                <button
                  class={styles.mapChip}
                  data-active={theme().backdrop.mapId === map.internal_id ? "" : undefined}
                  onClick={() => updateTheme((draft) => void (draft.backdrop.mapId = map.internal_id))}
                >
                  {map.display_name}
                </button>
              )}
            </For>
          </div>
        </Show>
        <Show when={theme().backdrop.kind === "shader"}>
          <Segmented<ShaderId>
            value={theme().backdrop.shader}
            options={SHADER_OPTIONS}
            onChange={(value) => updateTheme((draft) => void (draft.backdrop.shader = value))}
          />
        </Show>
        <Show when={theme().backdrop.kind === "image" || theme().backdrop.kind === "video"}>
          <TextField
            label={theme().backdrop.kind === "video" ? "Video link (.mp4 / .webm)" : "Image link"}
            value={theme().backdrop.url}
            placeholder="https://…"
            onInput={(value) => updateTheme((draft) => void (draft.backdrop.url = value.trim()), "theme.backdrop.url")}
          />
        </Show>
        <Show when={theme().backdrop.kind === "agent"}>
          <div class={styles.agentStrip}>
            <For each={catalog().agents}>
              {(agent) => (
                <button
                  class={styles.agentChip}
                  data-active={theme().backdrop.agentUuid === agent.uuid ? "" : undefined}
                  title={agent.display_name}
                  onClick={() => updateTheme((draft) => void (draft.backdrop.agentUuid = agent.uuid))}
                >
                  <Art src={agent.icon} fallback={agent.display_name.slice(0, 2)} />
                </button>
              )}
            </For>
          </div>
          <FigureControls />
        </Show>
        <Show when={theme().backdrop.kind !== "aurora" && theme().backdrop.kind !== "color"}>
          <Switch
            title="ASCII"
            hint="Redraw the background as characters"
            isOn={theme().backdrop.isAscii}
            onChange={(value) => updateTheme((draft) => void (draft.backdrop.isAscii = value))}
          />
          <Show when={theme().backdrop.isAscii}>
            <Range
              label="Character size"
              value={theme().backdrop.asciiSize}
              min={6}
              max={24}
              step={1}
              format={(value) => `${value}px`}
              onInput={(value) => updateTheme((draft) => void (draft.backdrop.asciiSize = value), "theme.asciiSize")}
            />
          </Show>
        </Show>
        <Range
          label="Dim"
          value={theme().backdrop.dim}
          min={0}
          max={0.9}
          step={0.01}
          format={(value) => `${Math.round(value * 100)}%`}
          onInput={(value) => updateTheme((draft) => void (draft.backdrop.dim = value), "theme.dim")}
        />
        <Range
          label="Blur"
          value={theme().backdrop.blur}
          min={0}
          max={24}
          step={1}
          format={(value) => `${value}px`}
          onInput={(value) => updateTheme((draft) => void (draft.backdrop.blur = value), "theme.blur")}
        />
        <Range
          label="Vignette"
          value={theme().vignette}
          min={0}
          max={1}
          step={0.01}
          format={(value) => `${Math.round(value * 100)}%`}
          onInput={(value) => updateTheme((draft) => void (draft.vignette = value), "theme.vignette")}
        />
        <Range
          label="Parallax"
          value={theme().parallax}
          min={0}
          max={40}
          step={1}
          format={(value) => (value === 0 ? "Off" : `${value}px`)}
          onInput={(value) => updateTheme((draft) => void (draft.parallax = value), "theme.parallax")}
        />
        <Switch title="Film grain" isOn={theme().isGrainy} onChange={(value) => updateTheme((draft) => void (draft.isGrainy = value))} />
      </section>
      <FramingSection />
      <SoundSection />
      <TransitionSection />
      <FxSection />
      <section class={styles.section}>
        <span class="overline">Shape and type</span>
        <Segmented<DisplayFont>
          value={theme().displayFont}
          options={[
            { value: "saira", label: "Saira" },
            { value: "tall", label: "Tall" },
            { value: "inter", label: "Inter" },
            { value: "mono", label: "Mono" },
            { value: "pixel", label: "Pixel" },
            { value: "classic", label: "Classic" },
            { value: "robco", label: "Pip-Boy" },
          ]}
          onChange={(value) => updateTheme((draft) => void (draft.displayFont = value))}
        />
        <Range
          label="Cut corners"
          value={theme().cut}
          min={0}
          max={32}
          step={1}
          format={(value) => `${value}px`}
          onInput={(value) => updateTheme((draft) => void (draft.cut = value), "theme.cut")}
        />
      </section>
      <button
        class={`${controls.button} ${controls.ghost} ${controls.small}`}
        onClick={() => {
          playCue("back");
          resetEverything();
        }}
      >
        <RotateCcw size={14} /> Reset everything
      </button>
    </div>
  );
}

// ---- layers -----------------------------------------------------------------------------

function LayersList() {
  const ordered = () => [...stagePieces(currentStage())].sort((a, b) => b.z - a.z);
  return (
    <ol class={styles.layers}>
      <For each={ordered()}>
        {(piece) => {
          const def = () => pieceDef(piece.kind);
          return (
            <li
              class={styles.layer}
              data-selected={selectedId() === piece.id ? "" : undefined}
              onClick={() => setSelectedId(piece.id)}
            >
              <Show when={def()}>{(current) => <Dynamic component={current().icon} size={14} />}</Show>
              <span class={styles.layerName}>{def()?.label ?? piece.kind}</span>
              <button
                class={styles.layerToggle}
                title={piece.isLocked ? "Unlock" : "Lock"}
                onClick={(event) => {
                  event.stopPropagation();
                  updatePiece(currentStage(), piece.id, (draft) => void (draft.isLocked = !draft.isLocked));
                }}
              >
                <Show when={piece.isLocked} fallback={<LockOpen size={13} />}>
                  <Lock size={13} />
                </Show>
              </button>
              <button
                class={styles.layerToggle}
                title={piece.isHidden ? "Show" : "Hide"}
                onClick={(event) => {
                  event.stopPropagation();
                  updatePiece(currentStage(), piece.id, (draft) => void (draft.isHidden = !draft.isHidden));
                }}
              >
                <Show when={piece.isHidden} fallback={<Eye size={13} />}>
                  <EyeOff size={13} />
                </Show>
              </button>
            </li>
          );
        }}
      </For>
    </ol>
  );
}
