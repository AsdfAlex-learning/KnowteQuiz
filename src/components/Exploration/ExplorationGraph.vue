<template>
  <div class="space-y-3">
    <div v-if="graph.domains.length" class="flex flex-wrap gap-1.5">
      <span
        v-for="domain in graph.domains"
        :key="domain.id"
        class="px-2 py-0.5 rounded-full text-[11px] bg-[var(--accent-purple)]/12 text-[var(--accent-purple)]"
      >
        {{ domain.title }} · {{ domain.concept_ids.length }}
      </span>
    </div>

    <div class="space-y-0.5">
      <div
        v-for="concept in ordered"
        :key="concept.id"
        class="rounded-[var(--radius-sm)] px-2 py-1.5 hover:bg-[var(--bg-hover)]"
        :style="{ marginLeft: concept.depth * 14 + 'px' }"
      >
        <div class="flex items-center gap-2">
          <span
            class="w-1.5 h-1.5 rounded-full shrink-0"
            :class="concept.covered ? 'bg-[var(--color-success)]' : 'bg-[var(--text-faint)]'"
          />
          <span class="text-[var(--text-sm)] text-[var(--text-primary)]">{{ concept.title }}</span>
          <span
            v-if="concept.source_refs.length"
            class="ml-auto text-[10px] text-[var(--text-faint)] shrink-0"
            :title="sourceTitle(concept)"
          >
            {{ concept.source_refs.length }}
          </span>
        </div>
        <p v-if="concept.summary" class="mt-0.5 ml-3.5 text-[11px] text-[var(--text-muted)] leading-relaxed">
          {{ concept.summary }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Concept, KnowledgeGraph } from '@/types/exploration';

const props = defineProps<{
  graph: KnowledgeGraph;
}>();

const ordered = computed<Concept[]>(() => {
  const byId = new Map(props.graph.concepts.map((concept) => [concept.id, concept]));
  const seen = new Set<string>();
  const result: Concept[] = [];
  for (const id of props.graph.root_order) {
    const concept = byId.get(id);
    if (concept && !seen.has(id)) {
      result.push(concept);
      seen.add(id);
    }
  }
  for (const concept of props.graph.concepts) {
    if (!seen.has(concept.id)) result.push(concept);
  }
  return result;
});

function sourceTitle(concept: Concept): string {
  return concept.source_refs
    .map((ref) => (ref.heading ? `${ref.note_path} › ${ref.heading}` : ref.note_path))
    .join('\n');
}
</script>
