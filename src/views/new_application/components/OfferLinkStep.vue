<script setup lang="ts">
/**
 * Étape « lien de l'offre » : saisie de l'URL, analyse ou saisie manuelle.
 * Affiche l'offre expirée ou l'erreur réelle d'une analyse précédente.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const url = defineModel<string>({ required: true })

defineProps<{
  busy: boolean
  unavailable: string | null
  error: string | null
}>()

defineEmits<{ analyze: []; manual: [] }>()
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ t('new_application.link.section') }}</div>
    <p class="text-[13px] text-neutral-500 mb-[14px]">{{ t('new_application.link.intro') }}</p>
    <div v-if="unavailable" class="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-amber-700">
      {{ unavailable }} {{ t('new_application.link.unavailable_hint') }}
    </div>
    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-red-600">{{ error }}</div>
    <div class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.job.link') }}</label>
      <input v-model="url" type="url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('new_application.link.placeholder')" />
    </div>
    <div class="flex gap-3">
      <button
        class="h-12 text-[15px] rounded-10 px-4 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy || !url.trim() }"
        :disabled="busy || !url.trim()"
        @click="$emit('analyze')"
      >{{ busy ? t('new_application.analyzing') : t('new_application.link.analyze') }}</button>
      <button
        class="h-12 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('manual')"
      >{{ t('new_application.manual') }}</button>
    </div>
  </div>
</template>
