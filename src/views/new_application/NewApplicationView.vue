<script setup lang="ts">
/**
 * Vue « Nouvelle candidature » : formulaire de création (détails du poste,
 * contacts recruteur, CV et lettre de motivation joints).
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import type { CandidatureView, CreateCandidatureInput } from '@/types'
import {
  state,
  currentSession,
  navigate,
  addContact,
  removeContact,
} from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'

const roleRef = ref('')
const companyRef = ref('')
const locationRef = ref('')
const typeRef = ref('')
const sourceRef = ref('')
const remunerationRef = ref('')
const urlOffreRef = ref('')
const descriptionOffreRef = ref('')
const notesRef = ref('')

const isValid = computed(() =>
  roleRef.value.trim() !== '' &&
  companyRef.value.trim() !== '' &&
  locationRef.value.trim() !== '' &&
  typeRef.value !== '' &&
  sourceRef.value !== '' &&
  descriptionOffreRef.value.trim() !== ''
)

const saving = ref(false)
const saveError = ref<string | null>(null)

/** Valide et envoie le formulaire de nouvelle candidature au backend. */
async function submit() {
  if (!isValid.value) return
  saving.value = true
  saveError.value = null
  try {
    // 1. Construction de l'input à partir du formulaire
    const input: CreateCandidatureInput = {
      poste: roleRef.value.trim(),
      entreprise: companyRef.value.trim() || null,
      localisation: locationRef.value.trim() || null,
      type_contrat: typeRef.value.trim() || null,
      source: sourceRef.value || null,
      remuneration: remunerationRef.value.trim() || null,
      url_offre: urlOffreRef.value.trim() || null,
      description_offre: descriptionOffreRef.value.trim() || null,
      notes: notesRef.value.trim() || null,
      cv_path: null,
      lettre_motivation_path: null,
      session_id: currentSession.value?.id ?? null,
    }
    // 2. Création de la candidature côté backend
    await invoke<CandidatureView>('create_candidature', { input })
    // 3. Réinitialisation du formulaire et retour au suivi
    roleRef.value = ''
    companyRef.value = ''
    locationRef.value = ''
    typeRef.value = ''
    sourceRef.value = ''
    remunerationRef.value = ''
    urlOffreRef.value = ''
    descriptionOffreRef.value = ''
    notesRef.value = ''
    navigate('tracking')
  } catch (e: unknown) {
    saveError.value = extractErrorMessage(e, 'Erreur lors de la création de la candidature.')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="px-9 pt-[26px] pb-11">
    <button class="bg-transparent border-none font-mono text-[12px] text-neutral-500 cursor-pointer py-1 mb-[14px] inline-block hover:text-neutral-950" @click="navigate('tracking')">← Retour au suivi</button>

    <div class="flex items-center justify-between mb-5">
      <div>
        <h1 class="text-2xl font-semibold leading-tight">Nouvelle candidature</h1>
        <div class="text-[13px] text-neutral-500 mt-[5px] font-mono">{{ currentSession?.nom }}</div>
      </div>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        @click="navigate('tracking')"
      >Annuler</button>
    </div>

    <!-- Détails du poste -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DÉTAILS DU POSTE</div>
      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Intitulé du poste <span class="text-red-500 font-semibold">*</span></label>
        <input v-model="roleRef" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. Stage Data Scientist" />
      </div>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Entreprise <span class="text-red-500 font-semibold">*</span></label>
          <input v-model="companyRef" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Nom de l'entreprise" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Lieu <span class="text-red-500 font-semibold">*</span></label>
          <input v-model="locationRef" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ville, pays" />
        </div>
      </div>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Type de contrat <span class="text-red-500 font-semibold">*</span></label>
          <select
            v-model="typeRef"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
          >
            <option value="">-- Sélectionner --</option>
            <option value="stage">Stage</option>
            <option value="alternance">Alternance</option>
            <option value="interim">Intérim</option>
            <option value="cdd">CDD</option>
            <option value="cdi">CDI</option>
          </select>
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Gratification</label>
          <input v-model="remunerationRef" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. 1 300 €/mois" />
        </div>
      </div>
      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Plateforme de l'offre <span class="text-red-500 font-semibold">*</span></label>
        <select
          v-model="sourceRef"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
        >
          <option value="">-- Sélectionner --</option>
          <option>Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)</option>
          <option>Site carrière de l'entreprise (formulaire sur le site)</option>
          <option>Mail (candidature directe par email)</option>
          <option>Cooptation / Réseau (recommandation, contact perso)</option>
          <option>Forum / Salon emploi (rencontre physique)</option>
          <option>École (job board, offres des profs / asso)</option>
          <option>Autre</option>
        </select>
      </div>
      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Lien de l'offre</label>
        <input v-model="urlOffreRef" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://..." />
      </div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Contenu de l'offre <span class="text-red-500 font-semibold">*</span></label>
        <textarea v-model="descriptionOffreRef" class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-40" placeholder="Colle ici le texte complet de l'offre d'emploi..." />
      </div>
    </div>

    <!-- Contacts recruteur -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">CONTACTS RECRUTEUR</div>
      <div
        v-for="(ct, index) in state.newContacts"
        :key="ct.id"
        class="border border-neutral-200 rounded-10 px-[18px] py-4 mb-3 bg-neutral-50"
      >
        <div class="flex items-center justify-between mb-3">
          <span class="text-[11px] font-semibold text-neutral-500 font-mono tracking-[.06em]">CONTACT {{ index + 1 }}</span>
          <button
            v-if="state.newContacts.length > 1"
            class="bg-transparent border-none text-[12px] text-neutral-300 cursor-pointer font-sans px-[6px] py-1 rounded-md hover:text-neutral-800 hover:bg-neutral-100"
            @click="removeContact(index)"
          >
            Supprimer
          </button>
        </div>
        <div class="flex gap-[14px]">
          <div class="flex-1 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Nom</label>
            <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Prénom Nom" />
          </div>
          <div class="flex-1 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Poste</label>
            <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. Chargé·e de recrutement" />
          </div>
          <div class="flex-1 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Email</label>
            <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="contact@entreprise.com" />
          </div>
        </div>
      </div>
      <button
        class="w-full justify-center h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        @click="addContact"
      >
        + Ajouter un contact
      </button>
    </div>

    <!-- Notes -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">NOTES</div>
      <textarea v-model="notesRef" class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-20" placeholder="Notes libres, contexte, points à préparer..." />
    </div>

    <!-- Error -->
    <div v-if="saveError" class="text-red-500 text-[13px] mb-2">{{ saveError }}</div>

    <!-- Submit -->
    <button
      class="w-full justify-center h-12 text-[15px] rounded-10 mt-1 px-4 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
      :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': saving || !isValid }"
      :disabled="saving || !isValid"
      @click="submit"
    >
      {{ saving ? 'Enregistrement…' : 'Valider la candidature →' }}
    </button>
  </div>
</template>
