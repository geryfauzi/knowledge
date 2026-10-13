import { EditorView } from '@codemirror/view';
import { Extension } from '@codemirror/state';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';

export const zedDarkTheme = EditorView.theme(
  {
    '&': {
      color: '#f4f4f5',
      backgroundColor: '#18181b',
      fontSize: '13.5px',
      height: '100%'
    },
    '.cm-content': {
      caretColor: '#38bdf8',
      fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
      padding: '16px 24px',
      maxWidth: '900px',
      margin: '0 auto',
      wordBreak: 'break-word',
      overflowWrap: 'break-word'
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#38bdf8',
      borderLeftWidth: '2px'
    },
    '&.cm-focused .cm-selectionBackground, ::selection': {
      backgroundColor: 'rgba(56, 189, 248, 0.22)'
    },
    '.cm-selectionMatch': {
      backgroundColor: 'rgba(255, 255, 255, 0.08)'
    },
    '.cm-gutters': {
      backgroundColor: '#18181b',
      color: '#52525b',
      border: 'none',
      borderRight: '1px solid #222226',
      paddingLeft: '8px',
      paddingRight: '8px'
    },
    '.cm-activeLineGutter': {
      color: '#f4f4f5',
      backgroundColor: 'transparent',
      fontWeight: '600'
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(255, 255, 255, 0.025)'
    },
    '.cm-foldPlaceholder': {
      backgroundColor: '#27272a',
      border: 'none',
      color: '#a1a1aa',
      borderRadius: '3px',
      padding: '0 4px'
    },
    '.cm-tooltip': {
      border: '1px solid #27272a',
      backgroundColor: '#1c1c20',
      borderRadius: '6px',
      boxShadow: '0 8px 24px rgba(0, 0, 0, 0.5)'
    },
    '.cm-tooltip-autocomplete': {
      '& > ul > li[aria-selected]': {
        backgroundColor: '#27272a',
        color: '#f4f4f5'
      }
    }
  },
  { dark: true }
);

export const zedHighlightStyle = HighlightStyle.define([
  { tag: t.heading1, color: '#f4f4f5', fontWeight: '700', fontSize: '1.4em' },
  { tag: t.heading2, color: '#e4e4e7', fontWeight: '600', fontSize: '1.25em' },
  { tag: t.heading3, color: '#e4e4e7', fontWeight: '600', fontSize: '1.15em' },
  { tag: t.heading, color: '#e4e4e7', fontWeight: '600' },
  { tag: t.keyword, color: '#c084fc' }, // Zed Purple
  { tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName], color: '#f4f4f5' },
  { tag: [t.function(t.variableName), t.labelName], color: '#60a5fa' }, // Blue
  { tag: [t.color, t.constant(t.name), t.standard(t.name)], color: '#f59e0b' },
  { tag: [t.definition(t.name), t.separator], color: '#f4f4f5' },
  { tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace], color: '#fb923c' },
  { tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)], color: '#38bdf8' },
  { tag: [t.meta, t.comment], color: '#71717a', fontStyle: 'italic' },
  { tag: t.strong, fontWeight: '700', color: '#f4f4f5' },
  { tag: t.emphasis, fontStyle: 'italic', color: '#e4e4e7' },
  { tag: t.strikethrough, textDecoration: 'line-through', color: '#71717a' },
  { tag: t.link, color: '#38bdf8', textDecoration: 'underline' },
  { tag: t.string, color: '#4ade80' }, // Green
  { tag: t.monospace, color: '#e2e8f0', backgroundColor: 'rgba(255, 255, 255, 0.06)', borderRadius: '3px', padding: '1px 3px' }
]);

export const zedThemeExtension: Extension = [
  zedDarkTheme,
  syntaxHighlighting(zedHighlightStyle)
];
