<script setup lang="ts">
/**
 * Champs de l'événement « Entretien planifié » : type en sélecteur segmenté,
 * date et modalité côte à côte, lieu, puis les interlocuteurs (contacts de la
 * candidature) en pastilles à cocher avec avatar et rôle. Propre à
 * `DetailView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { state } from '@/stores/app'
import { interviewTypeOptions, modaliteOptions, nameInitials } from '@/composables/candidature_display'

const type = defineModel<string>('type', { required: true })
const date = defineModel<string>('date', { required: true })
const modalite = defineModel<string>('modalite', { required: true })
const lieu = defineModel<string>('lieu', { required: true })
const contactIds = defineModel<number[]>('contactIds', { required: true })

const { t } = useI18n()

/**
 * Coche ou décoche un interlocuteur.
 *
 * @param id - Contact concerné.
 */
function toggleContact(id: number) {
  contactIds.value = contactIds.value.includes(id)
    ? contactIds.value.filter((c) => c !== id)
    : [...contactIds.value, id]
}

const LABEL = 'text-[12px] text-neutral-600 mb-[5px] block'
const INPUT = 'w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400'
const SELECT = `${INPUT} appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]`
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-[14px] [&>div]:min-w-0">
    <div class="sm:col-span-2">
      <span :class="LABEL">{{ t('detail.add_event.interview_type') }}</span>
      <div class="flex w-full flex-wrap p-[3px] rounded-9 bg-neutral-100 gap-0.5" role="radiogroup" :aria-label="t('detail.add_event.interview_type')">
        <button
          v-for="opt in interviewTypeOptions()"
          :key="opt.value"
          type="button"
          role="radio"
          :aria-checked="type === opt.value"
          class="flex-1 border-0 px-[14px] py-[5px] rounded-7 text-[12.5px] font-sans cursor-pointer whitespace-nowrap"
          :class="type === opt.value
            ? 'bg-white text-neutral-950 font-medium shadow-[0_1px_2px_rgba(0,0,0,.08)]'
            : 'bg-transparent text-neutral-600'"
          @click="type = opt.value"
        >{{ opt.label }}</button>
      </div>
    </div>
    <div>
      <label :class="LABEL" for="ae-date">{{ t('detail.add_event.date_time') }}</label>
      <input id="ae-date" v-model="date" type="datetime-local" :class="INPUT" />
    </div>
    <div>
      <label :class="LABEL" for="ae-modalite">{{ t('detail.add_event.modality') }}</label>
      <select id="ae-modalite" v-model="modalite" :class="SELECT">
        <option value="">{{ t('detail.add_event.modality_unspecified') }}</option>
        <option v-for="opt in modaliteOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
      </select>
    </div>
    <div class="sm:col-span-2">
      <label :class="LABEL" for="ae-lieu">{{ t('detail.add_event.location') }}</label>
      <input id="ae-lieu" v-model="lieu" :placeholder="t('detail.add_event.location_placeholder')" :class="INPUT" />
    </div>
    <div class="sm:col-span-2">
      <span :class="LABEL">{{ t('detail.add_event.contacts') }}</span>
      <div v-if="state.detailContacts.length > 0" class="flex flex-wrap gap-2">
        <button
          v-for="contact in state.detailContacts"
          :key="contact.id"
          type="button"
          role="checkbox"
          :aria-checked="contactIds.includes(contact.id)"
          class="inline-flex items-center gap-2 py-[5px] pr-3 pl-[6px] border rounded-full text-[12.5px] font-sans cursor-pointer"
          :class="contactIds.includes(contact.id)
            ? 'border-neutral-950 bg-neutral-950 text-white'
            : 'border-neutral-200 bg-transparent text-neutral-800'"
          @click="toggleContact(contact.id)"
        >
          <span class="w-[22px] h-[22px] rounded-full bg-neutral-200 text-neutral-700 text-[9.5px] font-semibold flex items-center justify-center flex-none">{{ nameInitials(contact.nom) }}</span>
          {{ contact.nom }}
          <span v-if="contact.role" class="text-[11.5px]" :class="contactIds.includes(contact.id) ? 'text-neutral-400' : 'text-neutral-500'">{{ contact.role }}</span>
        </button>
      </div>
      <div v-else class="text-[12px] text-neutral-500">{{ t('detail.add_event.no_contacts') }}</div>
    </div>
  </div>
</template>
