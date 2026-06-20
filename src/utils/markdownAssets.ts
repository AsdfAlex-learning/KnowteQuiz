import type MarkdownIt from 'markdown-it'

type AssetUrlFactory = (absolutePath: string) => string

const EXTERNAL_URL_PATTERN = /^[a-z][a-z0-9+.-]*:/i

export function resolveMarkdownAssetUrl(
  src: string,
  notePath: string,
  toAssetUrl: AssetUrlFactory,
): string {
  if (!src || isPassthroughUrl(src) || isAbsolutePath(src) || !notePath) {
    return src
  }

  const { path, suffix } = splitUrlSuffix(src)
  const noteDir = dirname(notePath)
  const absolutePath = normalizePath(joinPath(noteDir, decodeURIPath(path)))
  return `${toAssetUrl(absolutePath)}${suffix}`
}

export function markdownWebAssetUrl(absolutePath: string): string {
  return `/api/notes/asset?path=${encodeURIComponent(absolutePath)}`
}

export function configureMarkdownAssetRenderer(
  md: MarkdownIt,
  getNotePath: () => string | undefined,
  toAssetUrl: AssetUrlFactory,
): void {
  const defaultImageRenderer = md.renderer.rules.image

  md.renderer.rules.image = (tokens, idx, options, env, self) => {
    const token = tokens[idx]
    const srcIndex = token.attrIndex('src')
    const notePath = getNotePath()

    if (srcIndex >= 0 && notePath && token.attrs) {
      const src = token.attrs[srcIndex][1]
      token.attrs[srcIndex][1] = resolveMarkdownAssetUrl(src, notePath, toAssetUrl)
    }

    return defaultImageRenderer
      ? defaultImageRenderer(tokens, idx, options, env, self)
      : self.renderToken(tokens, idx, options)
  }
}

function isPassthroughUrl(src: string): boolean {
  return src.startsWith('#')
    || src.startsWith('//')
    || EXTERNAL_URL_PATTERN.test(src)
}

function isAbsolutePath(src: string): boolean {
  return src.startsWith('/')
    || src.startsWith('\\')
    || /^[a-zA-Z]:[\\/]/.test(src)
}

function splitUrlSuffix(src: string): { path: string; suffix: string } {
  const queryIndex = src.search(/[?#]/)
  if (queryIndex === -1) {
    return { path: src, suffix: '' }
  }
  return {
    path: src.slice(0, queryIndex),
    suffix: src.slice(queryIndex),
  }
}

function decodeURIPath(path: string): string {
  try {
    return decodeURIComponent(path)
  } catch {
    return path
  }
}

function dirname(path: string): string {
  const normalized = path.replace(/\\/g, '/')
  const index = normalized.lastIndexOf('/')
  if (index <= 0) return index === 0 ? '/' : ''
  return normalized.slice(0, index)
}

function joinPath(base: string, relative: string): string {
  if (!base) return relative
  return `${base.replace(/\/+$/, '')}/${relative.replace(/^\/+/, '')}`
}

function normalizePath(path: string): string {
  const usesWindowsDrive = /^[a-zA-Z]:\//.test(path)
  const prefix = usesWindowsDrive ? path.slice(0, 2) : path.startsWith('/') ? '/' : ''
  const rest = usesWindowsDrive ? path.slice(3) : path.replace(/^\/+/, '')
  const parts: string[] = []

  for (const part of rest.split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      parts.pop()
    } else {
      parts.push(part)
    }
  }

  if (usesWindowsDrive) return `${prefix}/${parts.join('/')}`
  return `${prefix}${parts.join('/')}`
}
