<template>
  <div
    class="h-9 flex items-center justify-between px-3 bg-[var(--bg-sidebar)] border-b border-[var(--border-default)] select-none"
    data-tauri-drag-region
  >
    <!-- Left: App identity -->
    <div class="flex items-center gap-2 min-w-0" data-tauri-drag-region>
      <div class="w-5 h-5 rounded bg-[var(--accent-purple)] flex items-center justify-center flex-shrink-0">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M2 3h3v3H2V3zm5 0h3v3H7V3zM2 8h3v3H2V8zm5 1.5A1.5 1.5 0 017 11a1.5 1.5 0 01-1.5-1.5A1.5 1.5 0 017 8a1.5 1.5 0 011.5 1.5z"
            fill="#1e1e2e"
          />
        </svg>
      </div>
      <span class="text-xs font-semibold text-[var(--text-primary)] truncate" data-tauri-drag-region> KnowteQuiz </span>
    </div>

    <!-- Center: Current note title -->
    <div class="flex-1 text-center text-[11px] text-[var(--text-muted)] truncate px-4" data-tauri-drag-region>
      <slot name="center" />
    </div>

    <!-- Right: Panel toggles + Window controls -->
    <div class="flex items-center gap-1">
      <button
        class="w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-primary)] transition-colors"
        :class="{ 'text-[var(--accent-purple)]': layoutStore.leftPanelOpen }"
        title="Toggle Explorer (Ctrl+B)"
        @click="layoutStore.toggleLeftPanel()"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="2" width="4" height="12" rx="1" />
          <rect x="8" y="2" width="6" height="12" rx="1" />
        </svg>
      </button>
      <button
        class="w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-primary)] transition-colors"
        :class="{ 'text-[var(--accent-purple)]': layoutStore.rightPanelOpen }"
        title="Toggle Panel (Ctrl+Shift+B)"
        @click="layoutStore.toggleRightPanel()"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="10" y="2" width="4" height="12" rx="1" />
          <rect x="2" y="2" width="6" height="12" rx="1" />
        </svg>
      </button>

      <!-- Window controls (Tauri only) -->
      <template v-if="isTauri()">
        <div class="w-px h-4 bg-[var(--bg-active)] mx-1" />

        <button
          class="w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-primary)] transition-colors"
          title="Minimize"
          @click="minimize"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <rect x="1" y="5" width="10" height="2" rx="0.5" />
          </svg>
        </button>
        <button
          class="w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-primary)] transition-colors"
          title="Maximize"
          @click="toggleMaximize"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="1.5" y="1.5" width="9" height="9" rx="1" />
          </svg>
        </button>
        <button
          class="w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--color-error)] hover:text-[var(--bg-base)] transition-colors"
          title="Close"
          @click="closeWindow"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2 2l8 8M10 2l-8 8" />
          </svg>
        </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useLayoutStore } from '@/stores/layout';
import { isTauri } from '@/services/tauri';

const layoutStore = useLayoutStore();

const tauriAvailable = computed(() => isTauri());

function minimize() {
  if (tauriAvailable.value) {
    import('@tauri-apps/api/webviewWindow').then(({ getCurrentWebviewWindow }) => {
      getCurrentWebviewWindow().minimize();
    });
  }
}

function toggleMaximize() {
  if (tauriAvailable.value) {
    import('@tauri-apps/api/webviewWindow').then(({ getCurrentWebviewWindow }) => {
      getCurrentWebviewWindow().toggleMaximize();
    });
  }
}

function closeWindow() {
  if (tauriAvailable.value) {
    import('@tauri-apps/api/webviewWindow').then(({ getCurrentWebviewWindow }) => {
      getCurrentWebviewWindow().close();
    });
  }
}
</script>
