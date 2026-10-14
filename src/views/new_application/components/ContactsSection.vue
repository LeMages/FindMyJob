<script setup lang="ts">
/**
 * Section « Contacts recruteur » du formulaire de nouvelle candidature :
 * une ligne par contact (existant du carnet ou nouveau), envoyée avec la
 * candidature. Rien n'est enregistré avant la validation du formulaire.
 * Propre à `NewApplicationView`.
 */

// Importation des modules Vue / framework
import { onMounted, ref } from 'vue'

// Importation des composants
import ContactEntryFields from '@/components/ContactEntryFields.vue'

// Importation des modules internes
import { state, addContactDraft, removeContactDraft, loadContactBook } from '@/stores/app'

const loadError = ref<string | null>(null)

onMounted(async () => {
  loadError.value = await loadContactBook()
})

/**
 * Contacts du carnet déjà choisis par les autres lignes, pour ne pas lier
 * deux fois le même contact.
 *
 * @param index - Ligne concernée.
 * @returns Identifiants à exclure du choix de cette ligne.
 */
function idsChosenElsewhere(index: number): number[] {
  return state.newContacts
    .filter((d, i) => i !== index && d.mode === 'existing' && d.contact_id !== null)
    .map(d => d.contact_id as number)
}
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">CONTACTS RECRUTEUR</div>
    <div v-if="loadError" class="text-[12.5px] text-red-600 mb-3">{{ loadError }}</div>
    <div
      v-for="(draft, index) in state.newContacts"
      :key="draft.key"
      class="border border-neutral-200 rounded-10 px-[18px] py-4 mb-3 bg-neutral-50"
    >
      <div class="flex items-center justify-between mb-3">
        <span class="text-[11px] font-semibold text-neutral-500 font-mono tracking-[.06em]">CONTACT {{ index + 1 }}</span>
        <button
          type="button"
          class="bg-transparent border-none text-[12px] text-neutral-300 cursor-pointer font-sans px-[6px] py-1 rounded-md hover:text-neutral-800 hover:bg-surface-2"
          @click="removeContactDraft(index)"
        >
          Supprimer
        </button>
      </div>
      <ContactEntryFields
        v-model="state.newContacts[index]"
        :book="state.contactBook"
        :excluded-ids="idsChosenElsewhere(index)"
      />
    </div>
    <div v-if="state.newContacts.length === 0" class="text-[12px] text-neutral-400 font-mono mb-3">Aucun contact pour cette candidature.</div>
    <button
      type="button"
      class="w-full justify-center h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
      @click="addContactDraft()"
    >
      + Ajouter un contact
    </button>
  </div>
</template>
