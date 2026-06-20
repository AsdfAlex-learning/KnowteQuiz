import { describe, expect, it, vi } from 'vitest'
import MarkdownIt from 'markdown-it'
import { configureMarkdownAssetRenderer, resolveMarkdownAssetUrl } from './markdownAssets'

describe('markdown asset URL resolution', () => {
  it('resolves relative image paths against the current note directory', () => {
    const toAssetUrl = vi.fn((path: string) => `asset://${path}`)

    const result = resolveMarkdownAssetUrl(
      'assets/diagram.png',
      'D:/notes/rust/ownership.md',
      toAssetUrl,
    )

    expect(toAssetUrl).toHaveBeenCalledWith('D:/notes/rust/assets/diagram.png')
    expect(result).toBe('asset://D:/notes/rust/assets/diagram.png')
  })

  it('preserves external, data, anchor, and absolute image URLs', () => {
    const toAssetUrl = vi.fn((path: string) => `asset://${path}`)

    expect(resolveMarkdownAssetUrl('https://example.com/a.png', 'D:/notes/a.md', toAssetUrl)).toBe('https://example.com/a.png')
    expect(resolveMarkdownAssetUrl('data:image/png;base64,abc', 'D:/notes/a.md', toAssetUrl)).toBe('data:image/png;base64,abc')
    expect(resolveMarkdownAssetUrl('#figure-1', 'D:/notes/a.md', toAssetUrl)).toBe('#figure-1')
    expect(resolveMarkdownAssetUrl('C:/images/a.png', 'D:/notes/a.md', toAssetUrl)).toBe('C:/images/a.png')
    expect(toAssetUrl).not.toHaveBeenCalled()
  })

  it('normalizes parent directory segments without changing markdown URL query text', () => {
    const toAssetUrl = vi.fn((path: string) => `/api/notes/asset?path=${encodeURIComponent(path)}`)

    const result = resolveMarkdownAssetUrl(
      '../attachments/image 1.webp?raw=true',
      '/home/me/notes/rust/ownership.md',
      toAssetUrl,
    )

    expect(toAssetUrl).toHaveBeenCalledWith('/home/me/notes/attachments/image 1.webp')
    expect(result).toBe('/api/notes/asset?path=%2Fhome%2Fme%2Fnotes%2Fattachments%2Fimage%201.webp?raw=true')
  })

  it('rewrites relative image tokens when registered with MarkdownIt', () => {
    const md = new MarkdownIt()
    configureMarkdownAssetRenderer(
      md,
      () => 'D:/notes/rust/ownership.md',
      (path) => `asset://${path}`,
    )

    expect(md.render('![Diagram](assets/diagram.png)'))
      .toContain('src="asset://D:/notes/rust/assets/diagram.png"')
  })
})
