import { describe, expect, it } from 'vitest';
import { GLASS_BLUR_DEFAULT, GLASS_BLUR_MAX, GLASS_BLUR_MIN, clampBlur } from './glass';

describe('clampBlur', () => {
  it('keeps in-range values unchanged', () => {
    expect(clampBlur(0)).toBe(0);
    expect(clampBlur(12)).toBe(12);
    expect(clampBlur(30)).toBe(30);
  });

  it('clamps out-of-range values to the bounds', () => {
    expect(clampBlur(-5)).toBe(GLASS_BLUR_MIN);
    expect(clampBlur(99)).toBe(GLASS_BLUR_MAX);
  });

  it('parses numeric strings', () => {
    expect(clampBlur('8')).toBe(8);
    expect(clampBlur('45')).toBe(GLASS_BLUR_MAX);
    expect(clampBlur('')).toBe(0);
  });

  it('rounds fractional values to integers', () => {
    expect(clampBlur(12.4)).toBe(12);
    expect(clampBlur(12.5)).toBe(13);
  });

  it('falls back to the default for missing or invalid input', () => {
    expect(clampBlur(undefined)).toBe(GLASS_BLUR_DEFAULT);
    expect(clampBlur(null)).toBe(GLASS_BLUR_DEFAULT);
    expect(clampBlur('abc')).toBe(GLASS_BLUR_DEFAULT);
  });
});
