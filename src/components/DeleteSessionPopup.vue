<script setup lang="ts">
/**
 * Popup globale de confirmation de suppression de session (voir
 * `state.deleteSessionPopup`).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'

// Importation des modules internes
import { state, confirmDeleteSession, cancelDeleteSession } from '@/stores/app'

const { t } = useI18n()
</script>

<template>
  <AppModal v-if="state.deleteSessionPopup" :title="t('session.delete.title')" :width="440" backdrop-close @close="cancelDeleteSession">

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[6px]">{{ t('common.irreversible') }}</div>
      <div class="bg-neutral-50 border border-neutral-200 rounded-9 px-[14px] py-3 mb-3 text-[13.5px]">
        <i18n-t keypath="session.delete.body" tag="span" scope="global">
          <template #name><strong>{{ state.deleteSessionPopup.nom }}</strong></template>
        </i18n-t>
      </div>

    <template #footer>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="cancelDeleteSession"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-surface-3 hover:text-neutral-800"
          @click="confirmDeleteSession"
        >{{ t('common.delete') }}</button>
    </template>
  </AppModal>
</template>
