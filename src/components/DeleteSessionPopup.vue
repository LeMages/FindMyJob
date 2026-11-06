<script setup lang="ts">
/**
 * Popup globale de confirmation de suppression de session (voir
 * `state.deleteSessionPopup`).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { state, confirmDeleteSession, cancelDeleteSession } from '@/stores/app'

const { t } = useI18n()
</script>

<template>
  <div
    v-if="state.deleteSessionPopup"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]"
    @click="cancelDeleteSession"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">{{ t('session.delete.title') }}</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="cancelDeleteSession">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[6px]">{{ t('common.irreversible') }}</div>
      <div class="bg-neutral-50 border border-neutral-200 rounded-9 px-[14px] py-3 mb-5 text-[13.5px]">
        <i18n-t keypath="session.delete.body" tag="span" scope="global">
          <template #name><strong>{{ state.deleteSessionPopup.nom }}</strong></template>
        </i18n-t>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="cancelDeleteSession"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-surface-3 hover:text-neutral-800"
          @click="confirmDeleteSession"
        >{{ t('common.delete') }}</button>
      </div>
    </div>
  </div>
</template>
