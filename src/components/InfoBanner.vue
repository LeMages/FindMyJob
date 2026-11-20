<script setup lang="ts">
/**
 * Bandeau d'information arrondi : avertissement (ambre), erreur (rouge),
 * succès (vert) ou neutre (gris). Le contenu est libre (slot).
 */

withDefaults(defineProps<{
  tone?: 'warn' | 'err' | 'ok' | 'muted'
}>(), { tone: 'warn' })

const TONES = {
  warn: 'bg-amber-50 border-amber-200 text-amber-700',
  err: 'bg-red-50 border-red-200 text-red-600',
  ok: 'bg-green-50 border-green-400/40 text-green-700',
  muted: 'bg-neutral-50 border-neutral-200 text-neutral-600',
}
</script>

<template>
  <div
    class="rounded-10 border px-[14px] py-[10px] text-[13px] leading-[1.5] mb-[14px] text-left [&_ul]:my-1 [&_ul]:pl-[18px] [&_ul]:list-disc"
    :class="TONES[tone]"
    :role="tone === 'err' ? 'alert' : 'status'"
  >
    <slot />
  </div>
</template>
