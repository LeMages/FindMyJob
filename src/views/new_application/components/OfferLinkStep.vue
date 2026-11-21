<script setup lang="ts">
/**
 * Étape « lien de l'offre » : carte centrée avec le champ du lien et le
 * bouton Analyser dans le même bloc, et un lien discret vers la saisie
 * manuelle. Affiche au-dessus du champ l'offre expirée ou l'erreur réelle
 * d'une analyse précédente.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import UiIcon from '@/components/UiIcon.vue'

const { t } = useI18n()

const url = defineModel<string>({ required: true })

defineProps<{
  busy: boolean
  unavailable: string | null
  error: string | null
}>()

const emit = defineEmits<{ analyze: []; manual: [] }>()

/** Lance l'analyse (bouton ou touche Entrée), sauf lien vide ou analyse en cours. */
function submit(busy: boolean) {
  if (busy || !url.value.trim()) return
  emit('analyze')
}
</script>

<template>
  <div class="max-w-[680px] mx-auto mt-10 text-center bg-white border border-neutral-200 rounded-2xl px-6 sm:px-11 pt-10 pb-8">
    <div class="w-12 h-12 rounded-xl bg-neutral-100 text-neutral-700 flex items-center justify-center mx-auto mb-4">
      <UiIcon name="link" :size="18" />
    </div>
    <h2 class="text-[20px] font-semibold text-neutral-950 m-0">{{ t('new_application.link.title') }}</h2>
    <p class="text-[13.5px] text-neutral-500 mt-[6px] mb-[22px]">{{ t('new_application.link.intro') }}</p>

    <InfoBanner v-if="unavailable" tone="warn">{{ unavailable }} {{ t('new_application.link.unavailable_hint') }}</InfoBanner>
    <InfoBanner v-if="error" tone="err">{{ error }}</InfoBanner>

    <form
      class="flex items-center gap-2 border border-neutral-300 rounded-xl py-[6px] pr-[6px] pl-[14px] bg-neutral-50 text-left focus-within:border-neutral-500 focus-within:bg-white"
      @submit.prevent="submit(busy)"
    >
      <span class="flex text-neutral-400"><UiIcon name="link" :size="18" /></span>
      <input
        v-model="url"
        type="url"
        class="flex-1 min-w-0 border-0 bg-transparent text-[14px] font-sans text-neutral-950 outline-none h-9"
        :placeholder="t('new_application.link.placeholder')"
      />
      <button
        type="submit"
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
        :disabled="busy || !url.trim()"
      >
        <template v-if="busy"><UiIcon name="spinner" /> {{ t('new_application.analyzing') }}</template>
        <template v-else>{{ t('new_application.link.analyze') }} <UiIcon name="arrow" /></template>
      </button>
    </form>

    <button
      type="button"
      class="mt-[18px] border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer underline underline-offset-[3px] hover:text-neutral-950 disabled:opacity-40 disabled:cursor-not-allowed"
      :disabled="busy"
      @click="emit('manual')"
    >{{ t('new_application.link.manual') }}</button>
  </div>
</template>
