<template>
  <div class="h-screen w-screen flex flex-col bg-[var(--bg-base)] text-[var(--text-primary)] overflow-hidden">
    <TitleBar>
      <template #center>
        {{ readerStore.currentNote?.title || '' }}
      </template>
    </TitleBar>
    <div class="flex flex-1 overflow-hidden">
      <!-- Left panel -->
      <aside
        v-show="layoutStore.leftPanelOpen"
        class="flex-shrink-0 overflow-auto bg-[var(--bg-sidebar)] border-r border-[var(--border-default)]"
        :style="{ width: layoutStore.explorerWidth + 'px' }"
      >
        <FolderSelector />
        <FileTree />
      </aside>
      <div
        v-show="layoutStore.leftPanelOpen"
        class="w-1 cursor-col-resize bg-[var(--bg-active)] hover:bg-[var(--accent-purple)] flex-shrink-0"
        @mousedown="startDragLeft"
      />

      <!-- Center panel -->
      <main
        ref="readerMain"
        class="flex-1 min-w-[300px] overflow-auto bg-[var(--bg-base)]"
        @scroll="onReaderScroll"
      >
        <MarkdownRenderer v-if="readerStore.currentNote" :content="readerStore.currentNote.content" />
        <EmptyState v-else />
      </main>

      <!-- Right panel -->
      <div
        v-show="layoutStore.rightPanelOpen"
        class="w-1 cursor-col-resize bg-[var(--bg-active)] hover:bg-[var(--accent-purple)] flex-shrink-0"
        @mousedown="startDragRight"
      />
      <aside
        v-show="layoutStore.rightPanelOpen"
        class="flex-shrink-0 overflow-auto bg-[var(--bg-sidebar)] border-l border-[var(--border-default)]"
        :style="{ width: layoutStore.readerWidth + 'px' }"
      >
        <PanelContainer />
      </aside>
    </div>
    <StatusBar />
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useLayoutStore } from './stores/layout'
import { useReaderStore } from './stores/reader'
import { useExplorerStore } from './stores/explorer'
import { useSettingsStore } from './stores/settings'
import TitleBar from './components/Layout/TitleBar.vue'
import StatusBar from './components/Layout/StatusBar.vue'
import FolderSelector from './components/Explorer/FolderSelector.vue'
import FileTree from './components/Explorer/FileTree.vue'
import MarkdownRenderer from './components/Reader/MarkdownRenderer.vue'
import EmptyState from './components/Reader/EmptyState.vue'
import PanelContainer from './components/Panel/PanelContainer.vue'

const layoutStore = useLayoutStore()
const readerStore = useReaderStore()
const explorerStore = useExplorerStore()
const settingsStore = useSettingsStore()
const readerMain = ref<HTMLElement | null>(null)

let pendingScrollSave: { path: string; top: number } | null = null
let scrollSaveTimer: ReturnType<typeof setTimeout> | null = null

function startDragLeft(e: MouseEvent) {
  e.preventDefault()
  const startX = e.clientX
  const startWidth = layoutStore.explorerWidth
  function onMove(e: MouseEvent) {
    const delta = e.clientX - startX
    layoutStore.setExplorerWidth(Math.max(150, Math.min(500, startWidth + delta)))
  }
  function onUp() {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    layoutStore.persistLayout()
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

function startDragRight(e: MouseEvent) {
  e.preventDefault()
  const startX = e.clientX
  const startWidth = layoutStore.readerWidth
  function onMove(e: MouseEvent) {
    const delta = startX - e.clientX
    layoutStore.setReaderWidth(Math.max(200, Math.min(600, startWidth + delta)))
  }
  function onUp() {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    layoutStore.persistLayout()
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

function onKeyDown(e: KeyboardEvent) {
  if (e.ctrlKey && e.key === 'b') {
    e.preventDefault()
    if (e.shiftKey) {
      layoutStore.toggleRightPanel()
    } else {
      layoutStore.toggleLeftPanel()
    }
  }
}

function onReaderScroll() {
  const path = readerStore.currentNote?.path
  const top = readerMain.value?.scrollTop ?? 0
  if (!path) return

  readerStore.scrollTop = top
  pendingScrollSave = { path, top }
  if (scrollSaveTimer) clearTimeout(scrollSaveTimer)
  scrollSaveTimer = setTimeout(() => {
    void flushReaderScrollSave()
  }, 500)
}

async function flushReaderScrollSave() {
  if (scrollSaveTimer) {
    clearTimeout(scrollSaveTimer)
    scrollSaveTimer = null
  }
  const pending = pendingScrollSave
  pendingScrollSave = null
  if (!pending) return

  await readerStore.saveScrollPosition(pending.path, pending.top)
}

async function restoreReaderScroll() {
  await nextTick()
  requestAnimationFrame(() => {
    if (readerMain.value) {
      readerMain.value.scrollTop = readerStore.scrollTop
    }
  })
}

watch(() => explorerStore.selectedPath, async (path) => {
  await flushReaderScrollSave()
  if (path) {
    await readerStore.loadNote(path)
  }
})

watch(() => readerStore.currentNote?.path, (path) => {
  if (path) {
    void restoreReaderScroll()
  }
})

onMounted(async () => {
  layoutStore.loadLayout()
  await settingsStore.loadSettings()
  await explorerStore.restoreWorkspace()
  document.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeyDown)
  void flushReaderScrollSave()
})
</script>
