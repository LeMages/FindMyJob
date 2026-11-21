<script setup lang="ts">
/**
 * Lien discret « ← Revenir au lien de l'offre » des étapes qui suivent la
 * saisie du lien. Propre à `NewApplicationView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

defineEmits<{ back: [] }>()

const { t } = useI18n()
</script>

<template>
  <button
    type="button"
    class="inline-block border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer hover:text-neutral-950"
    @click="$emit('back')"
  >{{ t('new_application.back_to_offer_link') }}</button>
</template>
