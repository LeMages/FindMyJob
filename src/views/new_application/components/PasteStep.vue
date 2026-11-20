<script setup lang="ts">
/**
 * Étape « texte collé » : l'utilisateur colle le texte complet de la page
 * de l'offre (Ctrl + A), analysé comme une page.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'
import BackToLinkButton from './BackToLinkButton.vue'

const { t } = useI18n()

const text = defineModel<string>({ required: true })

defineProps<{
  busy: boolean
  error: string | null
}>()

defineEmits<{ analyze: []; manual: []; back: [] }>()
</script>

<template>
  <div class="max-w-[720px] mx-auto mt-6">
    <SectionCard :title="t('new_application.paste.title')" :hint="t('new_application.paste.hint')">
      <InfoBanner v-if="error" tone="err">{{ error }}</InfoBanner>
      <textarea
        v-model="text"
        class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[280px] focus:outline-none focus:border-neutral-400"
        :placeholder="t('new_application.form.offer_placeholder')"
        :aria-label="t('new_application.paste.title')"
      />
      <div class="flex flex-wrap items-center gap-[10px] mt-4">
        <button
          type="button"
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
          :disabled="busy || !text.trim()"
          @click="$emit('analyze')"
        >
          <template v-if="busy"><UiIcon name="spinner" /> {{ t('new_application.analyzing') }}</template>
          <template v-else>{{ t('new_application.paste.analyze') }} <UiIcon name="arrow" /></template>
        </button>
        <span class="flex-1" />
        <button
          type="button"
          class="border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer underline underline-offset-[3px] hover:text-neutral-950"
          @click="$emit('manual')"
        >{{ t('new_application.fill_manually') }}</button>
      </div>
    </SectionCard>
    <BackToLinkButton class="mt-4" @back="$emit('back')" />
  </div>
</template>
