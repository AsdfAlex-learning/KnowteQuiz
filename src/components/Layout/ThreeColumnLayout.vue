<template>
  <div class="flex flex-col h-screen w-screen bg-[var(--bg-base)] text-[var(--text-primary)] overflow-hidden">
    <TitleBar />

    <div class="flex flex-1 overflow-hidden">
      <!-- Left panel: Explorer -->
      <div
        v-show="layoutStore.leftPanelOpen"
        class="flex-shrink-0 overflow-hidden"
        :style="{ width: layoutStore.explorerWidth + 'px' }"
      >
        <slot name="left" />
      </div>

      <!-- Left resize handle -->
      <div
        v-show="layoutStore.leftPanelOpen"
        class="w-1 cursor-col-resize bg-transparent hover:bg-[var(--accent-purple)]/30 active:bg-[var(--accent-purple)]/50 transition-colors flex-shrink-0"
        @mousedown="startResize('left', $event)"
      />

      <!-- Center panel: Reader -->
      <div class="flex-1 overflow-hidden">
        <slot name="center" />
      </div>

      <!-- Right resize handle -->
      <div
        v-show="layoutStore.rightPanelOpen"
        class="w-1 cursor-col-resize bg-transparent hover:bg-[var(--accent-purple)]/30 active:bg-[var(--accent-purple)]/50 transition-colors flex-shrink-0"
        @mousedown="startResize('right', $event)"
      />

      <!-- Right panel: Quiz/Settings -->
      <div
        v-show="layoutStore.rightPanelOpen"
        class="flex-shrink-0 overflow-hidden"
        :style="{ width: layoutStore.readerWidth + 'px' }"
      >
        <slot name="right" />
      </div>
    </div>

    <StatusBar />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useLayoutStore } from '@/stores/layout'
import TitleBar from './TitleBar.vue'
import StatusBar from './StatusBar.vue'

const layoutStore = useLayoutStore()

let isResizing = false
let resizeSide: 'left' | 'right' | null = null
let startX = 0
let startWidth = 0

function startResize(side: 'left' | 'right', event: MouseEvent) {
  isResizing = true
  resizeSide = side
  startX = event.clientX
  startWidth = side === 'left' ? layoutStore.explorerWidth : layoutStore.readerWidth
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(event: MouseEvent) {
  if (!isResizing || !resizeSide) return

  const delta = event.clientX - startX

  if (resizeSide === 'left') {
    layoutStore.setExplorerWidth(startWidth + delta)
  } else {
    layoutStore.setReaderWidth(startWidth - delta)
  }
}

function stopResize() {
  if (isResizing) {
    isResizing = false
    resizeSide = null
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    document.removeEventListener('mousemove', onResize)
    document.removeEventListener('mouseup', stopResize)
  }
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>