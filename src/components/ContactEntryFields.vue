<script setup lang="ts">
/**
 * Saisie d'un contact recruteur : choix d'un contact existant du carnet ou
 * création d'un nouveau contact (nom, rôle, email, téléphone, notes).
 * Partagé par le formulaire de nouvelle candidature et la popup de contact
 * du détail. Le trim et la validation du nom sont faits par le backend.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import type { ContactDraft, ContactView } from '@/types'

const { t } = useI18n()

const draft = defineModel<ContactDraft>({ required: true })

const props = withDefaults(defineProps<{
  /** Carnet de contacts proposé en mode « contact existant ». */
  book: ContactView[]
  /** Autorise le choix d'un contact existant (désactivé en modification). */
  allowExisting?: boolean
  /** Contacts à ne pas proposer (déjà liés à la candidature, ou choisis ailleurs). */
  excludedIds?: number[]
}>(), {
  allowExisting: true,
  excludedIds: () => [],
})

/** Contacts du carnet proposables (le contact déjà choisi par cette ligne reste proposé). */
const availableContacts = computed(() =>
  props.book.filter(c => c.id === draft.value.contact_id || !props.excludedIds.includes(c.id)),
)

const showModeChoice = computed(() => props.allowExisting && availableContacts.value.length > 0)

/**
 * Change le mode de saisie de la ligne.
 *
 * @param mode - `existing` pour choisir dans le carnet, `new` pour saisir un nouveau contact.
 */
function setMode(mode: ContactDraft['mode']) {
  draft.value.mode = mode
  if (mode === 'new') draft.value.contact_id = null
}

const INPUT_CLASS = 'w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400'
const MODE_CLASS = 'h-8 px-3 rounded-lg border text-[12.5px] font-medium cursor-pointer font-sans'
const MODE_ACTIVE = 'border-neutral-950 bg-neutral-950 text-white'
const MODE_IDLE = 'border-neutral-300 bg-surface-2 text-neutral-900 hover:bg-surface-3'
</script>

<template>
  <div>
    <div v-if="showModeChoice" class="flex gap-2 mb-[14px]">
      <button
        type="button"
        :class="[MODE_CLASS, draft.mode === 'existing' ? MODE_ACTIVE : MODE_IDLE]"
        @click="setMode('existing')"
      >{{ t('contact_form.existing') }}</button>
      <button
        type="button"
        :class="[MODE_CLASS, draft.mode === 'new' ? MODE_ACTIVE : MODE_IDLE]"
        @click="setMode('new')"
      >{{ t('contact_form.new') }}</button>
    </div>

    <div v-if="showModeChoice && draft.mode === 'existing'" class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.from_book') }} <span class="text-red-500 font-semibold">*</span></label>
      <select v-model="draft.contact_id" :class="INPUT_CLASS">
        <option :value="null" disabled>{{ t('contact_form.pick') }}</option>
        <option v-for="c in availableContacts" :key="c.id" :value="c.id">
          {{ c.nom }}{{ c.role ? ` — ${c.role}` : '' }}{{ c.email ? ` (${c.email})` : '' }}
        </option>
      </select>
    </div>

    <template v-else>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.name') }} <span class="text-red-500 font-semibold">*</span></label>
          <input v-model="draft.nom" :class="INPUT_CLASS" :placeholder="t('contact_form.name_placeholder')" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.role') }}</label>
          <input v-model="draft.role" :class="INPUT_CLASS" :placeholder="t('contact_form.role_placeholder')" />
        </div>
      </div>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.email') }}</label>
          <input v-model="draft.email" type="email" :class="INPUT_CLASS" :placeholder="t('contact_form.email_placeholder')" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.phone') }}</label>
          <input v-model="draft.telephone" type="tel" :class="INPUT_CLASS" :placeholder="t('contact_form.phone_placeholder')" />
        </div>
      </div>
      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('contact_form.notes') }}</label>
        <textarea
          v-model="draft.notes"
          class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-20"
          :placeholder="t('contact_form.notes_placeholder')"
        />
      </div>
    </template>
  </div>
</template>
