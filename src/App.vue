<template>
  <div class="h-screen w-screen flex flex-col text-[var(--text-primary)] overflow-hidden" :class="appBgClass">
    <BackgroundLayer />
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
        data-glass-panel="sidebar"
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
        data-glass-panel="content"
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
        data-glass-panel="sidebar"
      >
        <PanelContainer />
      </aside>
    </div>
    <StatusBar />
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch, computed } from 'vue';
import { useLayoutStore } from './stores/layout';
import { useReaderStore } from './stores/reader';
import { useExplorerStore } from './stores/explorer';
import { useSettingsStore } from './stores/settings';
import { useNavigationStore } from './stores/navigation';
import { useTheme } from './composables/useTheme';
import TitleBar from './components/Layout/TitleBar.vue';
import StatusBar from './components/Layout/StatusBar.vue';
import BackgroundLayer from './components/Layout/BackgroundLayer.vue';
import FolderSelector from './components/Explorer/FolderSelector.vue';
import FileTree from './components/Explorer/FileTree.vue';
import MarkdownRenderer from './components/Reader/MarkdownRenderer.vue';
import EmptyState from './components/Reader/EmptyState.vue';
import PanelContainer from './components/Panel/PanelContainer.vue';

const layoutStore = useLayoutStore();
const readerStore = useReaderStore();
const explorerStore = useExplorerStore();
const settingsStore = useSettingsStore();
const navigationStore = useNavigationStore();
const readerMain = ref<HTMLElement | null>(null);

useTheme();

const appBgClass = computed(() => {
  const theme = settingsStore.settings.theme_config;
  if (!theme) return 'bg-[var(--bg-base)]';
  const hasCustomBg = theme.background_image || (theme.background_color && theme.background_color !== '#1e1e2e');
  const hasGlass = theme.glassmorphism?.enabled;
  return hasCustomBg || hasGlass ? 'bg-transparent' : 'bg-[var(--bg-base)]';
});

let pendingScrollSave: { path: string; top: number } | null = null;
let scrollSaveTimer: ReturnType<typeof setTimeout> | null = null;
let activeDragCleanup: (() => void) | null = null;

function startDragLeft(e: MouseEvent) {
  e.preventDefault();
  // Prevent duplicate listener attachment from rapid clicks
  if (activeDragCleanup) {
    activeDragCleanup();
    activeDragCleanup = null;
  }
  const startX = e.clientX;
  const startWidth = layoutStore.explorerWidth;
  function onMove(e: MouseEvent) {
    const delta = e.clientX - startX;
    layoutStore.setExplorerWidth(Math.max(150, Math.min(500, startWidth + delta)));
  }
  function onUp() {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    activeDragCleanup = null;
    layoutStore.persistLayout();
  }
  activeDragCleanup = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

function startDragRight(e: MouseEvent) {
  e.preventDefault();
  // Prevent duplicate listener attachment from rapid clicks
  if (activeDragCleanup) {
    activeDragCleanup();
    activeDragCleanup = null;
  }
  const startX = e.clientX;
  const startWidth = layoutStore.readerWidth;
  function onMove(e: MouseEvent) {
    const delta = startX - e.clientX;
    layoutStore.setReaderWidth(Math.max(200, Math.min(600, startWidth + delta)));
  }
  function onUp() {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    activeDragCleanup = null;
    layoutStore.persistLayout();
  }
  activeDragCleanup = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

function onKeyDown(e: KeyboardEvent) {
  if (e.ctrlKey && e.key === 'b') {
    e.preventDefault();
    if (e.shiftKey) {
      layoutStore.toggleRightPanel();
    } else {
      layoutStore.toggleLeftPanel();
    }
  }
}

function onReaderScroll() {
  const path = readerStore.currentNote?.path;
  const top = readerMain.value?.scrollTop ?? 0;
  if (!path) return;

  readerStore.scrollTop = top;
  pendingScrollSave = { path, top };
  if (scrollSaveTimer) clearTimeout(scrollSaveTimer);
  scrollSaveTimer = setTimeout(() => {
    void flushReaderScrollSave();
  }, 500);
}

async function flushReaderScrollSave() {
  if (scrollSaveTimer) {
    clearTimeout(scrollSaveTimer);
    scrollSaveTimer = null;
  }
  const pending = pendingScrollSave;
  pendingScrollSave = null;
  if (!pending) return;

  await readerStore.saveScrollPosition(pending.path, pending.top);
}

async function restoreReaderScroll() {
  await nextTick();
  requestAnimationFrame(() => {
    if (readerMain.value) {
      readerMain.value.scrollTop = readerStore.scrollTop;
    }
  });
}

watch(
  () => readerStore.currentNote?.path,
  (path) => {
    if (path) {
      void restoreReaderScroll();
    }
  }
);

onMounted(async () => {
  layoutStore.loadLayout();
  await settingsStore.loadSettings();
  await explorerStore.restoreWorkspace();
  await navigationStore.restoreSelectedNote();
  document.addEventListener('keydown', onKeyDown);
});

onUnmounted(() => {
  if (activeDragCleanup) {
    activeDragCleanup();
    activeDragCleanup = null;
  }
  document.removeEventListener('keydown', onKeyDown);
  void flushReaderScrollSave();
});
</script>
