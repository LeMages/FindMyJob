<script setup lang="ts">
/**
 * Étape « texte collé » : l'utilisateur colle le texte de l'offre, analysé
 * par l'agent comme une page.
 */

const text = defineModel<string>({ required: true })

defineProps<{
  busy: boolean
  error: string | null
}>()

defineEmits<{ analyze: []; manual: []; back: [] }>()
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">TEXTE DE L'OFFRE</div>
    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-red-600">{{ error }}</div>
    <p class="text-[13px] text-neutral-500 mb-[14px]">Sélectionne tout le texte de la page de l'offre et colle-le ci-dessous.</p>
    <div class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">Texte de l'offre</label>
      <textarea v-model="text" class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-48" placeholder="Colle ici le texte complet de l'offre d'emploi..." />
    </div>
    <div class="flex gap-3">
      <button
        class="h-12 text-[15px] rounded-10 px-4 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy || !text.trim() }"
        :disabled="busy || !text.trim()"
        @click="$emit('analyze')"
      >{{ busy ? 'Analyse en cours…' : 'Analyser le texte →' }}</button>
      <button
        class="h-12 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('manual')"
      >Saisie manuelle</button>
    </div>
    <button class="bg-transparent border-none font-mono text-[12px] text-neutral-500 cursor-pointer py-1 mt-[14px] hover:text-neutral-950" @click="$emit('back')">← Revenir au lien</button>
  </div>
</template>
