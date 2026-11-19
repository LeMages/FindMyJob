<script setup lang="ts">
/**
 * Popups du détail de candidature : rédaction du mail, ajout ou
 * modification d'un contact, suppression définitive d'un contact, contenu
 * de l'offre, ajout d'événement (voir `state.detailPopup`).
 * Propre à `DetailView`.
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
  applyWithoutMessage,
  applyByMail,
  applyByMessage,
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

/** Voie de candidature en cours de traitement (désactive les trois options tant qu'elle est active). */
const applyingVia = ref<'mail' | 'message' | 'sans_message' | null>(null)

/** Titre de chaque variante de la popup (le contact a sa propre popup). */
const POPUP_TITLES: Record<string, string> = {
  apply: 'detail.apply.title',
  mail: 'detail.mail.title',
  confirmDeleteContact: 'detail.delete_contact.title',
  offer: 'detail.offer.title',
  statusHistoryContent: 'detail.sent_content.title',
  confirmDelete: 'detail.delete_application.title',
}

const popupTitle = computed(() => {
  const key = state.detailPopup ? POPUP_TITLES[state.detailPopup] : undefined
  return key ? t(key) : null
})

/** Toutes les variantes ont un pied, sauf le choix de la voie d'envoi. */
const hasFooter = computed(() => state.detailPopup !== 'apply')

/**
 * Déclenche une des trois façons de postuler pour la candidature ouverte,
 * en bloquant les autres options le temps de l'appel (rédaction IA incluse
 * pour mail/message) et en affichant l'erreur éventuelle.
 *
 * @param via - Voie choisie, utilisée pour l'état de chargement affiché.
 * @param action - Fonction du store à invoquer.
 */
async function confirmApply(via: 'mail' | 'message' | 'sans_message', action: () => Promise<string | null>) {
  if (applyingVia.value) return
  applyingVia.value = via
  const error = await action()
  applyingVia.value = null
  if (error) state.detailError = error
}
</script>

<template>
  <!-- Contact popup (ajout ou modification) : popup autonome -->
  <ContactFormPopup v-if="state.detailPopup === 'contact'" />

  <AppModal v-else-if="state.detailPopup && popupTitle" :title="popupTitle" :width="580" backdrop-close @close="closeDetailPopup">

      <!-- Apply popup -->
      <template v-if="state.detailPopup === 'apply'">
        <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[18px]">{{ t('detail.apply.question') }}</div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'mail' }"
          @click="confirmApply('mail', applyByMail)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">✉</div>
          <div>
            <div class="text-[14px] font-semibold">{{ t('detail.apply.mail_title') }}</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">{{ applyingVia === 'mail' ? t('detail.apply.mail_writing') : t('detail.apply.mail_description') }}</div>
          </div>
        </div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'message' }"
          @click="confirmApply('message', applyByMessage)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">💬</div>
          <div>
            <div class="text-[14px] font-semibold">{{ t('detail.apply.message_title') }}</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">{{ applyingVia === 'message' ? t('detail.apply.message_writing') : t('detail.apply.message_description') }}</div>
          </div>
        </div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'sans_message' }"
          @click="confirmApply('sans_message', applyWithoutMessage)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">→</div>
          <div>
            <div class="text-[14px] font-semibold">{{ t('detail.apply.none_title') }}</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">{{ t('detail.apply.none_description') }}</div>
          </div>
        </div>
      </template>

      <!-- Mail popup -->
      <template v-else-if="state.detailPopup === 'mail'">
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.mail.subject_label') }}</label>
          <input
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            :value="t('detail.mail.subject', { role: state.selectedCard?.role ?? '' })"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.mail.body_label') }}</label>
          <textarea class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[180px]" :placeholder="t('detail.mail.body_placeholder')" />
        </div>
        <button class="w-full justify-center mb-1 h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3">
          {{ t('detail.mail.generate') }}
        </button>
      </template>

      <!-- Confirm delete contact popup -->
      <template v-else-if="state.detailPopup === 'confirmDeleteContact' && state.detailContact">
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

    <template v-if="hasFooter" #footer>
      <template v-if="state.detailPopup === 'mail'">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="closeDetailPopup"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          @click="closeDetailPopup"
        >{{ t('common.save') }}</button>
      </template>
      <template v-else-if="state.detailPopup === 'confirmDeleteContact'">
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
