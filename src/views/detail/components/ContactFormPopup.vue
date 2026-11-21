<script setup lang="ts">
/**
 * Contenu de la popup de contact du détail de candidature : ajout (contact
 * existant du carnet ou nouveau contact) ou modification du contact
 * `state.detailContact`. Ouverte par `DetailPopup` quand
 * `state.detailPopup === 'contact'`.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import ContactEntryFields from '@/components/ContactEntryFields.vue'

// Importation des modules internes
import { state, closeDetailPopup, addContactToDetail, updateContact } from '@/stores/app'
import { draftFromContact, draftToContactInput, emptyContactDraft } from '@/composables/contact_draft'

const { t } = useI18n()

const editing = state.detailContact
const draft = ref(editing ? draftFromContact(editing) : emptyContactDraft())
const saving = ref(false)
const error = ref<string | null>(null)

/** Contacts déjà liés à la candidature : inutile de les proposer à l'ajout. */
const linkedIds = computed(() => state.detailContacts.map(c => c.id))

/** Enregistre l'ajout ou la modification et affiche l'erreur éventuelle. */
async function save() {
  if (saving.value) return
  saving.value = true
  error.value = editing
    ? await updateContact(editing.id, draftToContactInput(draft.value))
    : await addContactToDetail(draft.value)
  saving.value = false
}
</script>

<template>
  <AppModal :title="editing ? t('contact_form.edit_title') : t('contact_form.add_title')" :width="580" backdrop-close @close="closeDetailPopup">
    <div v-if="editing && editing.candidature_count > 1" class="text-[12px] text-neutral-500 font-mono mb-4">
      {{ t('contact_form.shared_warning', editing.candidature_count) }}
    </div>

    <ContactEntryFields
      v-model="draft"
      :book="state.contactBook"
      :allow-existing="!editing"
      :excluded-ids="linkedIds"
    />

    <div v-if="error" class="text-[12.5px] text-red-600 mt-1">{{ error }}</div>

    <template #footer>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="closeDetailPopup"
      >{{ t('common.cancel') }}</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-50 disabled:cursor-default"
        :disabled="saving"
        @click="save"
      >{{ saving ? t('common.saving') : t('common.save') }}</button>
    </template>
  </AppModal>
</template>
