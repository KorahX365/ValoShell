// A fixed 1860 x 1032 stage (the client's screen under its 48 px bar and beside its 60 px
// social strip) scaled to fit the piece. Pages laid out on it can use the client's own pixel
// positions and still fit any window size.
import { createSignal, onCleanup, onMount, type JSX } from "solid-js";
import styles from "./valCanvas.module.css";

export const CANVAS_W = 1860;
export const CANVAS_H = 1032;

export function ValCanvas(props: { children: JSX.Element; style?: JSX.CSSProperties }) {
  let box: HTMLDivElement | undefined;
  const [scale, setScale] = createSignal(1);
  onMount(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (!entry) return;
      const { width, height } = entry.contentRect;
      setScale(Math.min(width / CANVAS_W, height / CANVAS_H));
    });
    if (box) observer.observe(box);
    onCleanup(() => observer.disconnect());
  });
  return (
    <div ref={box} class={styles.fit} style={props.style}>
      <div class={styles.canvas} style={{ transform: `scale(${scale()})` }}>
        {props.children}
      </div>
    </div>
  );
}
