<script setup lang="ts">
/**
 * Ligne de coordonnées de la carte Identité : icône et valeur, lien ouvert
 * dans le navigateur pour LinkedIn, GitHub et portfolio, ou invitation à
 * compléter quand la valeur manque.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

// Importation des composants
import ContactIcon from './ContactIcon.vue'

const props = defineProps<{
  icon: 'mail' | 'phone' | 'linkedin' | 'github' | 'globe' | 'car'
  value: string | null
  /** Texte affiché quand la valeur manque. */
  placeholder: string
  /** `true` si la valeur est une URL à ouvrir au clic. */
  link?: boolean
}>()

const emit = defineEmits<{
  open: [url: string]
}>()

/** URL affichée sans protocole, `www.` ni barre finale. */
const label = computed(() =>
  props.link && props.value ? props.value.replace(/^https?:\/\/(www\.)?/, '').replace(/\/$/, '') : props.value,
)
</script>

<template>
  <a
    v-if="value && link"
    href="#"
    class="group/line flex items-center gap-[10px] py-[5px] text-[12.5px] text-neutral-700 min-w-0 no-underline hover:text-neutral-950"
    :title="value"
    @click.prevent="emit('open', value)"
  >
    <ContactIcon :icon="icon" />
    <span class="truncate group-hover/line:underline">{{ label }}</span>
  </a>
  <div v-else class="flex items-center gap-[10px] py-[5px] text-[12.5px] text-neutral-700 min-w-0">
    <ContactIcon :icon="icon" />
    <span class="truncate">{{ value ? label : placeholder }}</span>
  </div>
</template>
