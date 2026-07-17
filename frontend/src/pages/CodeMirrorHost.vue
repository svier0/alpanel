<template>
  <div ref="host" class="cm-host" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { EditorState, Compartment } from '@codemirror/state'
import {
  EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, rectangularSelection, highlightSpecialChars,
} from '@codemirror/view'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput, foldGutter,
} from '@codemirror/language'
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { php } from '@codemirror/lang-php'
import { javascript } from '@codemirror/lang-javascript'
import { css } from '@codemirror/lang-css'
import { html } from '@codemirror/lang-html'
import { json } from '@codemirror/lang-json'
import { oneDark } from '@codemirror/theme-one-dark'

export interface CursorPos {
  line: number
  col: number
}

const props = defineProps<{ value: string; language: string; dark?: boolean }>()
const emit = defineEmits<{
  'update:value': [string]
  cursor: [CursorPos]
}>()

const host = ref<HTMLElement | null>(null)
let view: EditorView | null = null
const langCompartment = new Compartment()
const themeCompartment = new Compartment()

function langExtension(lang: string) {
  switch (lang) {
    case 'php': return php()
    case 'javascript': return javascript()
    case 'css': return css()
    case 'html': return html()
    case 'json': return json()
    default: return []
  }
}

function themeExtension(dark: boolean) {
  return dark ? oneDark : []
}

function buildState() {
  return EditorState.create({
    doc: props.value,
    extensions: [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightSpecialChars(),
      history(),
      foldGutter(),
      drawSelection(),
      rectangularSelection(),
      closeBrackets(),
      indentOnInput(),
      bracketMatching(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      autocompletion(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...completionKeymap,
        indentWithTab,
      ]),
      langCompartment.of(langExtension(props.language)),
      themeCompartment.of(themeExtension(props.dark ?? false)),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) {
          emit('update:value', u.state.doc.toString())
        }
        if (u.selectionSet || u.docChanged) {
          const line = u.state.doc.lineAt(u.state.selection.main.head)
          emit('cursor', { line: line.number, col: u.state.selection.main.head - line.from + 1 })
        }
      }),
      EditorView.theme({
        '&': { height: '100%', fontSize: '13px' },
        '.cm-scroller': { fontFamily: "'Cascadia Code','Fira Code',Consolas,monospace", lineHeight: '1.5' },
      }),
    ],
  })
}

onMounted(() => {
  if (!host.value) return
  view = new EditorView({ state: buildState(), parent: host.value })
})

onBeforeUnmount(() => {
  view?.destroy()
  view = null
})

watch(() => props.value, (val) => {
  if (view && val !== view.state.doc.toString()) {
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: val } })
  }
})

watch(() => props.language, (lang) => {
  view?.dispatch({ effects: langCompartment.reconfigure(langExtension(lang)) })
})

watch(() => props.dark, (d) => {
  view?.dispatch({ effects: themeCompartment.reconfigure(themeExtension(d ?? false)) })
})
</script>

<style scoped>
.cm-host {
  height: 100%;
  width: 100%;
}
.cm-host :deep(.cm-editor) {
  height: 100%;
}
.cm-host :deep(.cm-editor.cm-focused) {
  outline: none;
}
</style>
