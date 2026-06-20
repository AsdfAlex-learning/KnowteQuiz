import { describe, expect, it, vi } from 'vitest'
import { renderMarkdownWithFallback } from './markdown'

describe('markdown rendering safety', () => {
  it('returns rendered html when the renderer succeeds', () => {
    expect(renderMarkdownWithFallback('# Title', () => '<h1>Title</h1>')).toBe('<h1>Title</h1>')
  })

  it('falls back to escaped source text when the renderer throws', () => {
    const render = vi.fn(() => {
      throw new Error('broken plugin')
    })

    expect(renderMarkdownWithFallback('<script>alert(1)</script>', render))
      .toBe('<pre class="markdown-render-fallback"><code>&lt;script&gt;alert(1)&lt;/script&gt;</code></pre>')
  })
})
