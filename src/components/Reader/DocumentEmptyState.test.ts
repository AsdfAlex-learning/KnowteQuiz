import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import DocumentEmptyState from './DocumentEmptyState.vue';
import * as docConvert from '@/services/docConvert';
import { useReaderStore } from '@/stores/reader';
import type { DocumentStatus } from '@/types/docConvert';

vi.mock('@/services/docConvert', () => ({
  convertDocument: vi.fn(),
  getDocumentStatus: vi.fn(),
}));

function status(overrides: Partial<DocumentStatus> = {}): DocumentStatus {
  return {
    source_path: '/vault/design.docx',
    converted: false,
    stale: false,
    output_path: null,
    ...overrides,
  };
}

describe('DocumentEmptyState', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(docConvert.getDocumentStatus).mockResolvedValue(status());
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  function mountPanel() {
    // The component reads the document from the reader store, so seed the same
    // Pinia instance the component will use.
    const pinia = createPinia();
    setActivePinia(pinia);
    const reader = useReaderStore();
    reader.openDocumentPath('/vault/design.docx');
    return mount(DocumentEmptyState, { global: { plugins: [pinia] } });
  }

  it('shows the file name and a convert button for an unconverted document', async () => {
    const wrapper = mountPanel();

    await vi.waitFor(() => expect(wrapper.text()).toContain('design.docx'));
    expect(docConvert.getDocumentStatus).toHaveBeenCalledWith('/vault/design.docx');
    expect(wrapper.find('button').text()).toContain('Convert to Markdown');
  });

  it('converts and then offers to open the note', async () => {
    vi.mocked(docConvert.getDocumentStatus).mockResolvedValue(
      status({ converted: true, output_path: '/vault/design.docx.md' })
    );
    vi.mocked(docConvert.convertDocument).mockResolvedValue({
      source_path: '/vault/design.docx',
      output_path: '/vault/design.docx.md',
      image_dir: null,
      title: 'Design Review',
      image_count: 0,
      warnings: [],
    });

    const wrapper = mountPanel();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Open note'));

    expect(wrapper.text()).not.toContain('stale');
  });

  it('warns and offers re-convert when the note is stale', async () => {
    vi.mocked(docConvert.getDocumentStatus).mockResolvedValue(
      status({ converted: true, stale: true, output_path: '/vault/design.docx.md' })
    );

    const wrapper = mountPanel();
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('changed after it was converted');
    });
    expect(wrapper.text()).toContain('Re-convert');
  });

  it('shows conversion warnings returned by the backend', async () => {
    vi.mocked(docConvert.convertDocument).mockResolvedValue({
      source_path: '/vault/design.docx',
      output_path: '/vault/design.docx.md',
      image_dir: null,
      title: null,
      image_count: 0,
      warnings: ['PDF conversion is text-only'],
    });

    const wrapper = mountPanel();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Convert to Markdown'));

    await wrapper.get('button').trigger('click');

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('PDF conversion is text-only');
    });
  });

  it('surfaces a failed conversion in the error line', async () => {
    vi.mocked(docConvert.convertDocument).mockRejectedValue(
      new Error('Not a valid OOXML package: missing word/document.xml')
    );

    const wrapper = mountPanel();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Convert to Markdown'));
    await wrapper.get('button').trigger('click');

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Not a valid OOXML package');
    });
  });
});
