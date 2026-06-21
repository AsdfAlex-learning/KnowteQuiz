import MarkdownIt from 'markdown-it'
import mk from '@traptitech/markdown-it-katex'
import hljs from 'highlight.js'

import 'katex/dist/katex.min.css'

export interface MarkdownOptions {
  html?: boolean
  xhtmlOut?: boolean
  breaks?: boolean
  linkify?: boolean
  typographer?: boolean
  katex?: boolean
  highlight?: boolean
}

type ResolvedMarkdownOptions = Required<MarkdownOptions>

const defaultOptions: ResolvedMarkdownOptions = {
  html: false,
  xhtmlOut: false,
  breaks: true,
  linkify: true,
  typographer: true,
  katex: true,
  highlight: true,
}

const markdownInstances = new Map<string, MarkdownIt>()

export function getMarkdown_it(options: MarkdownOptions = {}): MarkdownIt {
  const resolvedOptions = resolveMarkdownOptions(options)
  const cacheKey = JSON.stringify(resolvedOptions)
  const cached = markdownInstances.get(cacheKey)
  if (cached) return cached

  const md = new MarkdownIt({
    html: resolvedOptions.html,
    xhtmlOut: resolvedOptions.xhtmlOut,
    breaks: resolvedOptions.breaks,
    linkify: resolvedOptions.linkify,
    typographer: resolvedOptions.typographer,
    highlight: resolvedOptions.highlight
      ? (str: string, lang: string) => {
          const langLower = lang.toLowerCase()
          if (langLower && hljs.getLanguage(langLower)) {
            try {
              return hljs.highlight(str, { language: langLower }).value
            } catch {
              // fallback to plain text
            }
          }
          return ''
        }
      : undefined,
  })

  if (resolvedOptions.katex) {
    md.use(mk, {
      throwOnError: false,
      errorColor: '#cc0000',
    })
  }

  markdownInstances.set(cacheKey, md)
  return md
}

export function renderMarkdown(content: string, options?: MarkdownOptions): string {
  const md = getMarkdown_it(options)
  return renderMarkdownWithFallback(content, (source) => md.render(source))
}

export function renderInline(content: string, options?: MarkdownOptions): string {
  const md = getMarkdown_it(options)
  return md.renderInline(content)
}

export function renderMarkdownWithFallback(
  content: string,
  render: (source: string) => string,
): string {
  try {
    return render(content)
  } catch {
    return `<pre class="markdown-render-fallback"><code>${escapeHtml(content)}</code></pre>`
  }
}

export function highlightCode(code: string, language: string): string {
  const langLower = language.toLowerCase()
  if (langLower && hljs.getLanguage(langLower)) {
    try {
      return hljs.highlight(code, { language: langLower }).value
    } catch {
      // fallback
    }
  }
  return code
}

export function extractText(content: string): string {
  const md = getMarkdown_it({ katex: false, highlight: false })
  const html = md.render(content)
  return html.replace(/<[^>]+>/g, '').replace(/</g, '<').replace(/>/g, '>').replace(/&/g, '&').replace(/"/g, '"')
}

export function splitByParagraph(content: string): string[] {
  return content.split(/\n\n+/).filter(Boolean)
}

export interface TocHeading {
  level: number
  text: string
  id: string
}

export function extractHeadings(content: string): TocHeading[] {
  const headingRegex = /^(#{1,6})\s+(.+)$/gm
  const headings: TocHeading[] = []
  let match: RegExpExecArray | null

  while ((match = headingRegex.exec(content)) !== null) {
    const level = match[1].length
    const text = match[2].trim()
    const id = headingId(text)
    headings.push({ level, text, id })
  }

  return headings
}

function headingId(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\u4e00-\u9fff\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}

function escapeHtml(content: string): string {
  return content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function resolveMarkdownOptions(options: MarkdownOptions): ResolvedMarkdownOptions {
  return {
    html: options.html ?? defaultOptions.html,
    xhtmlOut: options.xhtmlOut ?? defaultOptions.xhtmlOut,
    breaks: options.breaks ?? defaultOptions.breaks,
    linkify: options.linkify ?? defaultOptions.linkify,
    typographer: options.typographer ?? defaultOptions.typographer,
    katex: options.katex ?? defaultOptions.katex,
    highlight: options.highlight ?? defaultOptions.highlight,
  }
}
