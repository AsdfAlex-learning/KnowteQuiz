export function withAlpha(color: string, alpha: number): string {
  const hex = color.trim();

  const match6 = hex.match(/^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/);
  if (match6) {
    const r = parseInt(match6[1], 16);
    const g = parseInt(match6[2], 16);
    const b = parseInt(match6[3], 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  const match3 = hex.match(/^#([0-9a-fA-F])([0-9a-fA-F])([0-9a-fA-F])$/);
  if (match3) {
    const r = parseInt(match3[1] + match3[1], 16);
    const g = parseInt(match3[2] + match3[2], 16);
    const b = parseInt(match3[3] + match3[3], 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  return color;
}
