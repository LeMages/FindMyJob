<script setup lang="ts">
/**
 * Popup « Mail de candidature », présentée comme une messagerie :
 * destinataires (contacts qui ont un e-mail) en étiquettes, objet, pièces
 * jointes, grand corps de texte ; « Régénérer » à gauche, Annuler et
 * Enregistrer à droite. Comportement inchangé par la refonte : rien n'ouvre
 * encore cette popup, « Régénérer » n'a pas d'action et Enregistrer ferme
 * simplement la popup. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import { state, closeDetailPopup } from '@/stores/app'
import { buildDocumentFileName } from '@/composables/document_file_name'

const { t } = useI18n()

/** Contacts de la candidature qui ont un e-mail : destinataires du mail. */
const recipients = computed(() => state.detailContacts.filter((c) => !!c.email))

/** Noms des fichiers joints, tels qu'ils sont exportés au moment de postuler. */
const attachments = computed(() => {
  const c = state.detailCandidature
  if (!c) return []
  return (['cv', 'lm'] as const).map((docType) =>
    buildDocumentFileName(docType, state.userNom, state.userPrenom, c.entreprise, c.poste),
  )
})

const ROW = 'flex items-center gap-3 px-[14px] py-2 border-b border-neutral-100 text-[13px]'
const ROW_LABEL = 'w-24 flex-none text-neutral-500 text-[12px]'
const CHIP = 'inline-flex items-center px-[9px] py-[3px] rounded-full bg-neutral-100 text-[12px] text-neutral-700 max-w-full truncate'
</script>

<template>
  <AppModal
    v-if="state.detailPopup === 'mail'"
    :title="t('detail.mail.title')"
    :subtitle="t('detail.mail.subtitle')"
    :width="760"
    divided
    backdrop-close
    @close="closeDetailPopup"
  >
    <template #icon><UiIcon name="mail" /></template>

    <div class="border border-neutral-200 rounded-xl overflow-hidden">
      <div :class="ROW">
        <span :class="ROW_LABEL">{{ t('detail.mail.to_label') }}</span>
        <div class="flex flex-wrap gap-[6px] min-w-0">
          <span v-for="contact in recipients" :key="contact.id" :class="CHIP">{{ contact.nom }} &lt;{{ contact.email }}&gt;</span>
          <span v-if="recipients.length === 0" class="text-neutral-400">{{ t('detail.mail.no_recipient') }}</span>
        </div>
      </div>
      <div :class="ROW">
        <label :class="ROW_LABEL" for="mail-subject">{{ t('detail.mail.subject_label') }}</label>
        <input
          id="mail-subject"
          class="flex-1 min-w-0 border-0 bg-transparent font-sans text-[13.5px] text-neutral-950 outline-none h-7"
          :value="t('detail.mail.subject', { role: state.detailCandidature?.poste ?? '' })"
        />
      </div>
      <div :class="ROW">
        <span :class="ROW_LABEL">{{ t('detail.mail.attachments_label') }}</span>
        <div class="flex flex-wrap gap-[6px] min-w-0">
          <span v-for="name in attachments" :key="name" :class="CHIP">{{ name }}</span>
        </div>
      </div>
      <textarea
        class="block w-full h-[300px] border-0 resize-none p-[14px] font-sans text-[13.5px] leading-[1.6] bg-transparent text-neutral-950 outline-none"
        :placeholder="t('detail.mail.body_placeholder')"
        :aria-label="t('detail.mail.body_label')"
      />
    </div>

    <template #footer>
      <button
        type="button"
        class="h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
      ><UiIcon name="spark" /> {{ t('detail.mail.regenerate') }}</button>
      <span class="flex-1" />
      <button
        type="button"
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="closeDetailPopup"
      >{{ t('common.cancel') }}</button>
      <button
        type="button"
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
        @click="closeDetailPopup"
      >{{ t('common.save') }}</button>
    </template>
  </AppModal>
</template>
