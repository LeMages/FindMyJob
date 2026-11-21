<script setup lang="ts">
/**
 * Vue « Paramètres » : une section par thème (intelligence artificielle,
 * polices, interface), une ligne par réglage.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import AiSection from '@/views/settings/components/AiSection.vue'
import FontsSection from '@/views/settings/components/FontsSection.vue'
import InterfaceSection from '@/views/settings/components/InterfaceSection.vue'

const { t } = useI18n()
</script>

<template>
  <div class="px-5 sm:px-9 pt-[26px] pb-16">
    <h1 class="text-2xl font-semibold leading-tight">{{ t('settings.title') }}</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] mb-7">{{ t('settings.subtitle') }}</div>

    <AiSection />
    <FontsSection />
    <InterfaceSection />
  </div>
</template>
