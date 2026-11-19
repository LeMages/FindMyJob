<script setup lang="ts">
/**
 * Popup partagée : fond assombri, panneau jamais plus large que l'écran moins
 * une marge, au plus 88 % de la hauteur de la fenêtre, en-tête et pied fixes
 * et contenu défilant.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

withDefaults(defineProps<{
  /** Titre affiché dans l'en-tête. */
  title: string
  /** Sous-titre facultatif sous le titre. */
  subtitle?: string
  /** Largeur cible en pixels (bornée par la largeur de l'écran). */
  width: number
  /**
   * `false` pour laisser déborder le contenu (liste déroulante d'un champ
   * dans une petite popup) au lieu de le faire défiler.
   */
  scroll?: boolean
  /** `true` pour fermer la popup d'un clic sur le fond assombri. */
  backdropClose?: boolean
}>(), { scroll: true, backdropClose: false })

const emit = defineEmits<{
  /** Fermeture demandée (croix, ou fond si `backdropClose`). */
  close: []
}>()

const { t } = useI18n()
</script>

<template>
  <div class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100] p-4" @click.self="backdropClose && emit('close')">
    <div
      class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] max-h-[min(88vh,860px)] flex flex-col"
      :class="scroll ? 'overflow-hidden' : 'overflow-visible'"
      :style="{ width: `min(${width}px, calc(100vw - 32px))` }"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
    >
      <div class="flex items-center justify-between gap-4 px-5 sm:px-8 pt-6 pb-[18px] flex-none">
        <div class="min-w-0">
          <div class="text-[17px] font-semibold">{{ title }}</div>
          <div v-if="subtitle" class="text-[13px] text-neutral-500">{{ subtitle }}</div>
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" :aria-label="t('common.close')" @click="emit('close')">×</button>
      </div>
      <div class="px-5 sm:px-8 pt-0.5 flex-1 min-h-0" :class="scroll ? 'overflow-y-auto pb-2' : 'overflow-visible pb-[18px]'">
        <slot />
      </div>
      <div v-if="$slots.footer" class="flex flex-wrap items-center justify-end gap-[10px] px-5 sm:px-8 pt-[14px] pb-[18px] border-t border-neutral-100 flex-none">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>
