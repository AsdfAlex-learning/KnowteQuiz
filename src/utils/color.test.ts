import { describe, expect, it } from 'vitest';
import { withAlpha } from './color';

describe('withAlpha', () => {
  it('converts #rrggbb to rgba', () => {
    expect(withAlpha('#1e1e2e', 0.5)).toBe('rgba(30, 30, 46, 0.5)');
  });

  it('converts #rgb to rgba', () => {
    expect(withAlpha('#abc', 0.25)).toBe('rgba(170, 187, 204, 0.25)');
  });

  it('handles mixed case hex', () => {
    expect(withAlpha('#AbCdEf', 0.75)).toBe('rgba(171, 205, 239, 0.75)');
  });

  it('returns non-hex input unchanged', () => {
    expect(withAlpha('red', 0.5)).toBe('red');
    expect(withAlpha('rgb(0,0,0)', 0.5)).toBe('rgb(0,0,0)');
    expect(withAlpha('', 0.5)).toBe('');
  });
});
