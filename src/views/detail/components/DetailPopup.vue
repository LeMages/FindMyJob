<script setup lang="ts">
/**
 * Popups du détail de candidature : ajout ou modification d'un contact,
 * suppression définitive d'un contact, contenu de l'offre, contenu d'un
 * événement du suivi, suppression de la candidature (voir
 * `state.detailPopup`). « Postuler » et « Mail de candidature » ont leur
 * propre popup (`ApplyPopup`, `MailPopup`). Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import ContactFormPopup from './ContactFormPopup.vue'

// Importation des modules internes
import {
  state,
  closeDetailPopup,
  deleteContact,
  deleteCandidature,
} from '@/stores/app'

const { t } = useI18n()

/** Confirme la suppression de la candidature ouverte et affiche l'erreur éventuelle. */
async function confirmDelete() {
  const error = await deleteCandidature()
  if (error) state.detailError = error
}

/** Erreur de la suppression définitive d'un contact. */
const deleteContactError = ref<string | null>(null)

/** Nombre d'autres candidatures liées au contact à supprimer. */
const otherCandidatureCount = computed(() => {
  const contact = state.detailContact
  if (!contact) return 0
  const linkedHere = state.detailContacts.some(c => c.id === contact.id)
  return contact.candidature_count - (linkedHere ? 1 : 0)
})

/** Supprime définitivement le contact `state.detailContact` et affiche l'erreur éventuelle. */
async function confirmDeleteContact() {
  const contact = state.detailContact
  if (!contact) return
  deleteContactError.value = await deleteContact(contact.id)
}

/** Titre de chaque variante de la popup (le contact, Postuler et le mail ont leur propre popup). */
const POPUP_TITLES: Record<string, string> = {
  confirmDeleteContact: 'detail.delete_contact.title',
  offer: 'detail.offer.title',
  statusHistoryContent: 'detail.sent_content.title',
  confirmDelete: 'detail.delete_application.title',
}

const popupTitle = computed(() => {
  const key = state.detailPopup ? POPUP_TITLES[state.detailPopup] : undefined
  return key ? t(key) : null
})
</script>

<template>
  <!-- Contact popup (ajout ou modification) : popup autonome -->
  <ContactFormPopup v-if="state.detailPopup === 'contact'" />

  <AppModal v-else-if="state.detailPopup && popupTitle" :title="popupTitle" :width="580" backdrop-close @close="closeDetailPopup">

      <!-- Confirm delete contact popup -->
      <template v-if="state.detailPopup === 'confirmDeleteContact' && state.detailContact">
        <div class="text-[13px] text-neutral-600 leading-[1.6]">
          {{ t('detail.delete_contact.body', { name: state.detailContact.nom }) }}
          <template v-if="otherCandidatureCount > 0">
            {{ t('detail.delete_contact.also_linked', otherCandidatureCount) }}
          </template>
          {{ t('common.irreversible') }}
        </div>
        <div v-if="deleteContactError" class="text-[12.5px] text-red-600 mt-3">{{ deleteContactError }}</div>
      </template>

      <!-- Offer content popup -->
      <template v-else-if="state.detailPopup === 'offer'">
        <div class="bg-neutral-50 border border-neutral-200 rounded-10 p-[18px] text-[13px] leading-[1.7] text-neutral-700 min-h-[200px] font-sans whitespace-pre-wrap">
          {{ state.detailCandidature?.description_offre || t('detail.offer.empty') }}
        </div>
      </template>

      <!-- Status history content popup -->
      <template v-else-if="state.detailPopup === 'statusHistoryContent'">
        <div class="bg-neutral-50 border border-neutral-200 rounded-10 p-[18px] text-[13px] leading-[1.7] text-neutral-700 min-h-[200px] font-mono whitespace-pre-wrap">
          {{ state.detailStatusHistoryContent }}
        </div>
      </template>

      <!-- Confirm delete popup -->
      <template v-else-if="state.detailPopup === 'confirmDelete'">
        <div class="text-[13px] text-neutral-600 leading-[1.6]">
          {{ t('detail.delete_application.body') }}
        </div>
      </template>

    <template #footer>
      <template v-if="state.detailPopup === 'confirmDeleteContact'">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="closeDetailPopup"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-red-600 bg-red-600 text-white hover:bg-red-700"
          @click="confirmDeleteContact"
        >{{ t('detail.delete_contact.confirm') }}</button>
      </template>
      <template v-else-if="state.detailPopup === 'offer' || state.detailPopup === 'statusHistoryContent'">
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          @click="closeDetailPopup"
        >{{ t('common.close') }}</button>
      </template>
      <template v-else-if="state.detailPopup === 'confirmDelete'">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="closeDetailPopup"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-red-600 bg-red-600 text-white hover:bg-red-700"
          @click="confirmDelete"
        >{{ t('common.delete') }}</button>
      </template>
    </template>
  </AppModal>
</template>
