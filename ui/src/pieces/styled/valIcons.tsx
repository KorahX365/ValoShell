// Glyphs of the current VALORANT client that no icon set has: the V mark, the currency coins,
// the battlepass ticket and the mode and party-button marks. All draw in currentColor.

export function ValLogo(props: { size?: number; class?: string | undefined }) {
  // Drawn on the client's 30 x 25 box: the long left stroke and the short right one.
  const size = () => props.size ?? 24;
  return (
    <svg class={props.class} width={size()} height={(size() * 25) / 30} viewBox="0 0 30 25" fill="currentColor" aria-hidden="true">
      <path d="M0 0.6 V9.4 L12.4 25 H21.6 Z" />
      <path d="M30 0.6 V12.8 L27.4 16.2 H18.2 Z" />
    </svg>
  );
}

export function VpCoin(props: { size?: number }) {
  return (
    <svg width={props.size ?? 18} height={props.size ?? 18} viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
      <circle cx="12" cy="12" r="10" stroke-width="1.6" />
      <path d="M7.4 8 L11.2 15.2 L13 15.2 L9.2 8 Z M16.6 8 L13.6 13.2 L12.6 11.4 L14.3 8 Z" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function RpCoin(props: { size?: number }) {
  return (
    <svg width={props.size ?? 18} height={props.size ?? 18} viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
      <circle cx="12" cy="12" r="10" stroke-width="1.6" />
      <path d="M9 17 V7 H13 A3 3 0 0 1 13 13 H9 M12.6 13 L15.4 17" stroke-width="1.9" stroke-linejoin="round" />
    </svg>
  );
}

export function KcCoin(props: { size?: number }) {
  return (
    <svg width={props.size ?? 18} height={props.size ?? 18} viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
      <path d="M12 2.5 L21.5 12 L12 21.5 L2.5 12 Z" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M12 7.5 L12 16.5 M9 10 L12 7.5 L15 10" stroke-width="1.6" />
    </svg>
  );
}

export function PassGlyph(props: { size?: number }) {
  // The battlepass ticket on its corner: notched ends and a diamond punched through.
  return (
    <svg width={props.size ?? 24} height={props.size ?? 24} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path
        transform="rotate(-45 12 12)"
        d="M2.5 7 H21.5 V9.8 A2.2 2.2 0 0 0 21.5 14.2 V17 H2.5 V14.2 A2.2 2.2 0 0 0 2.5 9.8 Z M7.4 12 L9.4 9.8 L11.4 12 L9.4 14.2 Z M14 8.6 H15 V15.4 H14 Z"
        fill-rule="evenodd"
      />
    </svg>
  );
}

/** Range: four arms round a dot, a rounded corner bracket between each pair (client icon). */
export function RangeGlyph(props: { size?: number }) {
  return (
    <svg width={props.size ?? 24} height={props.size ?? 24} viewBox="0 0 28 28" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
      <circle cx="14" cy="14" r="1.9" fill="currentColor" stroke="none" />
      <path d="M14 1 V8.5 M14 19.5 V27 M1 14 H8.5 M19.5 14 H27" />
      <path d="M10 3.5 V6.5 Q10 10 6.5 10 H3.5 M18 3.5 V6.5 Q18 10 21.5 10 H24.5 M18 24.5 V21.5 Q18 18 21.5 18 H24.5 M10 24.5 V21.5 Q10 18 6.5 18 H3.5" />
    </svg>
  );
}

/** One asterisk cut out of the code bubble: a six-pointed star round (cx, cy). */
function starPath(cx: number, cy: number, radius: number): string {
  const points = Array.from({ length: 12 }, (_, index) => {
    const angle = (Math.PI / 6) * index - Math.PI / 2;
    const reach = index % 2 === 0 ? radius : radius * 0.42;
    return `${(cx + Math.cos(angle) * reach).toFixed(2)} ${(cy + Math.sin(angle) * reach).toFixed(2)}`;
  });
  return `M${points.join(" L")} Z`;
}

const CODE_STARS = [7.5, 15, 22.5].map((x) => starPath(x, 10, 3.6)).join(" ");

/** Party code: a filled rounded bubble, its tail at the bottom left, with three stars. */
export function CodeGlyph(props: { size?: number }) {
  return (
    <svg width={props.size ?? 24} height={props.size ?? 24} viewBox="0 0 30 24" fill="currentColor" aria-hidden="true">
      <path
        d={`M3 3 H27 Q29 3 29 5 V15 Q29 17 27 17 H11.5 L6.5 21.5 V17 H3 Q1 17 1 15 V5 Q1 3 3 3 Z ${CODE_STARS}`}
        fill-rule="evenodd"
      />
    </svg>
  );
}

/** Leave party: an arrow breaking out through the left point of a diamond. */
export function LeaveGlyph(props: { size?: number }) {
  return (
    <svg width={props.size ?? 24} height={props.size ?? 24} viewBox="0 0 27 26" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
      <path d="M10.5 6.5 L15 2 L26 13 L15 24 L10.5 19.5" stroke-linejoin="round" />
      <path d="M20 13 H3.5 M8 8.5 L3.5 13 L8 17.5" />
    </svg>
  );
}

/** The party lock's knob: a solid padlock with a keyhole, its shackle up when open. */
export function PadlockGlyph(props: { isOpen: boolean }) {
  return (
    <svg width="11" height="14" viewBox="0 0 11 14" aria-hidden="true">
      <path d={props.isOpen ? "M2.5 6 V4 A3 3 0 0 1 8.5 3" : "M2.5 6 V4 A3 3 0 0 1 8.5 4 V6"} fill="none" stroke="currentColor" stroke-width="1.8" />
      <path d="M1 6 H10 V13 H1 Z M5.5 8 A1.1 1.1 0 0 0 4.9 10.1 V11.6 H6.1 V10.1 A1.1 1.1 0 0 0 5.5 8 Z" fill="currentColor" fill-rule="evenodd" />
    </svg>
  );
}

/** The party chat's channel mark: a bubble with a square bottom-left corner round a ring and dot. */
export function ChatGlyph(props: { size?: number; class?: string | undefined }) {
  return (
    <svg class={props.class} width={props.size ?? 25} height={props.size ?? 25} viewBox="0 0 25 25" fill="none" stroke="currentColor" aria-hidden="true">
      <path d="M1 24 V12.5 A11.5 11.5 0 1 1 12.5 24 Z" stroke-width="1.3" stroke-linejoin="round" />
      <path d="M17.5 8.5 A6.5 6.5 0 1 0 19 13" stroke-width="1.3" stroke-linecap="round" />
      <circle cx="10.5" cy="13.5" r="3" fill="currentColor" stroke="none" />
    </svg>
  );
}

/** Riot's fist, the client's mark for friends in another Riot game. */
export function RiotFistGlyph(props: { size?: number }) {
  return (
    <svg width={props.size ?? 20} height={props.size ?? 20} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M3 6.5 L19.5 3 L21.5 15 L13 21 L7.2 20 L6.7 15.8 L10 16.2 L9.6 12.7 L5.3 12.2 Z M8 6.6 L8.3 10.4 L10.5 10.7 L10.2 6.3 Z M12 6 L12.3 10.9 L14.5 11.1 L14.2 5.6 Z M16 5.3 L16.4 11.3 L18.4 11.4 L18 4.9 Z" fill-rule="evenodd" />
    </svg>
  );
}

/** The game a friend is playing, as the client's strip shows it in place of their card. */
export function ProductGlyph(props: { product: string; size?: number }) {
  const size = () => props.size ?? 26;
  const id = () => props.product.toLowerCase();
  if (id() === "league_of_legends") {
    // League's crest: a gold ring broken where the tall serif L stands over it.
    return (
      <svg width={size()} height={size()} viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M10.5 4.1 A8.2 8.2 0 1 1 6.2 18.6" stroke="#c8aa6e" stroke-width="1.3" />
        <path d="M10.5 6.2 A6.1 6.1 0 1 1 9.4 17.4" stroke="#c8aa6e" stroke-width="0.8" opacity="0.7" />
        <path d="M4.2 2.2 H9.6 L8.4 3.6 V17.6 H17.8 L16.2 20.6 H3.6 L4.9 19.2 V3.4 Z" fill="#c8aa6e" />
      </svg>
    );
  }
  if (id() === "lion") {
    return (
      <svg width={size()} height={size()} viewBox="0 0 24 24" fill="#ffffff" aria-hidden="true">
        <path d="M5 3 H9 L12.5 12 L9 21 H5 L8.5 12 Z M13 3 H19 L14.5 11 Z M14.5 13 L19 21 H13 Z" />
      </svg>
    );
  }
  if (id() === "tft") {
    return (
      <svg width={size()} height={size()} viewBox="0 0 24 24" fill="none" stroke="#e8c26a" stroke-width="1.6" aria-hidden="true">
        <path d="M12 2.5 L20.5 7.25 V16.75 L12 21.5 L3.5 16.75 V7.25 Z" />
        <path d="M8 8 H16 M12 8 V17" />
      </svg>
    );
  }
  return <RiotFistGlyph size={size() - 4} />;
}

/** Competitive: the ranked diamond. */
export function CompetitiveGlyph(props: { size?: number }) {
  return (
    <svg width={props.size ?? 24} height={props.size ?? 24} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 1.5 L22.5 12 L12 22.5 L1.5 12 Z M12 5.2 L5.2 12 L12 18.8 L18.8 12 Z" fill-rule="evenodd" />
      <path d="M12 8.6 L15.4 12 L12 15.4 L8.6 12 Z" />
    </svg>
  );
}

/** The Spike, as the client marks Unrated and Swiftplay matches (traced at 43 x 36). */
export function SpikeGlyph(props: { size?: number }) {
  const width = () => props.size ?? 40;
  return (
    <svg width={width()} height={(width() * 36) / 43} viewBox="0 0 43 36" fill="currentColor" aria-hidden="true">
      <path d="M17 0 H27 L24.8 4 H19.2 Z" />
      <path
        d="M6 1 H15.2 L17.4 4.4 V8.4 L15.4 9.6 H13.4 L12.6 11 V15.6 H6.4 L5 13 L3 10.4 L2 7 L3 3.6 Z M7 5 H9 V7.6 H7 Z"
        fill-rule="evenodd"
      />
      <path
        d="M37 1 H27.8 L25.6 4.4 V8.4 L27.6 9.6 H29.6 L30.4 11 V15.6 H36.6 L38 13 L40 10.4 L41 7 L40 3.6 Z M36 5 H34 V7.6 H36 Z"
        fill-rule="evenodd"
      />
      <circle cx="21.6" cy="14.6" r="4.1" fill="none" stroke="currentColor" stroke-width="2.4" />
      <path d="M7 18 H11.2 L13 22.4 L12.2 25.4 L10.6 25 L8.6 21 Z M36 18 H31.8 L30 22.4 L30.8 25.4 L32.4 25 L34.4 21 Z" />
      <path
        d="M17 23 L19.6 24.6 L21.5 25.4 L23.4 24.6 L26 23 L28.6 25 L29.6 28 L28.6 30.4 L27 32.4 L25.6 34 L21.5 35.6 L17.4 34 L16 32.4 L14.4 30.4 L13.4 28 L14.4 25 Z M20.6 29.4 H22.4 V32.6 H20.6 Z"
        fill-rule="evenodd"
      />
    </svg>
  );
}

/** Deathmatch and Team Deathmatch: a ring in an octagon (traced at 44 x 36). */
export function DeathmatchGlyph(props: { size?: number }) {
  const width = () => props.size ?? 40;
  return (
    <svg width={width()} height={(width() * 36) / 44} viewBox="0 0 44 36" fill="currentColor" aria-hidden="true">
      <path d="M14.4 1.6 H29.6 L40.4 11.4 V24.6 L29.6 34.4 H14.4 L3.6 24.6 V11.4 Z" fill="none" stroke="currentColor" stroke-width="2.6" />
      <circle cx="22.5" cy="17" r="7" fill="none" stroke="currentColor" stroke-width="3.2" />
      <path d="M7 13 Q13.4 17.2 7 21.4 Z M38 13 Q31.6 17.2 38 21.4 Z" />
      <path d="M19.5 3.5 H25.5 L24.5 6.5 H20.5 Z M20.5 28.5 H24.5 L25.5 31.5 H19.5 Z" />
    </svg>
  );
}
