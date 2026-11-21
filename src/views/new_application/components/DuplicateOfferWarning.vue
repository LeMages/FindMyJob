<script setup lang="ts">
/**
 * Avertissement non bloquant : candidatures existantes (toutes sessions)
 * pointant vers la même offre que le lien saisi.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'

// Importation des modules internes
import { statusLabel } from '@/composables/candidature_display'
import type { DuplicateCandidatureView } from '@/types'

const { t } = useI18n()

defineProps<{
  duplicates: DuplicateCandidatureView[]
  checkError: string | null
}>()
</script>

<template>
  <InfoBanner v-if="duplicates.length > 0" tone="warn">
    <strong>{{ t('new_application.duplicates.title', duplicates.length) }}</strong>
    <ul>
      <li v-for="dup in duplicates" :key="dup.id">
        {{ dup.poste }}<template v-if="dup.entreprise"> · {{ dup.entreprise }}</template>
        {{ t('new_application.duplicates.item', { status: statusLabel(dup.status), session: dup.session_nom }) }}
      </li>
    </ul>
    <div>{{ t('new_application.duplicates.hint') }}</div>
  </InfoBanner>
  <div v-else-if="checkError" class="text-[12px] text-neutral-500 mb-3">{{ checkError }}</div>
</template>
