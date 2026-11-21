<script setup lang="ts">
/**
 * Carte « Contacts recruteur » de la fiche candidature : avatar, nom, rôle,
 * e-mail, téléphone et notes de chaque contact, avec modifier, retirer de la
 * candidature et supprimer définitivement en icônes (infobulle, suppression
 * rouge au survol). Ajout par « + Ajouter ». Tout est désactivé, avec
 * infobulle, quand la session est en lecture seule. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import IconButton from '@/components/IconButton.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import { state, openContactPopup, openDeleteContactPopup, unlinkContact } from '@/stores/app'
import { nameInitials } from '@/composables/candidature_display'

defineProps<{
  /** Infobulle des actions bloquées (session en lecture seule), sinon `null`. */
  lockedHint: string | null
}>()

const { t } = useI18n()

/** Erreur du dernier retrait de contact. */
const contactActionError = ref<string | null>(null)

/**
 * Retire un contact de la candidature (il reste dans le carnet).
 *
 * @param contactId - Contact à retirer.
 */
async function removeContactFromCandidature(contactId: number) {
  contactActionError.value = await unlinkContact(contactId)
}
</script>

<template>
  <SectionCard :title="t('detail.contacts.title')">
    <template #actions>
      <span class="inline-flex" :title="lockedHint ?? undefined">
        <button
          type="button"
          class="h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
          :disabled="lockedHint !== null"
          @click="openContactPopup(null)"
        ><UiIcon name="plus" /> {{ t('detail.contacts.add') }}</button>
      </span>
    </template>

    <div
      v-for="(contact, i) in state.detailContacts"
      :key="contact.id"
      class="group flex gap-3"
      :class="i > 0 ? 'py-3 border-t border-neutral-100' : 'pb-3'"
    >
      <div class="w-[34px] h-[34px] rounded-full flex-none bg-neutral-200 text-neutral-700 flex items-center justify-center font-semibold text-[12px]">{{ nameInitials(contact.nom) }}</div>
      <div class="flex-1 min-w-0">
        <div class="text-[13.5px] font-semibold text-neutral-950">{{ contact.nom }}</div>
        <div v-if="contact.role" class="text-[12px] text-neutral-500">{{ contact.role }}</div>
        <div v-if="contact.email" class="text-[11.5px] text-neutral-600 font-mono truncate">{{ contact.email }}</div>
        <div v-if="contact.telephone" class="text-[11.5px] text-neutral-600 font-mono truncate">{{ contact.telephone }}</div>
        <div v-if="contact.notes" class="text-[12px] text-neutral-600 mt-[5px] whitespace-pre-wrap">{{ contact.notes }}</div>
      </div>
      <div class="flex gap-0.5 opacity-55 group-hover:opacity-100 group-focus-within:opacity-100 self-start">
        <IconButton :title="t('common.edit')" :tooltip="lockedHint" :disabled="lockedHint !== null" @click="openContactPopup(contact)">
          <UiIcon name="edit" />
        </IconButton>
        <IconButton :title="t('detail.contacts.unlink')" :tooltip="lockedHint" :disabled="lockedHint !== null" @click="removeContactFromCandidature(contact.id)">
          <UiIcon name="unlink" />
        </IconButton>
        <IconButton :title="t('detail.contacts.delete')" :tooltip="lockedHint" danger :disabled="lockedHint !== null" @click="openDeleteContactPopup(contact)">
          <UiIcon name="trash" />
        </IconButton>
      </div>
    </div>
    <div v-if="state.detailContacts.length === 0" class="text-[12px] text-neutral-500 mt-2">{{ t('detail.contacts.empty') }}</div>
    <div v-if="contactActionError" class="text-[12.5px] text-red-600 mt-2">{{ contactActionError }}</div>
  </SectionCard>
</template>
