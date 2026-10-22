// Collection pieces: the loadout as a weapon wall grouped by category, and the skin picker
// for the selected weapon (owned skins, chroma swatches, Equip). Equipping changes your
// real loadout, so the wall only updates from Riot's answer.
import { For, Match, Show, Switch, createEffect, createMemo, createSignal, on, onMount } from "solid-js";
import { Check, Play } from "lucide-solid";
import type { SkinOption } from "~/bindings/SkinOption";
import type { WeaponSlot } from "~/bindings/WeaponSlot";
import type { PieceProps } from "~/studio/registry";
import { call, errorText } from "~/ipc/commands";
import { playCue } from "~/menu/sound";
import { pushToast } from "~/state/toasts";
import { collectionView } from "~/state/views";
import { Art } from "~/ui/Art";
import { Empty, controls } from "~/ui/controls";
import { doc } from "~/studio/store";
import type { Skin } from "~/studio/types";
import styles from "./collection.module.css";
import layouts from "./collectionLayouts.module.css";

export const [selectedWeapon, setSelectedWeapon] = createSignal<string | null>(null);

function useCollection(): void {
  onMount(() => {
    if (!collectionView.value()) void collectionView.load();
  });
}

function Status(props: { what: string }) {
  return (
    <Show
      when={collectionView.error()}
      fallback={<Empty title={props.what}>{collectionView.isLoading() ? "Opening your collection…" : "Connect to VALORANT to see your loadout."}</Empty>}
    >
      {(error) => <Empty title={props.what}>{error()}</Empty>}
    </Show>
  );
}

function WeaponCard(props: { slot: WeaponSlot; index: number }) {
  return (
    <button
      class={styles.weapon}
      data-part="weapon-card"
      style={{ "--stagger": props.index }}
      data-selected={selectedWeapon() === props.slot.weapon_id ? "" : undefined}
      onMouseEnter={() => playCue("hover")}
      onClick={() => {
        playCue("tab");
        setSelectedWeapon(props.slot.weapon_id);
      }}
    >
      <span class={styles.weaponArt}>
        <Art src={props.slot.image} fallback={<span class={styles.weaponInitial}>{props.slot.weapon_name}</span>} />
      </span>
      <span class={styles.weaponName}>{props.slot.weapon_name}</span>
      <span class={styles.skinName}>{props.slot.skin_name === props.slot.weapon_name ? "Standard" : props.slot.skin_name}</span>
    </button>
  );
}

type WallLayout = "categories" | "inventory" | "buymenu" | "list" | "slots";

/** The loadout's look when set to Auto: what the style's own game would show. */
const AUTO_WALL: Record<Skin, WallLayout> = {
  modern: "categories",
  current: "categories",
  glass: "categories",
  valnow: "categories",
  beta: "categories",
  fortnite: "categories",
  csgo: "inventory",
  cs2: "inventory",
  cs16: "buymenu",
  terminal: "list",
  halo: "list",
  minecraft: "slots",
  pipboy: "list",
  league: "categories",
};

/** Category names as a CS-style buy menu words them. */
const BUY_NAMES: Record<string, string> = {
  Sidearm: "Pistols",
  SMG: "Sub-machine guns",
  Shotgun: "Shotguns",
  Rifle: "Rifles",
  Sniper: "Sniper rifles",
  Heavy: "Machine guns",
  Melee: "Knife",
};

/** "Reaver Spectre" on a Spectre -> "Reaver"; the plain skin reads "Standard". */
function finishName(slot: WeaponSlot): string {
  if (slot.skin_name === slot.weapon_name) return "Standard";
  const trimmed = slot.skin_name.replace(new RegExp(`\\s*${slot.weapon_name}$`), "").trim();
  return trimmed || slot.skin_name;
}

function editionColor(edition: string | null | undefined): string {
  return edition ? (EDITION_COLORS[edition] ?? "var(--border-default)") : "#8f98a0";
}

function pickWeapon(slot: WeaponSlot): void {
  playCue("tab");
  setSelectedWeapon(slot.weapon_id);
}

export function WeaponWall(props: PieceProps) {
  useCollection();
  const groups = createMemo(() => {
    const byCategory = new Map<string, WeaponSlot[]>();
    for (const slot of collectionView.value()?.weapons ?? []) {
      byCategory.set(slot.category, [...(byCategory.get(slot.category) ?? []), slot]);
    }
    return [...byCategory.entries()];
  });
  const layout = (): WallLayout => {
    const chosen = props.piece.props["layout"];
    return typeof chosen === "string" && chosen !== "auto" ? (chosen as WallLayout) : AUTO_WALL[doc.theme.skin];
  };
  return (
    <Show when={collectionView.value()} fallback={<Status what="Collection" />}>
      <Switch>
        <Match when={layout() === "inventory"}>
          <InventoryWall groups={groups()} />
        </Match>
        <Match when={layout() === "buymenu"}>
          <BuyMenuWall groups={groups()} isEditing={props.isEditing} />
        </Match>
        <Match when={layout() === "list"}>
          <ListWall groups={groups()} />
        </Match>
        <Match when={layout() === "slots"}>
          <SlotWall groups={groups()} />
        </Match>
        <Match when={true}>
          <div class={styles.wall}>
            <div class={styles.columns}>
              <For each={wallColumns(groups())}>
                {(column, columnIndex) => (
                  <div class={styles.column}>
                    <For each={column}>
                      {([category, slots]) => (
                        <section class={styles.category} style={{ "--n": slots.length }}>
                          <span class={styles.categoryName} data-part="category-name">
                            {category}
                          </span>
                          <div class={styles.categoryGrid}>
                            <For each={slots}>{(slot, index) => <WeaponCard slot={slot} index={columnIndex() * 4 + index()} />}</For>
                          </div>
                        </section>
                      )}
                    </For>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Match>
      </Switch>
    </Show>
  );
}

type Groups = [string, WeaponSlot[]][];

/** VALORANT's own arrangement: sidearms; SMGs over shotguns; rifles over the knife; snipers
    over heavies. At most five weapons per column, so the whole loadout fits on one screen. */
const WALL_COLUMN: Record<string, number> = { sidearm: 0, smg: 1, shotgun: 1, rifle: 2, melee: 2, sniper: 3, heavy: 3 };

function wallColumns(groups: Groups): Groups[] {
  const columns: Groups[] = [[], [], [], []];
  for (const group of groups) {
    const known = WALL_COLUMN[group[0].toLowerCase()];
    const index = known ?? columns.reduce((best, column, at) => (column.length < (columns[best]?.length ?? 0) ? at : best), 0);
    columns[index]?.push(group);
  }
  return columns.filter((column) => column.length > 0);
}

/** CS:GO's inventory: filter tabs over one dense grid, "Weapon | Finish", a rarity bar. */
function InventoryWall(props: { groups: Groups }) {
  const [filter, setFilter] = createSignal<string | null>(null);
  const shown = () => props.groups.filter(([category]) => filter() === null || category === filter()).flatMap(([, slots]) => slots);
  return (
    <div class={layouts.inventory}>
      <header class={layouts.invHead}>
        <span class={layouts.invTitle}>Inventory</span>
        <nav class={layouts.invTabs}>
          <button data-active={filter() === null ? "" : undefined} onClick={() => setFilter(null)}>
            All
          </button>
          <For each={props.groups}>
            {([category]) => (
              <button data-active={filter() === category ? "" : undefined} onClick={() => setFilter(category)}>
                {category}
              </button>
            )}
          </For>
        </nav>
        <span class={`${layouts.invCount} num`}>{shown().length} items</span>
      </header>
      <div class={layouts.invGrid}>
        <For each={shown()}>
          {(slot, index) => (
            <button
              class={layouts.invTile}
              data-part="weapon-card"
              data-selected={selectedWeapon() === slot.weapon_id ? "" : undefined}
              style={{ "--edition": editionColor(slot.edition), "--stagger": index() }}
              onMouseEnter={() => playCue("hover")}
              onClick={() => pickWeapon(slot)}
            >
              <span class={layouts.invArt}>
                <Art src={slot.image} fallback={<span class={styles.weaponInitial}>{slot.weapon_name}</span>} />
              </span>
              <span class={layouts.invName}>
                {slot.weapon_name} <i>|</i> <b>{finishName(slot)}</b>
              </span>
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

/** CS 1.6's buy menu: numbered categories, then numbered guns, with the picture beside. Keys 1-9 pick, 0 goes back. */
function BuyMenuWall(props: { groups: Groups; isEditing: boolean }) {
  const [category, setCategory] = createSignal<string | null>(null);
  const [hovered, setHovered] = createSignal<WeaponSlot | null>(null);
  let root: HTMLDivElement | undefined;
  const items = () => props.groups.find(([name]) => name === category())?.[1] ?? [];
  const shownSlot = () =>
    hovered() ?? collectionView.value()?.weapons.find((slot) => slot.weapon_id === selectedWeapon()) ?? null;
  const press = (key: string) => {
    if (key === "0") {
      playCue("back");
      setCategory(null);
      return;
    }
    const index = Number(key) - 1;
    if (!Number.isInteger(index) || index < 0) return;
    if (category() === null) {
      const group = props.groups[index];
      if (group) {
        playCue("tab");
        setCategory(group[0]);
      }
    } else {
      const slot = items()[index];
      if (slot) pickWeapon(slot);
    }
  };
  onMount(() => {
    if (!props.isEditing) root?.focus({ preventScroll: true });
  });
  return (
    <div
      ref={root}
      class={layouts.buy}
      tabIndex={0}
      onKeyDown={(event) => {
        if (/^[0-9]$/.test(event.key)) {
          event.preventDefault();
          press(event.key);
        }
      }}
    >
      <header class={layouts.buyTitle}>{category() === null ? "Select a category" : (BUY_NAMES[category() ?? ""] ?? category())}</header>
      <div class={layouts.buyCols}>
        <ol class={layouts.buyList}>
          <Show
            when={category() !== null}
            fallback={
              <For each={props.groups}>
                {([name], index) => (
                  <li>
                    <button class={layouts.buyItem} onClick={() => press(String(index() + 1))}>
                      <kbd>{index() + 1}</kbd> {BUY_NAMES[name] ?? name}
                    </button>
                  </li>
                )}
              </For>
            }
          >
            <For each={items()}>
              {(slot, index) => (
                <li>
                  <button
                    class={layouts.buyItem}
                    data-selected={selectedWeapon() === slot.weapon_id ? "" : undefined}
                    onMouseEnter={() => setHovered(slot)}
                    onMouseLeave={() => setHovered(null)}
                    onClick={() => pickWeapon(slot)}
                  >
                    <kbd>{index() + 1}</kbd> {slot.weapon_name}
                    <small>{finishName(slot)}</small>
                  </button>
                </li>
              )}
            </For>
            <li>
              <button class={layouts.buyItem} onClick={() => press("0")}>
                <kbd>0</kbd> Back
              </button>
            </li>
          </Show>
        </ol>
        <div class={layouts.buyPreview}>
          <Show when={shownSlot()} fallback={<span class={layouts.buyHint}>Pick a weapon to see its skin.</span>}>
            {(slot) => (
              <>
                <Art src={slot().image} class={layouts.buyImage} fallback={<span class={styles.weaponInitial}>{slot().weapon_name}</span>} />
                <span class={layouts.buyCaption}>
                  {slot().weapon_name}: {finishName(slot())}
                </span>
              </>
            )}
          </Show>
        </div>
      </div>
      <footer class={layouts.buyFooter}>1-9 select · 0 back</footer>
    </div>
  );
}

/** Compact rows: picture, weapon, finish, category. */
function ListWall(props: { groups: Groups }) {
  return (
    <div class={layouts.list}>
      <For each={props.groups.flatMap(([, slots]) => slots)}>
        {(slot, index) => (
          <button
            class={layouts.listRow}
            data-part="weapon-card"
            data-selected={selectedWeapon() === slot.weapon_id ? "" : undefined}
            style={{ "--edition": editionColor(slot.edition), "--stagger": index() }}
            onMouseEnter={() => playCue("hover")}
            onClick={() => pickWeapon(slot)}
          >
            <span class={layouts.listArt}>
              <Art src={slot.image} fallback={<span />} />
            </span>
            <b>{slot.weapon_name}</b>
            <span>{finishName(slot)}</span>
            <small>{slot.category}</small>
          </button>
        )}
      </For>
    </div>
  );
}

/** Minecraft's inventory: nine square slots per row, the name in a tooltip. */
function SlotWall(props: { groups: Groups }) {
  return (
    <div class={layouts.slots}>
      <span class={layouts.slotsTitle}>Inventory</span>
      <div class={layouts.slotGrid}>
        <For each={props.groups.flatMap(([, slots]) => slots)}>
          {(slot) => (
            <button
              class={layouts.slot}
              data-part="weapon-card"
              data-selected={selectedWeapon() === slot.weapon_id ? "" : undefined}
              onMouseEnter={() => playCue("hover")}
              onClick={() => pickWeapon(slot)}
            >
              <Art src={slot.image} class={layouts.slotImage} fallback={<span class={layouts.slotLetter}>{slot.weapon_name.slice(0, 2)}</span>} />
              <span class={layouts.slotTip}>
                <b>{slot.skin_name}</b>
                <small>{slot.category}</small>
              </span>
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

/** Edition colours, as the game tints its store cards. */
const EDITION_COLORS: Record<string, string> = {
  Select: "#5a9fe2",
  Deluxe: "#009587",
  Premium: "#d1548d",
  Exclusive: "#f5955b",
  Ultra: "#fad663",
};
const MAX_TILT_DEG = 16;
const MIN_ZOOM = 1;
const MAX_ZOOM = 1.8;

/** The weapon on a stage: tilts toward the pointer, drag spins it, wheel zooms, double-click resets. */
function InspectStage(props: { image: string | null; fallback: string; edition: string | null; video: string | null; onCloseVideo: () => void; swapKey: string }) {
  const [tilt, setTilt] = createSignal({ x: 0, y: 0 });
  const [spin, setSpin] = createSignal(0);
  const [zoom, setZoom] = createSignal(1);
  let dragFrom: { x: number; spin: number } | null = null;
  const onPointerMove = (event: PointerEvent) => {
    const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = (event.clientX - box.left) / box.width - 0.5;
    const y = (event.clientY - box.top) / box.height - 0.5;
    setTilt({ x: -y * MAX_TILT_DEG, y: x * MAX_TILT_DEG });
    if (dragFrom) setSpin(dragFrom.spin + (event.clientX - dragFrom.x) * 0.5);
  };
  const reset = () => {
    setTilt({ x: 0, y: 0 });
    setSpin(0);
    setZoom(1);
  };
  return (
    <div
      class={styles.stage}
      data-part="skin-stage"
      data-edition={props.edition ?? undefined}
      style={{ "--edition": props.edition ? (EDITION_COLORS[props.edition] ?? "var(--accent)") : "var(--accent)" }}
      onPointerMove={onPointerMove}
      onPointerLeave={() => {
        dragFrom = null;
        setTilt({ x: 0, y: 0 });
      }}
      onPointerDown={(event) => {
        (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
        dragFrom = { x: event.clientX, spin: spin() };
      }}
      onPointerUp={() => (dragFrom = null)}
      onWheel={(event) => {
        event.preventDefault();
        setZoom((current) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, current - event.deltaY * 0.0015)));
      }}
      onDblClick={reset}
    >
      <span class={styles.stageFloor} />
      <Show when={props.edition}>
        {(edition) => (
          <span class={styles.edition}>
            <i />
            {edition()} Edition
          </span>
        )}
      </Show>
      <Show keyed when={props.swapKey}>
        <div class={styles.model}>
          <div
            class={styles.modelTilt}
            style={{ transform: `perspective(900px) rotateX(${tilt().x}deg) rotateY(${tilt().y + spin()}deg) scale(${zoom()})` }}
          >
            <Art src={props.image} class={styles.modelImage} fallback={<span class={styles.weaponInitial}>{props.fallback}</span>} />
            <span class={styles.sheen} />
          </div>
        </div>
      </Show>
      <Show when={props.image}>
        {(image) => <img class={styles.reflection} src={image()} alt="" aria-hidden="true" />}
      </Show>
      <span class={styles.stageHint}>Drag to turn · scroll to zoom · double-click to reset</span>
      <Show when={props.video}>
        {(url) => (
          <div class={styles.showcase}>
            <video class={styles.showcaseVideo} src={url()} autoplay controls playsinline onEnded={props.onCloseVideo} />
            <button class={styles.showcaseClose} title="Close video" onClick={props.onCloseVideo}>
              ×
            </button>
          </div>
        )}
      </Show>
    </div>
  );
}

export function SkinPicker(_props: PieceProps) {
  useCollection();
  const [options, setOptions] = createSignal<SkinOption[] | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [picked, setPicked] = createSignal<{ skinId: string; chromaId: string } | null>(null);
  const [isEquipping, setEquipping] = createSignal(false);
  const [video, setVideo] = createSignal<string | null>(null);
  const slot = () => collectionView.value()?.weapons.find((weapon) => weapon.weapon_id === selectedWeapon());

  createEffect(
    on(selectedWeapon, (weaponId) => {
      setOptions(null);
      setError(null);
      setPicked(null);
      setVideo(null);
      if (!weaponId) return;
      call("get_skin_options", { weaponId }).then(
        (next) => selectedWeapon() === weaponId && setOptions(next),
        (failure) => setError(errorText(failure)),
      );
    }),
  );

  const preview = () => {
    const choice = picked();
    const all = options() ?? [];
    const skin = all.find((option) => option.skin_id === (choice?.skinId ?? slot()?.skin_id));
    const chroma = skin?.chromas.find((entry) => entry.chroma_id === (choice?.chromaId ?? slot()?.chroma_id));
    return { skin, chroma, image: chroma?.image ?? skin?.image ?? slot()?.image ?? null };
  };

  const isEquipped = () => {
    const choice = picked();
    return !choice || (choice.skinId === slot()?.skin_id && choice.chromaId === slot()?.chroma_id);
  };

  const pick = (option: SkinOption) => {
    playCue("tab");
    setVideo(null);
    setPicked({ skinId: option.skin_id, chromaId: option.chromas[0]?.chroma_id ?? "" });
  };

  /** Step through the owned skins with the arrows (or ← →). */
  const step = (direction: 1 | -1) => {
    const all = options() ?? [];
    if (all.length === 0) return;
    const index = all.findIndex((option) => option.skin_id === preview().skin?.skin_id);
    const next = all[(index + direction + all.length) % all.length];
    if (next) pick(next);
  };

  const equip = async () => {
    const weaponId = selectedWeapon();
    const choice = picked();
    if (!weaponId || !choice) return;
    setEquipping(true);
    try {
      collectionView.set(await call("equip_skin", { weaponId, skinId: choice.skinId, chromaId: choice.chromaId }));
      playCue("confirm");
      setPicked(null);
    } catch (failure) {
      pushToast("negative", "Not equipped", errorText(failure));
    } finally {
      setEquipping(false);
    }
  };

  return (
    <Show when={slot()} fallback={<Empty title="Pick a weapon">Choose a weapon to see your skins for it.</Empty>}>
      {(current) => (
        <div
          class={styles.picker}
          data-part="screen-board"
          tabIndex={0}
          onKeyDown={(event) => {
            if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
              event.preventDefault();
              step(event.key === "ArrowRight" ? 1 : -1);
            }
          }}
        >
          <header class={styles.pickerHead}>
            <span class="overline">{current().category}</span>
            <div class={styles.titleRow}>
              <button class={styles.stepButton} title="Previous skin (←)" onClick={() => step(-1)}>
                ‹
              </button>
              <h2 class={styles.pickerTitle}>{preview().skin?.name ?? current().skin_name}</h2>
              <button class={styles.stepButton} title="Next skin (→)" onClick={() => step(1)}>
                ›
              </button>
            </div>
          </header>
          <InspectStage
            image={preview().image}
            fallback={current().weapon_name}
            edition={preview().skin?.edition ?? null}
            video={video()}
            onCloseVideo={() => setVideo(null)}
            swapKey={`${preview().skin?.skin_id ?? current().skin_id}:${preview().chroma?.chroma_id ?? ""}`}
          />
          <Show when={preview().skin}>
            {(skin) => (
              <div class={styles.upgrades}>
                <Show when={skin().levels.length > 1}>
                  <ol class={styles.levelTrack} aria-label="Upgrade levels">
                    <For each={skin().levels}>
                      {(level, index) => (
                        <li>
                          <button
                            class={styles.level}
                            data-part="skin-level"
                            data-owned={level.is_owned ? "" : undefined}
                            disabled={!level.video}
                            title={level.video ? `Play ${level.name}` : level.name}
                            onClick={() => {
                              playCue("confirm");
                              setVideo(level.video);
                            }}
                          >
                            <b class="num">{index() + 1}</b>
                            <span>{index() === 0 ? "Base" : (level.effect ?? "Upgrade")}</span>
                            <Show when={level.video}>
                              <Play size={11} />
                            </Show>
                          </button>
                        </li>
                      )}
                    </For>
                  </ol>
                </Show>
                <Show when={skin().chromas.length > 1}>
                  <div class={styles.chromas}>
                    <For each={skin().chromas}>
                      {(chroma) => (
                        <button
                          class={styles.chroma}
                          data-part="skin-chroma"
                          disabled={!chroma.is_owned}
                          title={chroma.is_owned ? chroma.name : `${chroma.name} (not owned)`}
                          data-active={(picked()?.chromaId ?? current().chroma_id) === chroma.chroma_id ? "" : undefined}
                          onClick={() => {
                            playCue("tab");
                            setVideo(null);
                            setPicked({ skinId: skin().skin_id, chromaId: chroma.chroma_id });
                          }}
                        >
                          <Art src={chroma.swatch} fallback={<span class={styles.swatchFallback} />} />
                        </button>
                      )}
                    </For>
                    <span class={styles.chromaName}>{preview().chroma?.name}</span>
                    <Show when={preview().chroma?.video}>
                      {(url) => (
                        <button class={styles.stepButton} title="Play this variant's video" onClick={() => setVideo(url())}>
                          <Play size={12} />
                        </button>
                      )}
                    </Show>
                  </div>
                </Show>
              </div>
            )}
          </Show>
          <div class={styles.skinList}>
            <Show when={options()} fallback={<span class={styles.loading}>{error() ?? "Loading your skins…"}</span>}>
              {(all) => (
                <For each={all()}>
                  {(option) => (
                    <button
                      class={styles.skin}
                      data-part="skin-row"
                      data-equipped={option.skin_id === current().skin_id ? "" : undefined}
                      data-picked={preview().skin?.skin_id === option.skin_id ? "" : undefined}
                      style={{ "--edition": option.edition ? (EDITION_COLORS[option.edition] ?? "transparent") : "transparent" }}
                      onMouseEnter={() => playCue("hover")}
                      onClick={() => pick(option)}
                    >
                      <Art src={option.image} fallback={<span />} />
                      <span>{option.name}</span>
                    </button>
                  )}
                </For>
              )}
            </Show>
          </div>
          <button class={controls.button} data-part="play" disabled={isEquipped() || isEquipping()} onClick={() => void equip()}>
            <Check size={15} /> {isEquipped() ? "Equipped" : isEquipping() ? "Equipping…" : "Equip"}
          </button>
        </div>
      )}
    </Show>
  );
}
