/**
 * Canvas glue — implements the `tairitsu-browser:canvas` WIT import interfaces.
 */

import { getCanvasElement } from "./handle-table.js";

// ---------------------------------------------------------------------------
// Context handle table
// ---------------------------------------------------------------------------

let _nextCtxHandle = 1n;
const _contexts = new Map<bigint, CanvasRenderingContext2D>();

function getCtx(handle: bigint): CanvasRenderingContext2D {
  const ctx = _contexts.get(handle);
  if (!ctx) throw new Error(`Canvas 2D context handle ${handle} not found`);
  return ctx;
}

// ---------------------------------------------------------------------------
// WIT interface: canvas2d
// ---------------------------------------------------------------------------

export function getContext(canvas: bigint): bigint {
  const el = getCanvasElement(canvas);
  const ctx = el.getContext("2d");
  if (!ctx) throw new Error("Failed to obtain 2D rendering context");
  const handle = _nextCtxHandle++;
  _contexts.set(handle, ctx);
  return handle;
}

export function save(ctx: bigint): void {
  getCtx(ctx).save();
}

export function restore(ctx: bigint): void {
  getCtx(ctx).restore();
}

export function translate(ctx: bigint, x: number, y: number): void {
  getCtx(ctx).translate(x, y);
}

export function rotate(ctx: bigint, angle: number): void {
  getCtx(ctx).rotate(angle);
}

export function scale(ctx: bigint, x: number, y: number): void {
  getCtx(ctx).scale(x, y);
}

export function resetTransform(ctx: bigint): void {
  getCtx(ctx).resetTransform();
}

export function setFillStyle(ctx: bigint, color: string): void {
  getCtx(ctx).fillStyle = color;
}

export function setStrokeStyle(ctx: bigint, color: string): void {
  getCtx(ctx).strokeStyle = color;
}

export function setLineWidth(ctx: bigint, width: number): void {
  getCtx(ctx).lineWidth = width;
}

export type LineCap = "butt" | "round" | "square";
export type LineJoin = "bevel" | "round" | "miter";

export function setLineCap(ctx: bigint, cap: LineCap): void {
  getCtx(ctx).lineCap = cap;
}

export function setLineJoin(ctx: bigint, join: LineJoin): void {
  getCtx(ctx).lineJoin = join;
}

export function setGlobalAlpha(ctx: bigint, alpha: number): void {
  getCtx(ctx).globalAlpha = alpha;
}

export function fillRect(
  ctx: bigint,
  x: number,
  y: number,
  w: number,
  h: number,
): void {
  getCtx(ctx).fillRect(x, y, w, h);
}

export function strokeRect(
  ctx: bigint,
  x: number,
  y: number,
  w: number,
  h: number,
): void {
  getCtx(ctx).strokeRect(x, y, w, h);
}

export function clearRect(
  ctx: bigint,
  x: number,
  y: number,
  w: number,
  h: number,
): void {
  getCtx(ctx).clearRect(x, y, w, h);
}

export function beginPath(ctx: bigint): void {
  getCtx(ctx).beginPath();
}

export function closePath(ctx: bigint): void {
  getCtx(ctx).closePath();
}

export function moveTo(ctx: bigint, x: number, y: number): void {
  getCtx(ctx).moveTo(x, y);
}

export function lineTo(ctx: bigint, x: number, y: number): void {
  getCtx(ctx).lineTo(x, y);
}

export function arc(
  ctx: bigint,
  x: number,
  y: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  anticlockwise: boolean,
): void {
  getCtx(ctx).arc(x, y, radius, startAngle, endAngle, anticlockwise);
}

export function bezierCurveTo(
  ctx: bigint,
  cp1x: number,
  cp1y: number,
  cp2x: number,
  cp2y: number,
  x: number,
  y: number,
): void {
  getCtx(ctx).bezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y);
}

export function fill(ctx: bigint): void {
  getCtx(ctx).fill();
}

export function stroke(ctx: bigint): void {
  getCtx(ctx).stroke();
}

export function setFont(ctx: bigint, font: string): void {
  getCtx(ctx).font = font;
}

export function fillText(
  ctx: bigint,
  text: string,
  x: number,
  y: number,
): void {
  getCtx(ctx).fillText(text, x, y);
}

export function strokeText(
  ctx: bigint,
  text: string,
  x: number,
  y: number,
): void {
  getCtx(ctx).strokeText(text, x, y);
}
