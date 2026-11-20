<script setup lang="ts">
/**
 * Icônes d'interface au trait (couleur héritée via `currentColor`) : actions,
 * options et états des écrans Nouvelle candidature et fiche candidature.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

/** Nom d'une icône d'interface. */
export type UiIconName =
  | 'link' | 'window' | 'paste' | 'pen' | 'check' | 'x' | 'arrow' | 'back'
  | 'mail' | 'message' | 'send' | 'spark' | 'eye' | 'download' | 'unlink'
  | 'plus' | 'lock' | 'external' | 'edit' | 'trash' | 'alert' | 'document' | 'spinner'

const props = withDefaults(defineProps<{
  name: UiIconName
  /** Taille en pixels (défaut propre à chaque icône, comme dans la maquette). */
  size?: number
}>(), { size: undefined })

/** Taille par défaut de chaque icône. */
const DEFAULT_SIZES: Partial<Record<UiIconName, number>> = {
  check: 12, x: 11, arrow: 14, spark: 14, unlink: 14, plus: 14, lock: 12,
  external: 13, edit: 14, trash: 14, eye: 15, download: 15, back: 16, alert: 16, spinner: 14,
}

/** Épaisseur du trait de chaque icône (1.8 pour les icônes d'option). */
const STROKE_WIDTHS: Partial<Record<UiIconName, number>> = {
  check: 3, x: 3, plus: 2.2,
  window: 1.8, paste: 1.8, pen: 1.8, mail: 1.8, message: 1.8, send: 1.8, document: 1.8,
}

const pixels = computed(() => props.size ?? DEFAULT_SIZES[props.name] ?? 18)
const stroke = computed(() => STROKE_WIDTHS[props.name] ?? 2)
</script>

<template>
  <svg
    :width="pixels" :height="pixels" viewBox="0 0 24 24" fill="none" stroke="currentColor"
    :stroke-width="stroke" stroke-linecap="round" stroke-linejoin="round"
    class="flex-none" :class="{ 'animate-spin': name === 'spinner' }" aria-hidden="true"
  >
    <template v-if="name === 'link'">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </template>
    <template v-else-if="name === 'window'">
      <rect x="3" y="4" width="18" height="16" rx="2" /><line x1="3" y1="9" x2="21" y2="9" /><circle cx="6.5" cy="6.5" r=".6" fill="currentColor" /><circle cx="9" cy="6.5" r=".6" fill="currentColor" />
    </template>
    <template v-else-if="name === 'paste'">
      <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" /><rect x="8" y="2" width="8" height="4" rx="1" />
    </template>
    <template v-else-if="name === 'pen'">
      <path d="M12 20h9" /><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4z" />
    </template>
    <template v-else-if="name === 'check'">
      <polyline points="20 6 9 17 4 12" />
    </template>
    <template v-else-if="name === 'x'">
      <line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" />
    </template>
    <template v-else-if="name === 'arrow'">
      <line x1="5" y1="12" x2="19" y2="12" /><polyline points="12 5 19 12 12 19" />
    </template>
    <template v-else-if="name === 'back'">
      <line x1="19" y1="12" x2="5" y2="12" /><polyline points="12 19 5 12 12 5" />
    </template>
    <template v-else-if="name === 'mail'">
      <rect x="3" y="5" width="18" height="14" rx="2" /><polyline points="3 7 12 13 21 7" />
    </template>
    <template v-else-if="name === 'message'">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </template>
    <template v-else-if="name === 'send'">
      <polyline points="20 6 9 17 4 12" />
    </template>
    <template v-else-if="name === 'spark'">
      <path d="M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2z" />
    </template>
    <template v-else-if="name === 'eye'">
      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" /><circle cx="12" cy="12" r="3" />
    </template>
    <template v-else-if="name === 'download'">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" />
    </template>
    <template v-else-if="name === 'unlink'">
      <path d="M18.84 12.25l1.72-1.71a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M5.17 11.75l-1.71 1.71a5 5 0 0 0 7.07 7.07l1.71-1.71" /><line x1="2" y1="2" x2="22" y2="22" />
    </template>
    <template v-else-if="name === 'plus'">
      <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
    </template>
    <template v-else-if="name === 'lock'">
      <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </template>
    <template v-else-if="name === 'external'">
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" />
    </template>
    <template v-else-if="name === 'edit'">
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
    </template>
    <template v-else-if="name === 'trash'">
      <polyline points="3 6 5 6 21 6" /><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" /><path d="M10 11v6M14 11v6" /><path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
    </template>
    <template v-else-if="name === 'alert'">
      <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
    </template>
    <template v-else-if="name === 'document'">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" />
    </template>
    <template v-else>
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </template>
  </svg>
</template>
