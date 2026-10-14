<script setup lang="ts">
/**
 * Étape de capture : la page de l'offre est ouverte dans une fenêtre
 * FindMyJob, l'utilisateur y accepte les cookies ou se connecte, puis
 * capture la page affichée pour l'envoyer à l'analyse.
 */

defineProps<{
  busy: boolean
  error: string | null
}>()

defineEmits<{ capture: []; reopen: []; paste: []; manual: []; back: [] }>()
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">PAGE DE L'OFFRE</div>
    <p class="text-[13px] text-neutral-500 mb-[14px] leading-[1.5]">
      La page de l'offre est ouverte dans une fenêtre FindMyJob. Accepte les cookies ou connecte-toi si le site le demande,
      attends que le texte de l'offre soit affiché, puis capture la page. Ta session sur ce site est conservée pour les prochaines fois.
    </p>
    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-[14px] text-[13px] text-red-600">{{ error }}</div>
    <div class="flex flex-wrap gap-3">
      <button
        class="h-12 text-[15px] rounded-10 px-4 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('capture')"
      >{{ busy ? 'Analyse en cours…' : 'Capturer cette page' }}</button>
      <button
        class="h-12 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('reopen')"
      >Rouvrir la fenêtre</button>
      <button
        class="h-12 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': busy }"
        :disabled="busy"
        @click="$emit('paste')"
      >Coller le texte</button>
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
