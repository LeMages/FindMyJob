<script setup lang="ts">
/**
 * Champ « Contexte détaillé » des formulaires d'expérience et de projet :
 * zone de texte libre, aide et compteur de caractères. La limite affichée
 * ici n'est qu'un confort de saisie : la règle est validée côté Rust.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

/** Nombre maximal de caractères, aligné sur `MAX_CONTEXTE_CHARS` (Rust). */
const DETAILED_CONTEXT_MAX_CHARS = 2000

const { t } = useI18n()

const model = defineModel<string>({ required: true })
</script>

<template>
  <div class="mt-[14px] mb-0">
    <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.detailed_context.label') }}</label>
    <textarea
      v-model="model"
      class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[100px]"
      :maxlength="DETAILED_CONTEXT_MAX_CHARS"
    />
    <div class="flex items-start justify-between gap-3 mt-[5px] text-[11.5px] text-neutral-500">
      <span>{{ t('profile.detailed_context.help') }}</span>
      <span class="font-mono flex-none">
        {{ t('profile.detailed_context.counter', { count: model.length, max: DETAILED_CONTEXT_MAX_CHARS }) }}
      </span>
    </div>
  </div>
</template>
