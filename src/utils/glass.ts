export const GLASS_BLUR_MIN = 0;
export const GLASS_BLUR_MAX = 60;
export const GLASS_BLUR_DEFAULT = 40;

/**
 * Clamp a glassmorphism blur radius to the supported 0-60 px range.
 * The glass reads as frosted material from ~40 px; below that it looks like a
 * tinted rectangle.
 */
export function clampBlur(value: number | string | null | undefined): number {
  if (value === null || value === undefined) return GLASS_BLUR_DEFAULT;
  const parsed = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(parsed)) return GLASS_BLUR_DEFAULT;
  return Math.min(GLASS_BLUR_MAX, Math.max(GLASS_BLUR_MIN, Math.round(parsed)));
}
