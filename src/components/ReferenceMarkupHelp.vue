<script setup lang="ts">
/**
 * Aide repliable sur le balisage des consignes d'une référence (`.typ`),
 * affichée à l'endroit de l'import. Le format complet est décrit dans
 * `docs/reference-markup.md` ; ce composant n'en reprend que l'essentiel.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <details class="mt-[10px] text-[12px] text-neutral-600">
    <summary class="cursor-pointer select-none text-neutral-500 hover:text-neutral-700">{{ t('documents.markup.summary') }}</summary>
    <div class="mt-2 flex flex-col gap-[6px] leading-[1.45]">
      <!-- Les balises ([CONSIGNES], [FIXE]…) sont la syntaxe du format : jamais traduites. -->
      <i18n-t keypath="documents.markup.intro" tag="p" scope="global">
        <template #comment><code class="font-mono">//</code></template>
      </i18n-t>
      <ul class="list-disc pl-5 flex flex-col gap-[3px]">
        <i18n-t keypath="documents.markup.general" tag="li" scope="global">
          <template #tag><code class="font-mono">// [CONSIGNES]</code></template>
          <template #comment><code class="font-mono">//</code></template>
        </i18n-t>
        <i18n-t keypath="documents.markup.fixed" tag="li" scope="global">
          <template #tag><code class="font-mono">// [FIXE] {{ t('documents.markup.instruction') }}</code></template>
        </i18n-t>
        <i18n-t keypath="documents.markup.variable" tag="li" scope="global">
          <template #tag><code class="font-mono">// [VARIABLE] {{ t('documents.markup.instruction') }}</code></template>
        </i18n-t>
        <i18n-t keypath="documents.markup.adaptable" tag="li" scope="global">
          <template #tag><code class="font-mono">// [ADAPTABLE] {{ t('documents.markup.instruction') }}</code></template>
        </i18n-t>
      </ul>
      <i18n-t keypath="documents.markup.zones" tag="p" scope="global">
        <template #tag><code class="font-mono">[VARIABLE]</code></template>
      </i18n-t>
      <i18n-t keypath="documents.markup.must_open" tag="p" scope="global">
        <template #example><code class="font-mono">// --- {{ t('documents.markup.example_heading') }} --- [FIXE]</code></template>
      </i18n-t>
    </div>
  </details>
</template>
