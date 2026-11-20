<script setup lang="ts">
/**
 * Popup partagée : fond assombri, panneau jamais plus large que l'écran moins
 * une marge, au plus 88 % de la hauteur de la fenêtre, en-tête et pied fixes
 * et contenu défilant. En variante `divided`, l'en-tête est séparé du contenu
 * par un trait et peut porter une flèche de retour (`back`) et une icône
 * (slot `icon`).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import UiIcon from './UiIcon.vue'

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
  /** En-tête séparé du contenu par un trait (popups à icône). */
  divided?: boolean
  /** Affiche une flèche de retour avant le titre (émet `back`). */
  back?: boolean
}>(), { subtitle: undefined, scroll: true, backdropClose: false, divided: false, back: false })

const emit = defineEmits<{
  /** Fermeture demandée (croix, ou fond si `backdropClose`). */
  close: []
  /** Retour demandé (flèche de l'en-tête, si `back`). */
  back: []
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
      <div
        class="flex items-center justify-between flex-none"
        :class="divided ? 'gap-3 px-5 sm:px-7 pt-[22px] pb-4 border-b border-neutral-100' : 'gap-4 px-5 sm:px-8 pt-6 pb-[18px]'"
      >
        <div class="flex items-center gap-3 flex-1 min-w-0">
          <button
            v-if="back"
            type="button"
            class="w-7 h-7 rounded-7 border border-transparent bg-transparent text-neutral-400 flex items-center justify-center cursor-pointer flex-none hover:border-neutral-200 hover:text-neutral-950 hover:bg-neutral-50"
            :title="t('common.back')"
            :aria-label="t('common.back')"
            @click="emit('back')"
          ><UiIcon name="back" /></button>
          <span v-if="$slots.icon" class="w-9 h-9 rounded-10 bg-neutral-100 text-neutral-700 flex items-center justify-center flex-none [&_svg]:w-[17px] [&_svg]:h-[17px]">
            <slot name="icon" />
          </span>
          <div class="min-w-0">
            <div :class="divided ? 'text-[16px]' : 'text-[17px]'" class="font-semibold text-neutral-950">{{ title }}</div>
            <div v-if="subtitle" class="text-neutral-500" :class="divided ? 'text-[12.5px] mt-px truncate' : 'text-[13px]'">{{ subtitle }}</div>
          </div>
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" :aria-label="t('common.close')" @click="emit('close')">×</button>
      </div>
      <div
        class="flex-1 min-h-0"
        :class="[
          divided ? 'px-5 sm:px-7 pt-[18px]' : 'px-5 sm:px-8 pt-0.5',
          scroll ? `overflow-y-auto ${divided ? 'pb-[22px]' : 'pb-2'}` : 'overflow-visible pb-[18px]',
        ]"
      >
        <slot />
      </div>
      <div v-if="$slots.footer" class="flex flex-wrap items-center justify-end gap-[10px] px-5 sm:px-8 pt-[14px] pb-[18px] border-t border-neutral-100 flex-none">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>
