<script setup lang="ts">
/**
 * Vue « Détail » d'une candidature : informations du poste, statut et suivi
 * (événements), documents joints, contacts recruteur et historique.
 */

// Importation des modules internes
import {
  state,
  selectedCardColumn,
  navigate,
  openDetailPopup,
  removeEvent,
  trashCard,
  acceptCard,
  cancelAcceptation,
} from '@/stores/app'
import DetailPopup from '@/views/detail/components/DetailPopup.vue'

// Libellés d'affichage des événements de suivi
const eventLabel = (type: string) => {
  if (type === 'mail_relance') return 'Mail de relance'
  if (type === 'mail_reponse') return 'Mail de réponse'
  return 'Entretien'
}
const eventIcon = (type: string) => {
  if (type === 'mail_relance') return 'ML'
  if (type === 'mail_reponse') return 'MR'
  return 'RV'
}
const eventAction = (type: string) => type === 'entretien' ? 'Préparer →' : 'Rédiger →'
const initials = (nom: string) =>
  nom.split(' ').map(w => w[0] ?? '').join('').slice(0, 2).toUpperCase()
</script>

<template>
  <div class="page">
    <button class="back" @click="navigate('tracking')">← Retour au suivi</button>

    <div style="margin-bottom:4px">
      <h1 class="h1">{{ state.selectedCard?.role }}</h1>
      <div class="sub">{{ state.selectedCard?.company }} · {{ state.selectedCard?.location }}</div>
    </div>

    <div class="dgrid">
      <!-- Left column -->
      <div>
        <!-- Détails du poste -->
        <div class="fieldset">
          <div class="fstitle-row">
            <div class="fstitle">DÉTAILS DU POSTE</div>
            <button class="btn" @click="openDetailPopup('offer')">Voir l'offre →</button>
          </div>
          <div class="field">
            <label class="flabel">Intitulé du poste</label>
            <input class="input" :value="state.selectedCard?.role" placeholder="Ex. Stage Data Scientist" />
          </div>
          <div class="frow">
            <div class="field">
              <label class="flabel">Entreprise</label>
              <input class="input" :value="state.selectedCard?.company" placeholder="Nom de l'entreprise" />
            </div>
            <div class="field">
              <label class="flabel">Lieu</label>
              <input class="input" :value="state.selectedCard?.location" placeholder="Ville" />
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
            <label class="flabel">Plateforme de l'offre</label>
            <select class="input">
              <option>Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)</option>
              <option>Site carrière de l'entreprise (formulaire sur le site)</option>
              <option>Mail (candidature directe par email)</option>
              <option>Cooptation / Réseau (recommandation, contact perso)</option>
              <option>Forum / Salon emploi (rencontre physique)</option>
              <option>École (job board, offres des profs / asso)</option>
              <option>Autre</option>
            </select>
          </div>
          <div class="field" style="margin-bottom:0">
            <label class="flabel">Lien de l'offre</label>
            <input class="input" placeholder="https://..." />
          </div>
        </div>

        <!-- Notes -->
        <div class="fieldset">
          <div class="fstitle">NOTES</div>
          <textarea class="input" style="height:100px" placeholder="Notes libres, retours d'entretien, contacts..." />
        </div>

        <!-- Statut & Suivi -->
        <div class="fieldset">
          <div class="fstitle" style="margin-bottom:14px">STATUT &amp; SUIVI</div>

          <!-- Mail de candidature (always first) -->
          <div class="event-card">
            <div class="event-icon">✉</div>
            <div class="event-body">
              <div class="event-title">Mail de candidature</div>
              <div class="event-meta">Par défaut · En attente d'envoi</div>
            </div>
            <button class="btn pri" @click="openDetailPopup('mail')">Rédiger le mail →</button>
          </div>

          <!-- Dynamic events -->
          <div v-for="ev in state.detailEvents" :key="ev.id" class="event-card">
            <div class="event-icon" style="font-family:'IBM Plex Mono',monospace;font-size:10px;font-weight:700;color:#56564f">
              {{ eventIcon(ev.type) }}
            </div>
            <div class="event-body">
              <div class="event-title">{{ eventLabel(ev.type) }}</div>
              <div class="event-meta">{{ ev.type === 'entretien' ? 'Date à définir' : 'En attente' }}</div>
            </div>
            <div class="btnrow">
              <button class="btn" @click="openDetailPopup('mail')">{{ eventAction(ev.type) }}</button>
              <button class="btn" @click="removeEvent(ev.id)" style="padding:0 10px;color:#bcbcb5">✕</button>
            </div>
          </div>

          <button class="btn" style="width:100%;justify-content:center;margin-top:4px" @click="openDetailPopup('addEvent')">
            + Ajouter un événement
          </button>
        </div>
      </div>

      <!-- Right column -->
      <div>
        <!-- Documents -->
        <div class="fieldset">
          <div class="fstitle">DOCUMENTS</div>
          <div class="doc">
            <div class="docicon ph" />
            <div class="f1">
              <div class="crole">CV_AlexMartin_2025.pdf</div>
              <div class="cmeta">Joint · 184 Ko</div>
            </div>
            <button class="eyebtn" title="Visualiser">
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>
          <div class="doc">
            <div class="docicon ph" />
            <div class="f1">
              <div class="crole">Lettre_motivation.pdf</div>
              <div class="cmeta">Joint · 92 Ko</div>
            </div>
            <button class="eyebtn" title="Visualiser">
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Contacts recruteur -->
        <div class="fieldset">
          <div class="fstitle-row">
            <div class="fstitle">CONTACTS RECRUTEUR</div>
            <button class="btn" @click="openDetailPopup('addRecruiter')">+ Ajouter</button>
          </div>
          <div v-for="rec in state.detailRecruiters" :key="rec.id" class="recruiter-card">
            <div class="rec-avatar">{{ initials(rec.nom) }}</div>
            <div class="f1">
              <div class="crole">{{ rec.nom }}</div>
              <div class="cco">{{ rec.poste }}</div>
              <div class="cmeta">{{ rec.email }}</div>
            </div>
          </div>
        </div>

        <!-- Historique -->
        <div class="fieldset">
          <div class="fstitle">HISTORIQUE</div>
          <div class="tl">
            <div class="tli">
              <div class="tlcol">
                <div class="tlmark on" />
                <div class="tlbar" />
              </div>
              <div>
                <div class="tlt">Candidature créée</div>
                <div class="tld">{{ state.selectedCard?.date ?? '—' }}</div>
              </div>
            </div>
            <div class="tli">
              <div class="tlcol">
                <div class="tlmark" :class="{ on: selectedCardColumn !== 'À postuler' }" />
                <div class="tlbar" />
              </div>
              <div>
                <div class="tlt">Candidature envoyée</div>
                <div class="tld">{{ selectedCardColumn !== 'À postuler' ? selectedCardColumn : 'À planifier' }}</div>
              </div>
            </div>
            <div class="tli">
              <div class="tlcol">
                <div class="tlmark" :class="{ on: selectedCardColumn === 'Entretien' || selectedCardColumn === 'Accepté' }" />
              </div>
              <div>
                <div class="tlt">Entretien</div>
                <div class="tld">{{ selectedCardColumn === 'Entretien' || selectedCardColumn === 'Accepté' ? 'Planifié' : 'À planifier' }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Action footer -->
  <div v-if="selectedCardColumn === 'Accepté'" class="action-footer">
    <div style="flex:1;font-size:13px;color:#2d5a2d;font-family:'IBM Plex Mono',monospace">
      ✓ Candidature acceptée — session verrouillée.
    </div>
    <button class="btn" @click="cancelAcceptation">Annuler l'acceptation</button>
  </div>
  <div v-else class="action-footer">
    <button class="btn-refuse" @click="trashCard">Refusé entreprise</button>
    <button class="btn-refuse" @click="trashCard">Refus de ma part</button>
    <button class="btn-accept" @click="acceptCard">Acceptation</button>
  </div>

  <!-- Popups -->
  <DetailPopup />
</template>
