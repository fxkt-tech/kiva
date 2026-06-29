export function clamp01(value: number): number {
  if (Number.isNaN(value)) {
    return 0;
  }

  return Math.min(1, Math.max(0, value));
}

export function easeOutCubic(progress: number): number {
  const clamped = clamp01(progress);
  return 1 - Math.pow(1 - clamped, 3);
}

export function easeInOutCubic(progress: number): number {
  const clamped = clamp01(progress);
  return clamped < 0.5
    ? 4 * clamped * clamped * clamped
    : 1 - Math.pow(-2 * clamped + 2, 3) / 2;
}

export function fadeIn(progress: number): number {
  return easeOutCubic(progress);
}

export function slideIn(progress: number, distance: number): number {
  return (1 - easeOutCubic(progress)) * distance;
}

export function pulse(progress: number, amount: number): number {
  const wave = Math.sin(clamp01(progress) * Math.PI * 2);
  return 1 + wave * amount;
}

export function zoom(progress: number, from: number, to: number): number {
  return from + (to - from) * easeOutCubic(progress);
}

export function shake(progress: number, amplitude: number): number {
  const clamped = clamp01(progress);
  const decay = 1 - clamped;
  return Math.sin(clamped * Math.PI * 10) * amplitude * decay;
}

export function stagedReveal(
  progress: number,
  index: number,
  count: number,
): number {
  if (count <= 0) {
    return 1;
  }

  const segment = 1 / count;
  const start = segment * index;
  return clamp01((clamp01(progress) - start) / segment);
}
