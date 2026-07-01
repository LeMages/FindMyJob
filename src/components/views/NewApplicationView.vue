<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import type { CandidatureView, CreateCandidatureInput, CvView } from '@/types'
import {
  state,
  currentSession,
  navigate,
  addContact,
  removeContact,
} from '@/stores/app'

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

// CV selection
const selectedCv = ref<CvView | null>(null)
const cvLoading = ref(false)
const cvError = ref<string | null>(null)
const cvDragOver = ref(false)

async function handleCvFile(file: File) {
  if (!file.name.toLowerCase().endsWith('.pdf')) {
    cvError.value = 'Seuls les fichiers PDF sont acceptés.'
    return
  }
  cvLoading.value = true
  cvError.value = null
  try {
    const buffer = await file.arrayBuffer()
    const uint8 = new Uint8Array(buffer)
    let binary = ''
    for (let i = 0; i < uint8.length; i++) binary += String.fromCharCode(uint8[i])
    const content = btoa(binary)
    selectedCv.value = await invoke<CvView>('upload_candidature_cv', { filename: file.name, content })
  } catch (e: unknown) {
    cvError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors du téléversement du CV.'
  } finally {
    cvLoading.value = false
  }
}

function onCvDrop(event: DragEvent) {
  cvDragOver.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) handleCvFile(file)
}

async function browseCv() {
  cvError.value = null
  try {
    const dir = await invoke<string>('get_cv_reference_dir')
    const path = await openDialog({
      title: 'Choisir un CV',
      defaultPath: dir,
      multiple: false,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    })
    if (!path || Array.isArray(path)) return
    cvLoading.value = true
    selectedCv.value = await invoke<CvView>('select_candidature_cv', { path })
  } catch (e: unknown) {
    cvError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors de la sélection du CV.'
  } finally {
    cvLoading.value = false
  }
}

function clearSelectedCv() {
  selectedCv.value = null
  cvError.value = null
}

async function submit() {
  if (!isValid.value) return
  saving.value = true
  saveError.value = null
  try {
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
      cv_id: selectedCv.value?.id ?? null,
      session_id: currentSession.value?.id ?? null,
    }
    await invoke<CandidatureView>('create_candidature', { input })
    roleRef.value = ''
    companyRef.value = ''
    locationRef.value = ''
    typeRef.value = ''
    sourceRef.value = ''
    remunerationRef.value = ''
    urlOffreRef.value = ''
    descriptionOffreRef.value = ''
    notesRef.value = ''
    selectedCv.value = null
    navigate('tracking')
  } catch (e: unknown) {
    saveError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors de la création de la candidature.'
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
        <div>
          <div class="doczonetitle">Curriculum Vitae</div>
          <div v-if="selectedCv" class="doc">
            <div class="docicon docicon-file">
              <svg width="14" height="17" viewBox="0 0 14 17" fill="none">
                <path d="M1.5 1.5h7l4 4v9.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1v-12.5a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
                <path d="M8.5 1.5v4h4" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
              </svg>
            </div>
            <div class="f1">
              <div class="crole">{{ selectedCv.label ?? 'CV sélectionné' }}</div>
              <div class="cmeta">Joint</div>
            </div>
            <button class="eyebtn" type="button" title="Retirer" @click="clearSelectedCv">✕</button>
          </div>
          <template v-else>
            <div
              class="doczone"
              :style="cvDragOver ? 'border-color:#539bf5' : ''"
              @dragover.prevent="cvDragOver = true"
              @dragleave.prevent="cvDragOver = false"
              @drop.prevent="onCvDrop"
            >
              <div class="droptext">→ Glisser le PDF ici</div>
              <button class="btn" type="button" :disabled="cvLoading" @click="browseCv">
                {{ cvLoading ? 'Chargement…' : 'Parcourir' }}
              </button>
            </div>
            <button class="btn" type="button" style="width:100%;justify-content:center;margin-top:8px">
              ✦ Générer mon CV
            </button>
          </template>
          <div v-if="cvError" style="color:#b94a48;font-size:12px;margin-top:4px">{{ cvError }}</div>
        </div>
        <div>
          <div class="doczonetitle">Lettre de motivation</div>
          <div class="doczone">
            <div class="droptext">→ Glisser le PDF ici</div>
            <button class="btn">Parcourir</button>
          </div>
          <button class="btn" style="width:100%;justify-content:center;margin-top:8px">
            ✦ Générer ma lettre
          </button>
        </div>
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
