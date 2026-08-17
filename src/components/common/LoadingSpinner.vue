<template>
  <div :class="containerClasses" :role="overlay ? 'alert' : undefined" :aria-busy="overlay ? 'true' : undefined">
    <svg :class="spinnerClasses" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-20" />
      <path
        d="M12 2a10 10 0 0 1 10 10"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
        class="spinner-arc"
      />
    </svg>
    <p v-if="label" class="text-[var(--text-sm)] text-[var(--text-muted)] mt-2">{{ label }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    size?: 'sm' | 'md' | 'lg';
    label?: string;
    overlay?: boolean;
  }>(),
  {
    size: 'md',
    label: '',
    overlay: false,
  }
);

const containerClasses = computed(() => [
  'flex flex-col items-center justify-center',
  props.overlay ? 'absolute inset-0 bg-[var(--bg-base)]/70 z-40' : '',
]);

const spinnerClasses = computed(() => [
  'animate-spin text-[var(--accent-purple)]',
  {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-10 h-10',
  }[props.size],
]);
</script>

<style scoped>
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
