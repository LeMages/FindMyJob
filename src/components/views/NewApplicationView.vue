<script setup lang="ts">
import { ref } from 'vue'
import {
  state,
  currentSession,
  navigate,
  addContact,
  removeContact,
  validateNewApp,
} from '@/stores/app'

const roleRef = ref('')
const companyRef = ref('')
const locationRef = ref('')

function submit() {
  validateNewApp(roleRef.value, companyRef.value, locationRef.value)
  roleRef.value = ''
  companyRef.value = ''
  locationRef.value = ''
}
</script>

<template>
  <div class="page-new">
    <button class="back" @click="navigate('tracking')">← Retour au suivi</button>

    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:20px">
      <div>
        <h1 class="h1">Nouvelle candidature</h1>
        <div class="sub">{{ currentSession?.name }}</div>
      </div>
      <button class="btn" @click="navigate('tracking')">Annuler</button>
    </div>

    <!-- Détails du poste -->
    <div class="fieldset">
      <div class="fstitle">DÉTAILS DU POSTE</div>
      <div class="field">
        <label class="flabel">Intitulé du poste</label>
        <input v-model="roleRef" class="input" placeholder="Ex. Stage Data Scientist" />
      </div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Entreprise</label>
          <input v-model="companyRef" class="input" placeholder="Nom de l'entreprise" />
        </div>
        <div class="field">
          <label class="flabel">Lieu</label>
          <input v-model="locationRef" class="input" placeholder="Ville, pays" />
        </div>
      </div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Type de contrat</label>
          <input class="input" placeholder="Stage / Alternance / CDI" />
        </div>
        <div class="field">
          <label class="flabel">Gratification</label>
          <input class="input" placeholder="Ex. 1 300 €/mois" />
        </div>
      </div>
      <div class="field">
        <label class="flabel">Lien de l'offre</label>
        <input class="input" placeholder="https://..." />
      </div>
      <div class="field" style="margin-bottom:0">
        <label class="flabel">Contenu de l'offre</label>
        <textarea class="input" style="height:160px" placeholder="Colle ici le texte complet de l'offre d'emploi..." />
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
          <div class="doczone">
            <div class="droptext">→ Glisser le PDF ici</div>
            <button class="btn">Parcourir</button>
          </div>
          <button class="btn" style="width:100%;justify-content:center;margin-top:8px">
            ✦ Générer mon CV
          </button>
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
      <textarea class="input" placeholder="Notes libres, contexte, points à préparer..." />
    </div>

    <!-- Submit -->
    <button
      class="btn pri"
      style="width:100%;justify-content:center;height:48px;font-size:15px;border-radius:10px;margin-top:4px"
      @click="submit"
    >
      Valider la candidature →
    </button>
  </div>
</template>
