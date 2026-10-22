// WebGL 2 backdrop: one full-screen triangle and a theme-tinted fragment shader. Rendered at
// half resolution and at most 30 fps; it stops while the window is hidden, while a match
// is live, or when motion is reduced (one still frame). Without WebGL 2 nothing renders and
// the plain backdrop shows through.
import { createEffect, onCleanup, onMount } from "solid-js";
import { isMotionReduced } from "~/studio/spring";
import type { ShaderId } from "~/studio/types";
import { hexToRgbTriplet } from "~/studio/theme";
import { VERTEX_SOURCE, fragmentSource } from "./shaders";

const FRAME_MS = 1000 / 30;
const RESOLUTION_SCALE = 0.5;

export interface ShaderColors {
  accent: string;
  accent2: string;
  ground: string;
}

function rgb(hex: string): [number, number, number] {
  const [r = 0, g = 0, b = 0] = hexToRgbTriplet(hex).split(" ").map(Number);
  return [r / 255, g / 255, b / 255];
}

function compile(gl: WebGL2RenderingContext, type: number, source: string): WebGLShader | null {
  const shader = gl.createShader(type);
  if (!shader) return null;
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.warn("backdrop shader failed to compile", gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function link(gl: WebGL2RenderingContext, id: ShaderId): WebGLProgram | null {
  const vertex = compile(gl, gl.VERTEX_SHADER, VERTEX_SOURCE);
  const fragment = compile(gl, gl.FRAGMENT_SHADER, fragmentSource(id));
  const program = gl.createProgram();
  if (!vertex || !fragment || !program) return null;
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  gl.deleteShader(vertex);
  gl.deleteShader(fragment);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.warn("backdrop shader failed to link", gl.getProgramInfoLog(program));
    gl.deleteProgram(program);
    return null;
  }
  return program;
}

export function ShaderCanvas(props: {
  shader: ShaderId;
  colors: ShaderColors;
  isPaused: () => boolean;
  class?: string | undefined;
  onCanvas?: (canvas: HTMLCanvasElement) => void;
}) {
  let canvas: HTMLCanvasElement | undefined;
  let gl: WebGL2RenderingContext | null = null;
  let program: WebGLProgram | null = null;
  let frame = 0;
  let last = 0;
  const started = performance.now();
  let pointer: [number, number] = [0, 0];

  const resize = () => {
    if (!canvas) return;
    canvas.width = Math.max(1, Math.round(canvas.clientWidth * RESOLUTION_SCALE));
    canvas.height = Math.max(1, Math.round(canvas.clientHeight * RESOLUTION_SCALE));
    gl?.viewport(0, 0, canvas.width, canvas.height);
  };

  const draw = (now: number) => {
    if (!gl || !program || !canvas) return;
    gl.useProgram(program);
    gl.uniform2f(gl.getUniformLocation(program, "uRes"), canvas.width, canvas.height);
    gl.uniform1f(gl.getUniformLocation(program, "uTime"), (now - started) / 1000);
    gl.uniform3f(gl.getUniformLocation(program, "uAccent"), ...rgb(props.colors.accent));
    gl.uniform3f(gl.getUniformLocation(program, "uAccent2"), ...rgb(props.colors.accent2));
    gl.uniform3f(gl.getUniformLocation(program, "uGround"), ...rgb(props.colors.ground));
    gl.uniform2f(gl.getUniformLocation(program, "uPointer"), pointer[0], pointer[1]);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  };

  const loop = (now: number) => {
    frame = requestAnimationFrame(loop);
    if (document.hidden || props.isPaused() || now - last < FRAME_MS) return;
    last = now;
    draw(now);
    if (isMotionReduced()) cancelAnimationFrame(frame);
  };

  const onPointer = (event: PointerEvent) => {
    pointer = [event.clientX / window.innerWidth - 0.5, 0.5 - event.clientY / window.innerHeight];
  };

  onMount(() => {
    if (!canvas) return;
    gl = canvas.getContext("webgl2", { antialias: false, alpha: false, preserveDrawingBuffer: true });
    if (!gl) return;
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    window.addEventListener("pointermove", onPointer, { passive: true });
    props.onCanvas?.(canvas);
    onCleanup(() => {
      observer.disconnect();
      window.removeEventListener("pointermove", onPointer);
      cancelAnimationFrame(frame);
      gl?.getExtension("WEBGL_lose_context")?.loseContext();
    });
  });

  // A new shader recompiles; the loop keeps running.
  createEffect(() => {
    const id = props.shader;
    if (!gl) return;
    if (program) gl.deleteProgram(program);
    program = link(gl, id);
    if (!program) return;
    const position = gl.getAttribLocation(program, "aPos");
    gl.enableVertexAttribArray(position);
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
    resize();
    cancelAnimationFrame(frame);
    last = 0;
    frame = requestAnimationFrame(loop);
  });

  return <canvas ref={canvas} class={props.class} aria-hidden="true" />;
}
