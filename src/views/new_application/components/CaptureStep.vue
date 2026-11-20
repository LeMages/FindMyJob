<script setup lang="ts">
/**
 * Étape de capture : la page de l'offre est ouverte dans une fenêtre
 * FindMyJob, l'utilisateur y accepte les cookies ou se connecte (consignes en
 * trois étapes), puis capture la page affichée pour l'envoyer à l'analyse.
 * La fenêtre peut être rouverte ; collage et saisie manuelle restent offerts.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'
import BackToLinkButton from './BackToLinkButton.vue'

const { t } = useI18n()

defineProps<{
  busy: boolean
  error: string | null
}>()

defineEmits<{ capture: []; reopen: []; paste: []; manual: []; back: [] }>()

const LINK = 'border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer underline underline-offset-[3px] hover:text-neutral-950'
</script>

<template>
  <div class="max-w-[720px] mx-auto mt-6">
    <SectionCard :title="t('new_application.capture.title')" :hint="t('new_application.capture.hint')">
      <ol class="mt-0 mb-4 pl-5 list-decimal text-[13px] text-neutral-700 leading-[1.8]">
        <li>{{ t('new_application.capture.step_1') }}</li>
        <li>{{ t('new_application.capture.step_2') }}</li>
        <li>{{ t('new_application.capture.step_3') }}</li>
      </ol>
      <InfoBanner v-if="error" tone="err">{{ error }}</InfoBanner>
      <div class="flex flex-wrap items-center gap-[10px] mt-4">
        <button
          type="button"
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
          :disabled="busy"
          @click="$emit('capture')"
        >
          <template v-if="busy"><UiIcon name="spinner" /> {{ t('new_application.analyzing') }}</template>
          <template v-else>{{ t('new_application.capture.capture') }}</template>
        </button>
        <button
          type="button"
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
          :disabled="busy"
          @click="$emit('reopen')"
        >{{ t('new_application.capture.reopen') }}</button>
        <span class="flex-1" />
        <button type="button" :class="LINK" @click="$emit('paste')">{{ t('new_application.capture.paste') }}</button>
        <button type="button" :class="LINK" @click="$emit('manual')">{{ t('new_application.fill_manually') }}</button>
      </div>
    </SectionCard>
    <BackToLinkButton class="mt-4" @back="$emit('back')" />
  </div>
</template>
