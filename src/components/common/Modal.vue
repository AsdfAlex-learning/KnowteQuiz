<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center" @keydown.escape="close">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close" />

        <!-- Dialog -->
        <div :class="dialogClasses" role="dialog" aria-modal="true" :aria-labelledby="titleId">
          <!-- Header -->
          <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--border-subtle)]">
            <h2 :id="titleId" class="text-[var(--text-lg)] font-semibold text-[var(--text-primary)]">
              {{ title }}
            </h2>
            <button
              class="w-7 h-7 flex items-center justify-center rounded-[var(--radius-sm)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
              aria-label="Close"
              @click="close"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 4l8 8M12 4l-8 8" />
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="px-5 py-4 overflow-y-auto max-h-[60vh]">
            <slot />
          </div>

          <!-- Footer -->
          <div
            v-if="$slots.footer"
            class="px-5 py-3 border-t border-[var(--border-subtle)] flex items-center justify-end gap-2"
          >
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title: string;
    size?: 'sm' | 'md' | 'lg';
  }>(),
  {
    size: 'md',
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const titleId = computed(() => `modal-title-${Math.random().toString(36).slice(2, 9)}`);

const dialogClasses = computed(() => [
  'relative z-10 bg-[var(--bg-sidebar)] border border-[var(--border-subtle)] rounded-[var(--radius-xl)] shadow-[var(--shadow-lg)] w-full',
  {
    sm: 'max-w-sm',
    md: 'max-w-lg',
    lg: 'max-w-2xl',
  }[props.size],
]);

function close() {
  emit('update:modelValue', false);
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 200ms ease;
}
.modal-enter-active .relative.z-10,
.modal-leave-active .relative.z-10 {
  transition:
    transform 200ms ease,
    opacity 200ms ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .relative.z-10,
.modal-leave-to .relative.z-10 {
  transform: scale(0.95);
  opacity: 0;
}
</style>
