import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanupSessions, diagnoseFollowUp, generateDiagnosisReport, generateQuiz, submitAnswerAdvanced } from './quiz';
import { webStream } from './tauri';
import type { QuizStreamParams } from '../types/quiz';

vi.mock('./tauri', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
  webStream: vi.fn(),
}));

describe('quiz service', () => {
  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it('sends quiz generation params directly to the web endpoint', async () => {
    const params: QuizStreamParams = {
      path: '/notes/rust.md',
      types: ['single'],
      count: 2,
      difficulty: 'medium',
      lang: 'zh',
    };
    vi.mocked(webStream).mockResolvedValue(undefined);

    await generateQuiz(params, vi.fn(), vi.fn(), vi.fn(), vi.fn());

    expect(webStream).toHaveBeenCalledWith('/api/quiz/generate', params, expect.any(Function));
  });

  it('sends the known correct answer to the web diagnosis endpoint', async () => {
    vi.mocked(webStream).mockResolvedValue(undefined);

    await submitAnswerAdvanced(
      'Which claim is true?',
      'B',
      'A',
      'I guessed.',
      '/notes/rust.md',
      vi.fn(),
      vi.fn(),
      vi.fn(),
      vi.fn()
    );

    expect(webStream).toHaveBeenCalledWith(
      '/api/quiz/diagnose',
      expect.objectContaining({
        question: 'Which claim is true?',
        correct_answer: 'B',
        user_answer: 'A',
      }),
      expect.any(Function)
    );
  });

  it('falls back to a local session id when crypto is unavailable in web mode', async () => {
    vi.stubGlobal('crypto', undefined);
    vi.mocked(webStream).mockResolvedValue(undefined);

    await submitAnswerAdvanced(
      'Which claim is true?',
      'B',
      'A',
      'I guessed.',
      '/notes/rust.md',
      vi.fn(),
      vi.fn(),
      vi.fn(),
      vi.fn()
    );

    expect(webStream).toHaveBeenCalledWith(
      '/api/quiz/diagnose',
      expect.objectContaining({
        session_id: expect.stringMatching(/^web-\d+-[a-z0-9]+$/),
      }),
      expect.any(Function)
    );
  });

  it('sends follow-up reply to the web diagnosis endpoint', async () => {
    vi.mocked(webStream).mockResolvedValue(undefined);

    await diagnoseFollowUp('session-123', 'I think the answer is B', vi.fn(), vi.fn(), vi.fn());

    expect(webStream).toHaveBeenCalledWith(
      '/api/quiz/diagnose/session-123/follow_up',
      { user_reply: 'I think the answer is B' },
      expect.any(Function)
    );
  });

  it('cleans up sessions through the web endpoint', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      text: async () => JSON.stringify({ deleted_count: 3, remaining_count: 2 }),
    }));
    vi.stubGlobal('fetch', fetchMock);

    const result = await cleanupSessions();

    expect(fetchMock).toHaveBeenCalledWith('/api/sessions/cleanup', { method: 'POST' });
    expect(result).toEqual({ deleted_count: 3, remaining_count: 2 });
  });

  it('includes response text when cleanup sessions fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 500,
        text: async () => 'Cleanup failed',
      }))
    );

    await expect(cleanupSessions()).rejects.toThrow('HTTP 500: Cleanup failed');
  });

  it('returns a session id from submitAnswerAdvanced in web mode', async () => {
    vi.mocked(webStream).mockResolvedValue(undefined);

    const sessionId = await submitAnswerAdvanced(
      'Q?',
      'A',
      'B',
      'reasoning',
      '/notes/test.md',
      vi.fn(),
      vi.fn(),
      vi.fn(),
      vi.fn()
    );

    expect(typeof sessionId).toBe('string');
    expect(sessionId.length).toBeGreaterThan(0);
  });

  it('includes the response body when web diagnosis report generation fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 404,
        text: async () => 'Session session-1 not found',
      }))
    );

    await expect(generateDiagnosisReport('session-1')).rejects.toThrow('HTTP 404: Session session-1 not found');
  });
});
