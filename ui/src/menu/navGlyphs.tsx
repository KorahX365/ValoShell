// The current client's nav glyphs, traced from 1920 x 1080 screenshots onto the 30 px box each
// icon occupies in its 79 px tab (1 unit = 1 screen pixel). All draw in currentColor.
import type { JSX } from "solid-js";

function Glyph(props: { children: JSX.Element }) {
  return (
    <svg width="30" height="30" viewBox="0 0 30 30" fill="currentColor" aria-hidden="true">
      {props.children}
    </svg>
  );
}

/** Battlepass: a ticket head with a notched corner and a square punched out, and its stub. */
export function NavPass() {
  return (
    <Glyph>
      <path
        d="M17.6 3 L20.6 5.8 V9.2 H24.6 L26.2 11.6 L16.6 21.2 L8.4 13 Z M14 10 V14 H18 V10 Z"
        fill-rule="evenodd"
      />
      <path d="M6.6 13.8 L15 22.2 L14.8 23.4 L12.4 25 L10 22.6 L9 19.4 L5.4 18.6 L3.8 16.6 Z" />
    </Glyph>
  );
}

/** Agents: a wide-brimmed hat over a masked face and a turned-up collar. */
export function NavAgents() {
  return (
    <Glyph>
      <path d="M11 4 H19 V8 H16.4 V9 H13 V8 H11 Z" />
      <path d="M4 9.4 L6.4 8.2 L9.6 8.2 L13 10.2 L17 10.2 L20.4 8.2 L23.6 8.2 L26 9.4 L22.6 10.6 L19 12.2 L10.6 12.2 L7.4 10.6 Z" />
      <path d="M8 13 L13 20 H8 Z M22 13 L17 20 H22 Z" />
      <path d="M11.6 13.6 H18.4 L15 18.8 Z" />
      <path d="M10 20 H13.6 L16 24 H12.2 Z" />
    </Glyph>
  );
}

/** Career: a medal on a split ribbon. */
export function NavCareer() {
  return (
    <Glyph>
      <path d="M6 3.2 H14 V12.6 H12.4 L6 8.8 Z M16 3.2 H24 V8.8 L17.6 12.6 H16 Z" />
      <path d="M14.6 13.9 A5.4 5.4 0 1 1 14.59 13.9 Z M14.6 16.9 A2.3 2.3 0 1 0 14.61 16.9 Z" fill-rule="evenodd" />
    </Glyph>
  );
}

/** Event: a pinwheel of bars closing on the centre (the VCT mark). */
export function NavEvent() {
  return (
    <Glyph>
      <path d="M8 2.4 H9 L16 10.6 V11 H12 L8 6.4 Z" />
      <path d="M19 2.8 H20 V7 L19 8 H15 Z" />
      <path d="M2 8 H6 L10 12 V15 L6 20 H2 L7 15 V13 Z" />
      <path d="M22 8 H26 L22 12 V16 L26 20 H22 L18 16 V12 Z" />
      <path d="M12 17 H17 L20 20.4 V25 H19.6 L14 19.6 H13.6 L8.6 25 H8 V21 Z" />
    </Glyph>
  );
}

/** Premier: two chevrons over a diamond, the top corners filled. */
export function NavPremier() {
  return (
    <Glyph>
      <path d="M3 4 H8 V6 L3.6 10 H3 Z M27 4 H22 V6 L26.4 10 H27 Z" />
      <g fill="none" stroke="currentColor" stroke-width="2.9">
        <path d="M5 13.8 L15 5 L25 13.8" />
        <path d="M4.8 21.8 L14.6 13 L24.4 21.8" />
      </g>
      <path d="M14.6 18.3 L18.1 21.8 L14.6 25.2 L11.1 21.8 Z" />
    </Glyph>
  );
}

/** Collection: a backpack with its flap, buckle, straps and side pockets. */
export function NavCollection() {
  return (
    <Glyph>
      <path d="M12.4 5.6 V3.9 H17.6 V5.6" fill="none" stroke="currentColor" stroke-width="1.4" />
      <path d="M9 6 H21 L24 7 V11 L22 11.4 L19.8 12.6 H10.2 L8 11.4 L6 11 V7 Z" />
      <path d="M12 13 H18 V15 H17 V16 H13 V15 H12 Z M8 14 H11 V16.6 H8 Z M19 14 H22 V16.6 H19 Z" />
      <path d="M8 16.6 H12 V17 H18 V16.6 H22 V24 H8 Z" />
      <path d="M4 15.8 H6 V23 H4 Z M24 15.8 H27 V23 H24 Z" />
    </Glyph>
  );
}

/** Store: a shopping cart. */
export function NavStore() {
  return (
    <Glyph>
      <path d="M4 3 H8.6 L9.2 5 L8.6 5 L8 4.6 H4.6 Z" />
      <path d="M8 4.6 L9.6 4.6 L11.4 18 H9.8 Z" />
      <path d="M11.2 6 H25.6 L24.6 13.2 H12.8 Z" />
      <path d="M9 18 H24 V19 H9 Z" />
      <path d="M9 21 H12 V24 H9 Z M21 21 H24 V24 H21 Z" />
    </Glyph>
  );
}

/** Right-hand cluster glyphs, each on the box the client draws it in (centred on y 24.5). */
function Box(props: { w: number; h: number; top: number; children: JSX.Element }) {
  return (
    <svg width={props.w} height={props.h} viewBox={`0 ${props.top} ${props.w} ${props.h}`} fill="currentColor" aria-hidden="true">
      {props.children}
    </svg>
  );
}

export function NavMail() {
  return (
    <Box w={24} h={24} top={13}>
      <rect x="4" y="18.6" width="16" height="12.8" fill="none" stroke="currentColor" stroke-width="2" />
      <path d="M5 20 L12 26.8 L19 20" fill="none" stroke="currentColor" stroke-width="2.3" />
    </Box>
  );
}

function Ring() {
  return <circle cx="14" cy="24.5" r="11.3" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.5" />;
}

/** VALORANT Points: the V in a ring. */
export function NavVp() {
  return (
    <Box w={28} h={28} top={10.5}>
      <Ring />
      <path d="M9 21.8 H10 L15.2 29 H11 L9 26 Z M18.4 21 H19 V24 L17.4 26 H15 Z" />
    </Box>
  );
}

/** Radianite: the open R in a ring. */
export function NavRp() {
  return (
    <Box w={28} h={28} top={10.5}>
      <Ring />
      <path d="M11 20 H16 L18 22 V24 L16.4 26 L18 30 H16.2 L14.4 26.2 H13 V30 H11 V24 H15.2 V22 H11 Z" />
    </Box>
  );
}

/** Kingdom Credits: the mark in a tilted diamond. */
export function NavKc() {
  return (
    <Box w={28} h={28} top={10.5}>
      <path d="M15.5 13.4 L25.6 23.2 L11.5 35.2 L2.4 26 Z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" opacity="0.5" />
      <path d="M14 19 H15 V23 L14 24 H13 V20 Z M10 23 H11 V26 H14 L14.5 27 H10 V28 H9 V24 Z M12 25 H14 L16.4 28.8 H13 L12 27 Z" />
    </Box>
  );
}

/** Settings: a heavy ring with eight teeth and a diamond hub. */
export function NavGear() {
  const diagonal = (x: number, y: number) => `rotate(45 ${x} ${y})`;
  return (
    <Box w={38} h={38} top={5.5}>
      <circle cx="19" cy="24.5" r="9.2" fill="none" stroke="currentColor" stroke-width="3" />
      <path d="M17 9.6 H21 V14 H17 Z M17 35 H21 V39.4 H17 Z M3 22.5 H9.4 V26.5 H3 Z M28.6 22.5 H35 V26.5 H28.6 Z" />
      <rect x="8.2" y="13" width="4.4" height="6" transform={diagonal(10.4, 16)} />
      <rect x="25.4" y="13" width="4.4" height="6" transform={diagonal(27.6, 16)} />
      <rect x="8.2" y="30" width="4.4" height="6" transform={diagonal(10.4, 33)} />
      <rect x="25.4" y="30" width="4.4" height="6" transform={diagonal(27.6, 33)} />
      <path d="M19 19.8 L23.8 24.5 L19 29.2 L14.2 24.5 Z" />
    </Box>
  );
}
