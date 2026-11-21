<script setup lang="ts">
/**
 * Écran de choix affiché quand l'analyse automatique du lien échoue (accès
 * refusé, contenu insuffisant…) : bandeau ambre avec la raison, puis trois
 * cartes — ouvrir la page dans FindMyJob (si la capture peut aider), coller
 * le texte, remplir à la main — dont l'option conseillée mise en avant.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import OptionCard from '@/components/OptionCard.vue'
import BackToLinkButton from './BackToLinkButton.vue'

// Importation des modules internes
import { recommendedOption } from '../extraction_choice'

const { t } = useI18n()

const props = defineProps<{
  message: string
  canCapture: boolean
  busy: boolean
}>()

defineEmits<{ capture: []; paste: []; manual: []; back: [] }>()

const recommended = computed(() => recommendedOption(props.canCapture))
</script>

<template>
  <div class="max-w-[720px] mx-auto mt-6">
    <InfoBanner tone="warn"><strong>{{ t('new_application.choice.impossible') }}</strong> {{ message }}</InfoBanner>
    <div class="text-[15px] font-semibold text-neutral-950 mt-[6px] mb-3">{{ t('new_application.choice.prompt') }}</div>
    <div class="flex flex-col gap-[10px]">
      <OptionCard
        v-if="canCapture"
        icon="window"
        :title="t('new_application.choice.capture')"
        :description="t('new_application.choice.capture_description')"
        :badge="recommended === 'capture' ? t('new_application.choice.recommended') : null"
        :primary="recommended === 'capture'"
        :disabled="busy"
        @select="$emit('capture')"
      />
      <OptionCard
        icon="paste"
        :title="t('new_application.choice.paste')"
        :description="t('new_application.choice.paste_description')"
        :badge="recommended === 'paste' ? t('new_application.choice.recommended') : null"
        :primary="recommended === 'paste'"
        :disabled="busy"
        @select="$emit('paste')"
      />
      <OptionCard
        icon="pen"
        :title="t('new_application.fill_manually')"
        :description="t('new_application.choice.manual_description')"
        :disabled="busy"
        @select="$emit('manual')"
      />
    </div>
    <BackToLinkButton class="mt-4" @back="$emit('back')" />
  </div>
</template>
