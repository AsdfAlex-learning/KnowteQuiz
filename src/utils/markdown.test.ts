import { describe, expect, it, vi } from 'vitest';
import { extractHeadings, renderMarkdown, renderMarkdownWithFallback } from './markdown';

describe('markdown rendering safety', () => {
  it('returns rendered html when the renderer succeeds', () => {
    expect(renderMarkdownWithFallback('# Title', () => '<h1>Title</h1>')).toBe('<h1>Title</h1>');
  });

  it('falls back to escaped source text when the renderer throws', () => {
    const render = vi.fn(() => {
      throw new Error('broken plugin');
    });

    expect(renderMarkdownWithFallback('<script>alert(1)</script>', render)).toBe(
      '<pre class="markdown-render-fallback"><code>&lt;script&gt;alert(1)&lt;/script&gt;</code></pre>'
    );
  });

  it('keeps MarkdownIt instances separate for different option sets', () => {
    expect(renderMarkdown('<b>raw</b>', { html: false, katex: false, highlight: false })).toContain(
      '&lt;b&gt;raw&lt;/b&gt;'
    );
    expect(renderMarkdown('<b>raw</b>', { html: true, katex: false, highlight: false })).toContain('<b>raw</b>');
  });
});

describe('extract headings for TOC', () => {
  it('extracts headings with correct levels and generated ids', () => {
    const content = ['# Introduction', 'Some text', '## Getting Started', '### Installation', '## Conclusion'].join(
      '\n'
    );

    const headings = extractHeadings(content);

    expect(headings).toHaveLength(4);
    expect(headings[0]).toEqual({ level: 1, text: 'Introduction', id: 'introduction' });
    expect(headings[1]).toEqual({ level: 2, text: 'Getting Started', id: 'getting-started' });
    expect(headings[2]).toEqual({ level: 3, text: 'Installation', id: 'installation' });
    expect(headings[3]).toEqual({ level: 2, text: 'Conclusion', id: 'conclusion' });
  });

  it('returns empty array for content without headings', () => {
    expect(extractHeadings('Just some text\nno headings here')).toEqual([]);
  });

  it('handles Chinese headings and generates ids', () => {
    const content = '## 入门指南\n### 安装步骤';
    const headings = extractHeadings(content);

    expect(headings).toHaveLength(2);
    expect(headings[0].id).toBe('入门指南');
    expect(headings[1].id).toBe('安装步骤');
  });

  it('strips special characters from heading ids', () => {
    const content = '## What is Rust?';
    const headings = extractHeadings(content);

    expect(headings[0].id).toBe('what-is-rust');
  });

  it('ignores heading-like lines inside fenced code blocks', () => {
    const content = [
      '# Real Title',
      '```python',
      '# Not a heading',
      'print("hello")',
      '```',
      '## Real Section',
      '~~~sh',
      '### Also not a heading',
      '~~~',
    ].join('\n');

    expect(extractHeadings(content)).toEqual([
      { level: 1, text: 'Real Title', id: 'real-title' },
      { level: 2, text: 'Real Section', id: 'real-section' },
    ]);
  });

  it('allows up to three leading spaces for headings but ignores indented code', () => {
    const content = ['   ## Valid Heading', '    # Indented code'].join('\n');

    expect(extractHeadings(content)).toEqual([{ level: 2, text: 'Valid Heading', id: 'valid-heading' }]);
  });

  it('generates stable unique ids for duplicate headings', () => {
    const content = ['## Overview', 'Some text', '## Overview', '### Overview'].join('\n');

    expect(extractHeadings(content)).toEqual([
      { level: 2, text: 'Overview', id: 'overview' },
      { level: 2, text: 'Overview', id: 'overview-2' },
      { level: 3, text: 'Overview', id: 'overview-3' },
    ]);
  });
});
