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

const defaultOptions: MarkdownOptions = {
  html: false,
  xhtmlOut: false,
  breaks: true,
  linkify: true,
  typographer: true,
  katex: true,
  highlight: true,
}

let markdownInstance: MarkdownIt | null = null

export function getMarkdown_it(options: MarkdownOptions = defaultOptions): MarkdownIt {
  if (markdownInstance) {
    return markdownInstance
  }

  const md = new MarkdownIt({
    html: options.html ?? defaultOptions.html,
    xhtmlOut: options.xhtmlOut ?? defaultOptions.xhtmlOut,
    breaks: options.breaks ?? defaultOptions.breaks,
    linkify: options.linkify ?? defaultOptions.linkify,
    typographer: options.typographer ?? defaultOptions.typographer,
    highlight: options.highlight
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

  if (options.katex ?? defaultOptions.katex) {
    md.use(mk, {
      throwOnError: false,
      errorColor: '#cc0000',
    })
  }

  markdownInstance = md
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

function escapeHtml(content: string): string {
  return content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}
