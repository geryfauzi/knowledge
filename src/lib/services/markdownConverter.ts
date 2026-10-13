import { marked } from 'marked';
import TurndownService from 'turndown';

const turndown = new TurndownService({
  headingStyle: 'atx',
  hr: '---',
  bulletListMarker: '-',
  codeBlockStyle: 'fenced',
  emDelimiter: '*'
});

// Configure Turndown to preserve underline or clean formatting
turndown.addRule('underline', {
  filter: ['u'],
  replacement: (content) => `<u>${content}</u>`
});

export function markdownToHtml(md: string): string {
  if (!md) return '';
  return marked.parse(md) as string;
}

export function htmlToMarkdown(html: string): string {
  if (!html) return '';
  return turndown.turndown(html);
}
