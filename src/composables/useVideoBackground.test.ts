import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  useVideoBackground,
  registerVideoControls,
  requestStop,
  requestResume,
  resetVideoState,
} from './useVideoBackground';

describe('useVideoBackground', () => {
  beforeEach(() => {
    resetVideoState();
    registerVideoControls(null);
  });

  it('requestStop calls controls.stop when registered', () => {
    const stop = vi.fn();
    registerVideoControls({
      play: vi.fn(),
      pause: vi.fn(),
      stop,
      resume: vi.fn(),
    });

    requestStop();

    expect(stop).toHaveBeenCalledOnce();
  });

  it('requestStop does nothing when controls are null', () => {
    expect(() => requestStop()).not.toThrow();
  });

  it('requestResume clears isStopped and screenshotUrl', () => {
    const { isStopped, screenshotUrl } = useVideoBackground();
    isStopped.value = true;
    screenshotUrl.value = 'data:image/png;base64,test';

    requestResume();

    expect(isStopped.value).toBe(false);
    expect(screenshotUrl.value).toBeNull();
  });

  it('registerVideoControls(null) nulls controls', () => {
    const { controls } = useVideoBackground();
    registerVideoControls({
      play: vi.fn(),
      pause: vi.fn(),
      stop: vi.fn(),
      resume: vi.fn(),
    });
    expect(controls.value).not.toBeNull();

    registerVideoControls(null);
    expect(controls.value).toBeNull();
  });

  it('resetVideoState clears isStopped and screenshotUrl', () => {
    const { isStopped, screenshotUrl } = useVideoBackground();
    isStopped.value = true;
    screenshotUrl.value = 'data:image/png;base64,test';

    resetVideoState();

    expect(isStopped.value).toBe(false);
    expect(screenshotUrl.value).toBeNull();
  });
});
