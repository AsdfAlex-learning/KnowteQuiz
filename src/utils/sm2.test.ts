import { describe, it, expect } from 'vitest';
import { sm2Update } from './sm2';

describe('sm2Update', () => {
  it('Again (0) resets interval to 1 and drops ease factor', () => {
    const result = sm2Update(2.5, 3, 0);
    expect(result.easeFactor).toBeCloseTo(2.3, 5);
    expect(result.intervalDays).toBe(1);
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Hard (1) slightly increases interval and drops ease factor', () => {
    const result = sm2Update(2.5, 3, 1);
    expect(result.easeFactor).toBeCloseTo(2.35, 5);
    expect(result.intervalDays).toBe(4); // round(3 * 1.2) = 4
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Good (2) with interval 0 sets interval to 1', () => {
    const result = sm2Update(2.5, 0, 2);
    expect(result.easeFactor).toBe(2.5);
    expect(result.intervalDays).toBe(1);
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Good (2) with existing interval multiplies by ease factor', () => {
    const result = sm2Update(2.5, 3, 2);
    expect(result.easeFactor).toBe(2.5);
    expect(result.intervalDays).toBe(8); // round(3 * 2.5) = 8
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Easy (3) with interval 0 sets interval to 4', () => {
    const result = sm2Update(2.5, 0, 3);
    expect(result.easeFactor).toBe(2.65);
    expect(result.intervalDays).toBe(4);
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Easy (3) with existing interval multiplies by ef * 1.3', () => {
    const result = sm2Update(2.5, 3, 3);
    expect(result.easeFactor).toBe(2.65);
    expect(result.intervalDays).toBe(10); // round(3 * 2.5 * 1.3) = 10
    expect(result.nextReviewDate).toBeDefined();
  });

  it('Again does not let ease factor drop below 1.3', () => {
    const result = sm2Update(1.35, 10, 0);
    expect(result.easeFactor).toBe(1.3);
    expect(result.intervalDays).toBe(1);
  });

  it('Hard does not let ease factor drop below 1.3', () => {
    const result = sm2Update(1.35, 10, 1);
    expect(result.easeFactor).toBe(1.3);
    expect(result.intervalDays).toBe(12); // round(10 * 1.2) = 12
  });

  it('computes next review date as today + interval', () => {
    const today = new Date();
    const result = sm2Update(2.5, 0, 2);
    const expected = new Date(today);
    expected.setDate(expected.getDate() + 1);
    expect(result.nextReviewDate).toBe(expected.toISOString().slice(0, 10));
  });
});
