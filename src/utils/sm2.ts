// SM-2 quality ratings: 0=Again, 1=Hard, 2=Good, 3=Easy
export function sm2Update(
  easeFactor: number,
  intervalDays: number,
  quality: number
): { easeFactor: number; intervalDays: number; nextReviewDate: string } {
  let ef = easeFactor;
  let iv = intervalDays;

  if (quality === 0) {
    // Again: reset to 1 day, ease factor drops
    iv = 1;
    ef = Math.max(1.3, ef - 0.2);
  } else if (quality === 1) {
    // Hard: small interval increase, ease factor drops slightly
    iv = Math.max(1, Math.round(iv * 1.2));
    ef = Math.max(1.3, ef - 0.15);
  } else if (quality === 2) {
    // Good: normal interval, ease factor stays
    if (iv === 0) {
      iv = 1;
    } else {
      iv = Math.round(iv * ef);
    }
  } else {
    // Easy: bigger interval, ease factor increases
    if (iv === 0) {
      iv = 4;
    } else {
      iv = Math.round(iv * ef * 1.3);
    }
    ef = ef + 0.15;
  }

  const next = new Date();
  next.setDate(next.getDate() + iv);
  const nextReviewDate = next.toISOString().slice(0, 10);

  return { easeFactor: ef, intervalDays: iv, nextReviewDate };
}
