<script setup lang="ts">
/**
 * Avertissement non bloquant : candidatures existantes (toutes sessions)
 * pointant vers la même offre que le lien saisi.
 */

// Importation des modules internes
import { statusLabel } from '@/composables/candidature_display'
import type { DuplicateCandidatureView } from '@/types'

defineProps<{
  duplicates: DuplicateCandidatureView[]
  checkError: string | null
}>()
</script>

<template>
  <div v-if="duplicates.length > 0" class="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-amber-700">
    <div class="font-medium mb-1">
      {{ duplicates.length === 1 ? 'Tu as déjà une candidature pour cette offre :' : `Tu as déjà ${duplicates.length} candidatures pour cette offre :` }}
    </div>
    <ul class="list-disc pl-5">
      <li v-for="dup in duplicates" :key="dup.id">
        {{ dup.poste }}<template v-if="dup.entreprise"> — {{ dup.entreprise }}</template>
        ({{ statusLabel(dup.status) }}, session « {{ dup.session_nom }} »)
      </li>
    </ul>
    <div class="mt-1">Tu peux quand même créer celle-ci.</div>
  </div>
  <div v-else-if="checkError" class="text-[12px] text-neutral-500 mb-[14px]">{{ checkError }}</div>
</template>
