<script setup lang="ts">
/**
 * Barre d'onglets de la page Candidatures : bascule entre le pipeline de
 * candidatures, le marché (offres scorées), le planning des entretiens et
 * l'onglet Paramètres.
 */

defineProps<{
  active: 'candidatures' | 'marche' | 'planning' | 'parametres'
}>()

defineEmits<{
  select: [tab: 'candidatures' | 'marche' | 'planning' | 'parametres']
}>()
</script>

<template>
  <div class="px-[30px] flex items-center gap-1 border-b border-neutral-200">
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'candidatures' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'candidatures')"
    >
      Candidatures
    </button>
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'marche' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'marche')"
    >
      Marché
    </button>
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'planning' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'planning')"
    >
      Planning
    </button>
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'parametres' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'parametres')"
    >
      Paramètres
    </button>
  </div>
</template>
