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
import DocumentReferenceEditor from '@/components/DocumentReferenceEditor.vue'

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

// Documents joints — la mise en scène (upload/sélection/glisser-déposer) est
// déléguée à DocumentReferenceEditor (mode "attachment") ; cette vue ne garde
// que le chemin résultant, lié en v-model.
const selectedCvPath = ref<string | null>(null)
const selectedLettrePath = ref<string | null>(null)

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
      cv_path: selectedCvPath.value,
      lettre_motivation_path: selectedLettrePath.value,
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
    selectedCvPath.value = null
    selectedLettrePath.value = null
    navigate('tracking')
  } catch (e: unknown) {
    saveError.value = extractErrorMessage(e, 'Erreur lors de la création de la candidature.')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="page-new">
    <button class="back" @click="navigate('tracking')">← Retour au suivi</button>

    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:20px">
      <div>
        <h1 class="h1">Nouvelle candidature</h1>
        <div class="sub">{{ currentSession?.nom }}</div>
      </div>
      <button class="btn" @click="navigate('tracking')">Annuler</button>
    </div>

    <!-- Détails du poste -->
    <div class="fieldset">
      <div class="fstitle">DÉTAILS DU POSTE</div>
      <div class="field">
        <label class="flabel">Intitulé du poste <span class="req">*</span></label>
        <input v-model="roleRef" class="input" placeholder="Ex. Stage Data Scientist" />
      </div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Entreprise <span class="req">*</span></label>
          <input v-model="companyRef" class="input" placeholder="Nom de l'entreprise" />
        </div>
        <div class="field">
          <label class="flabel">Lieu <span class="req">*</span></label>
          <input v-model="locationRef" class="input" placeholder="Ville, pays" />
        </div>
      </div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Type de contrat <span class="req">*</span></label>
          <select v-model="typeRef" class="input">
            <option value="">-- Sélectionner --</option>
            <option value="stage">Stage</option>
            <option value="alternance">Alternance</option>
            <option value="interim">Intérim</option>
            <option value="cdd">CDD</option>
            <option value="cdi">CDI</option>
          </select>
        </div>
        <div class="field">
          <label class="flabel">Gratification</label>
          <input v-model="remunerationRef" class="input" placeholder="Ex. 1 300 €/mois" />
        </div>
      </div>
      <div class="field">
        <label class="flabel">Plateforme de l'offre <span class="req">*</span></label>
        <select v-model="sourceRef" class="input">
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
      <div class="field">
        <label class="flabel">Lien de l'offre</label>
        <input v-model="urlOffreRef" class="input" placeholder="https://..." />
      </div>
      <div class="field" style="margin-bottom:0">
        <label class="flabel">Contenu de l'offre <span class="req">*</span></label>
        <textarea v-model="descriptionOffreRef" class="input" style="height:160px" placeholder="Colle ici le texte complet de l'offre d'emploi..." />
      </div>
    </div>

    <!-- Contacts recruteur -->
    <div class="fieldset">
      <div class="fstitle">CONTACTS RECRUTEUR</div>
      <div
        v-for="(ct, index) in state.newContacts"
        :key="ct.id"
        class="contact-block"
      >
        <div class="contact-hd">
          <span class="contact-n">CONTACT {{ index + 1 }}</span>
          <button v-if="state.newContacts.length > 1" class="btn-remove" @click="removeContact(index)">
            Supprimer
          </button>
        </div>
        <div class="frow">
          <div class="field">
            <label class="flabel">Nom</label>
            <input class="input" placeholder="Prénom Nom" />
          </div>
          <div class="field">
            <label class="flabel">Poste</label>
            <input class="input" placeholder="Ex. Chargé·e de recrutement" />
          </div>
          <div class="field">
            <label class="flabel">Email</label>
            <input class="input" placeholder="contact@entreprise.com" />
          </div>
        </div>
      </div>
      <button class="btn" style="width:100%;justify-content:center" @click="addContact">
        + Ajouter un contact
      </button>
    </div>

    <!-- Documents -->
    <div class="fieldset">
      <div class="fstitle">DOCUMENTS</div>
      <div class="docgrid">
        <DocumentReferenceEditor v-model="selectedCvPath" doc-type="cv" mode="attachment" />
        <DocumentReferenceEditor v-model="selectedLettrePath" doc-type="lettre_motivation" mode="attachment" />
      </div>
    </div>

    <!-- Notes -->
    <div class="fieldset">
      <div class="fstitle">NOTES</div>
      <textarea v-model="notesRef" class="input" placeholder="Notes libres, contexte, points à préparer..." />
    </div>

    <!-- Error -->
    <div v-if="saveError" style="color:#b94a48;font-size:13px;margin-bottom:8px">{{ saveError }}</div>

    <!-- Submit -->
    <button
      class="btn pri"
      style="width:100%;justify-content:center;height:48px;font-size:15px;border-radius:10px;margin-top:4px"
      :disabled="saving || !isValid"
      :class="{ disabled: saving || !isValid }"
      @click="submit"
    >
      {{ saving ? 'Enregistrement…' : 'Valider la candidature →' }}
    </button>
  </div>
</template>

<style scoped>
.req {
  color: #b94a48;
  font-weight: 600;
}
</style>
