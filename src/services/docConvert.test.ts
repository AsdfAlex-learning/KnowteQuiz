import { afterEach, describe, expect, it, vi } from 'vitest';
import { convertDocument, getDocumentStatus } from './docConvert';

vi.mock('./tauri', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
}));

const outcome = {
  source_path: '/vault/design.docx',
  output_path: '/vault/design.docx.md',
  image_dir: '/vault/design.docx.images',
  title: 'Design Review',
  image_count: 2,
  warnings: ['PDF conversion is text-only'],
};

const status = {
  source_path: '/vault/design.docx',
  converted: true,
  stale: false,
  output_path: '/vault/design.docx.md',
};

function okJson(body: unknown) {
  return {
    ok: true,
    status: 200,
    text: async () => JSON.stringify(body),
    json: async () => body,
  } as unknown as Response;
}

describe('docConvert service', () => {
  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it('posts the source path to the convert endpoint', async () => {
    const fetchMock = vi.fn(async () => okJson(outcome));
    vi.stubGlobal('fetch', fetchMock);

    const result = await convertDocument('/vault/design.docx');

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/documents/convert');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body as string)).toEqual({ source_path: '/vault/design.docx' });
    expect(result.image_count).toBe(2);
    expect(result.warnings).toHaveLength(1);
  });

  it('encodes the path in the status query', async () => {
    const fetchMock = vi.fn(async () => okJson(status));
    vi.stubGlobal('fetch', fetchMock);

    const result = await getDocumentStatus('/vault/my notes/design.docx');

    const [url] = fetchMock.mock.calls[0] as unknown as [string];
    expect(url).toContain('/api/documents/status?');
    expect(url).toContain('design.docx');
    expect(result.converted).toBe(true);
    expect(result.stale).toBe(false);
  });

  it('throws the HTTP error body when conversion fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 400,
        text: async () => 'Unsupported document type: png',
      })) as unknown as typeof fetch
    );

    await expect(convertDocument('/vault/image.png')).rejects.toThrow('HTTP 400: Unsupported document type: png');
  });
});
