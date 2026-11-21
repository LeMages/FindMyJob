<script setup lang="ts">
/**
 * Carte « Le poste » du formulaire de nouvelle candidature : intitulé,
 * entreprise, lieu, contrat, gratification, plateforme et lien, sur deux
 * colonnes (une seule sur un écran étroit). Le sous-titre rappelle si les
 * champs viennent de l'analyse de l'offre. Propre à `NewApplicationView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import SectionCard from '@/components/SectionCard.vue'

// Importation des modules internes
import { sourceOptions, typeContratOptions } from '@/composables/candidature_display'

defineProps<{
  /** Les champs ont été pré-remplis par l'analyse d'un lien. */
  prefilled: boolean
}>()

const emit = defineEmits<{
  /** Le lien de l'offre a été modifié (vérification des doublons). */
  'url-change': []
}>()

const role = defineModel<string>('role', { required: true })
const company = defineModel<string>('company', { required: true })
const location = defineModel<string>('location', { required: true })
const contract = defineModel<string>('contract', { required: true })
const salary = defineModel<string>('salary', { required: true })
const source = defineModel<string>('source', { required: true })
const url = defineModel<string>('url', { required: true })

const { t } = useI18n()

const LABEL = 'text-[12px] text-neutral-600 mb-[5px] block'
const INPUT = 'w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400'
const SELECT = `${INPUT} appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]`
</script>

<template>
  <SectionCard
    :title="t('new_application.form.job_title')"
    :hint="prefilled ? t('new_application.form.job_hint_prefilled') : t('new_application.form.job_hint_manual')"
  >
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-[14px] [&>div]:min-w-0">
      <div class="sm:col-span-2">
        <label :class="LABEL" for="na-role">{{ t('detail.job.title') }} <span class="text-red-500 font-semibold">*</span></label>
        <input id="na-role" v-model="role" :class="INPUT" :placeholder="t('detail.job.title_placeholder')" />
      </div>
      <div>
        <label :class="LABEL" for="na-company">{{ t('detail.job.company') }} <span class="text-red-500 font-semibold">*</span></label>
        <input id="na-company" v-model="company" :class="INPUT" :placeholder="t('detail.job.company_placeholder')" />
      </div>
      <div>
        <label :class="LABEL" for="na-location">{{ t('detail.job.location') }} <span class="text-red-500 font-semibold">*</span></label>
        <input id="na-location" v-model="location" :class="INPUT" :placeholder="t('new_application.form.location_placeholder')" />
      </div>
      <div>
        <label :class="LABEL" for="na-contract">{{ t('detail.job.contract') }} <span class="text-red-500 font-semibold">*</span></label>
        <select id="na-contract" v-model="contract" :class="SELECT">
          <option value="">{{ t('common.select_placeholder') }}</option>
          <option v-for="opt in typeContratOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
      <div>
        <label :class="LABEL" for="na-salary">{{ t('detail.job.salary') }}</label>
        <input id="na-salary" v-model="salary" :class="INPUT" :placeholder="t('detail.job.salary_placeholder')" />
      </div>
      <div>
        <label :class="LABEL" for="na-source">{{ t('detail.job.platform') }} <span class="text-red-500 font-semibold">*</span></label>
        <select id="na-source" v-model="source" :class="SELECT">
          <option value="">{{ t('common.select_placeholder') }}</option>
          <option v-for="opt in sourceOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
      <div>
        <label :class="LABEL" for="na-url">{{ t('detail.job.link') }}</label>
        <input id="na-url" v-model="url" :class="INPUT" placeholder="https://..." @change="emit('url-change')" />
      </div>
    </div>
  </SectionCard>
</template>
