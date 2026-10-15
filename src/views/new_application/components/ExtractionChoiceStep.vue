<script setup lang="ts">
/**
 * Écran de choix affiché quand l'analyse automatique du lien échoue (accès
 * refusé, contenu insuffisant…) : ouvrir la page dans FindMyJob, coller le
 * texte, ou saisir l'offre à la main.
 */

defineProps<{
  message: string
  canCapture: boolean
  busy: boolean
}>()

defineEmits<{ capture: []; paste: []; manual: []; back: [] }>()
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">ANALYSE AUTOMATIQUE IMPOSSIBLE</div>
    <div class="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-amber-700">{{ message }}</div>
    <p class="text-[13px] text-neutral-500 mb-[14px]">Choisis comment continuer :</p>
    <div class="flex flex-col gap-3">
      <button
        v-if="canCapture"
        class="h-12 text-[15px] rounded-10 px-4 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('capture')"
      >Ouvrir la page dans FindMyJob</button>
      <button
        class="h-12 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('paste')"
      >Coller le texte de l'offre</button>
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
