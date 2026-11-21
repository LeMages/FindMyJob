<script setup lang="ts">
/**
 * Icône du contexte détaillé d'une expérience ou d'un projet : grise si le
 * contexte est renseigné, ambre s'il manque, avec une infobulle.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

defineProps<{
  /** `true` si le contexte détaillé est renseigné. */
  provided: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <span
    class="inline-flex cursor-help"
    :class="provided ? 'text-neutral-400' : 'text-amber-500'"
    :title="provided ? t('profile.detailed_context.provided') : t('profile.detailed_context.missing')"
    :aria-label="provided ? t('profile.detailed_context.provided') : t('profile.detailed_context.missing')"
  >
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="8" y1="13" x2="16" y2="13" /><line x1="8" y1="17" x2="13" y2="17" />
    </svg>
  </span>
</template>
