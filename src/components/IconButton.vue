<script setup lang="ts">
/**
 * Bouton d'action réduit à une icône, avec infobulle (`title`) et libellé
 * accessible : discret au repos, encadré au survol, rouge au survol pour une
 * action destructive (`danger`).
 */

withDefaults(defineProps<{
  /** Infobulle et libellé accessible. */
  title: string
  /** Infobulle à la place de `title` (raison d'un bouton désactivé), sinon `null`. */
  tooltip?: string | null
  /** Action destructive : rouge au survol. */
  danger?: boolean
  disabled?: boolean
  /** Côté du bouton en pixels. */
  size?: number
}>(), { tooltip: null, danger: false, disabled: false, size: 28 })

defineEmits<{ click: [] }>()
</script>

<template>
  <!-- L'infobulle reste visible sur un bouton désactivé grâce à l'enveloppe. -->
  <span class="inline-flex flex-none" :title="tooltip ?? title">
    <button
      type="button"
      class="rounded-7 border border-transparent bg-transparent text-neutral-400 flex items-center justify-center cursor-pointer flex-none disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
      :class="danger
        ? 'hover:text-red-600 hover:bg-red-50 hover:border-red-200'
        : 'hover:text-neutral-950 hover:bg-neutral-50 hover:border-neutral-200'"
      :style="{ width: `${size}px`, height: `${size}px` }"
      :aria-label="title"
      :disabled="disabled"
      @click="$emit('click')"
    >
      <slot />
    </button>
  </span>
</template>
