<script setup lang="ts">
/**
 * Carte « Prêt à générer ? » du formulaire de nouvelle candidature : liste
 * de vérification (champs obligatoires, modèles, ville de la session, session
 * active), avec un lien vers les paramètres de la session pour chaque point
 * manquant qui s'y corrige, l'erreur de création et le bouton qui crée la
 * candidature et lance la génération. Propre à `NewApplicationView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import { openSessionSettings } from '@/stores/app'
import type { Checklist } from '../generation_checklist'

defineProps<{
  checklist: Checklist
  saving: boolean
  error: string | null
  /** Infobulle du bouton désactivé (session en lecture seule), sinon `null`. */
  lockedHint: string | null
}>()

defineEmits<{ submit: [] }>()

const { t } = useI18n()
</script>

<template>
  <SectionCard :title="t('new_application.ready.title')" :hint="t('new_application.ready.hint')">
    <ul class="list-none mt-0 mb-[14px] p-0 flex flex-col gap-2">
      <li
        v-for="item in checklist.items"
        :key="item.key"
        class="flex items-center gap-[10px] text-[12.5px]"
        :class="item.ok ? 'text-neutral-700' : 'text-neutral-950'"
      >
        <span
          class="w-[18px] h-[18px] rounded-full flex items-center justify-center flex-none"
          :class="item.ok ? 'bg-green-50 text-green-600' : 'bg-red-50 text-red-600'"
        >
          <UiIcon :name="item.ok ? 'check' : 'x'" />
        </span>
        <span class="flex-1">{{ t(`new_application.ready.${item.key}`) }}</span>
        <button
          v-if="!item.ok && item.fixInSessionSettings"
          type="button"
          class="border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer underline underline-offset-[3px] hover:text-neutral-950"
          @click="openSessionSettings"
        >{{ t('new_application.ready.settings_link') }}</button>
      </li>
    </ul>
    <InfoBanner v-if="error" tone="err">{{ error }}</InfoBanner>
    <span class="block" :title="lockedHint ?? undefined">
      <button
        type="button"
        class="w-full justify-center h-11 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
        :disabled="saving || !checklist.canGenerate"
        @click="$emit('submit')"
      >
        <template v-if="saving"><UiIcon name="spinner" /> {{ t('new_application.ready.creating') }}</template>
        <template v-else>{{ t('new_application.ready.submit') }}</template>
      </button>
    </span>
  </SectionCard>
</template>
