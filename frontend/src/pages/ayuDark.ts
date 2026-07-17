import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import type { Extension } from '@codemirror/state'

// Ayu Dark / Mirage palette (https://ayutheme.com)
const bg = '#0a0e14'
const bgAlt = '#01060e'
const fg = '#b3b1ad'
const caret = '#ffcc66'
const selection = '#2c3c5a'
const lineHighlight = '#ffffff0a'
const gutter = '#5c6773'
const gutterActive = '#aeb7c4'

const color = {
  keyword: '#ffa759',
  string: '#bae67e',
  number: '#ffcc66',
  comment: '#5c6773',
  func: '#73d0ff',
  tag: '#59c2ff',
  prop: '#73d0ff',
  constant: '#ff8f40',
  operator: '#f29e74',
  bool: '#ff8f40',
  meta: '#d4bff4',
}

const ayuDarkTheme = EditorView.theme(
  {
    '&': {
      color: fg,
      backgroundColor: bg,
      height: '100%',
    },
    '.cm-scroller': {
      fontFamily: "'Cascadia Code','Fira Code',Consolas,monospace",
      lineHeight: '1.5',
    },
    '.cm-content': { caretColor: caret },
    '.cm-cursor, .cm-dropCursor': { borderLeftColor: caret },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
      backgroundColor: selection,
    },
    '.cm-gutters': {
      backgroundColor: bgAlt,
      color: gutter,
      border: 'none',
    },
    '.cm-activeLineGutter': {
      backgroundColor: lineHighlight,
      color: gutterActive,
    },
    '.cm-activeLine': { backgroundColor: lineHighlight },
    '.cm-lineNumbers .cm-gutterElement': { padding: '0 12px 0 8px' },
    '.cm-foldGutter': { color: gutter },
    '.cm-matchingBracket, &.cm-focused .cm-matchingBracket': {
      backgroundColor: '#ffffff14',
      outline: '1px solid #ffffff20',
    },
    '.cm-selectionMatch': { backgroundColor: '#ffffff14' },
  },
  { dark: true },
)

const ayuDarkHighlight = HighlightStyle.define([
  { tag: [t.keyword, t.modifier, t.controlKeyword, t.moduleKeyword], color: color.keyword },
  { tag: [t.string, t.special(t.string), t.regexp], color: color.string },
  { tag: [t.number, t.integer, t.float], color: color.number },
  { tag: [t.bool, t.null, t.atom], color: color.bool },
  { tag: [t.comment, t.lineComment, t.blockComment, t.docComment], color: color.comment, fontStyle: 'italic' },
  { tag: [t.function(t.variableName), t.function(t.propertyName)], color: color.func },
  { tag: [t.function(t.definition(t.variableName))], color: color.func },
  { tag: [t.definition(t.variableName), t.labelName], color: color.func },
  { tag: [t.tagName], color: color.tag },
  { tag: [t.propertyName], color: color.prop },
  { tag: [t.attributeName], color: color.prop },
  { tag: [t.constant(t.name), t.constant(t.variableName), t.standard(t.name)], color: color.constant },
  { tag: [t.operator, t.operatorKeyword, t.punctuation, t.separator], color: color.operator },
  { tag: [t.meta, t.documentMeta], color: color.meta },
  { tag: [t.className, t.typeName, t.namespace], color: color.tag },
  { tag: [t.variableName], color: fg },
  { tag: [t.heading], color: color.tag, fontWeight: 'bold' },
  { tag: [t.link, t.url], color: color.func, textDecoration: 'underline' },
  { tag: [t.invalid], color: '#ff3333' },
])

export const ayuDark: Extension = [ayuDarkTheme, syntaxHighlighting(ayuDarkHighlight)]
