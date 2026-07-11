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
  <div class="px-9 pt-[26px] pb-11">
    <button class="bg-transparent border-none font-mono text-[12px] text-neutral-500 cursor-pointer py-1 mb-[14px] inline-block hover:text-neutral-950" @click="navigate('tracking')">← Retour au suivi</button>

    <div class="mb-1">
      <h1 class="text-2xl font-semibold leading-tight">{{ state.selectedCard?.role }}</h1>
      <div class="text-[13px] text-neutral-500 mt-[5px] font-mono">{{ state.selectedCard?.company }} · {{ state.selectedCard?.location }}</div>
    </div>

    <div class="grid grid-cols-[1fr_320px] gap-6 mt-[18px] items-start">
      <!-- Left column -->
      <div>
        <!-- Détails du poste -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">DÉTAILS DU POSTE</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
              @click="openDetailPopup('offer')"
            >Voir l'offre →</button>
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Intitulé du poste</label>
            <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" :value="state.selectedCard?.role" placeholder="Ex. Stage Data Scientist" />
          </div>
          <div class="flex gap-[14px]">
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Entreprise</label>
              <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" :value="state.selectedCard?.company" placeholder="Nom de l'entreprise" />
            </div>
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Lieu</label>
              <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" :value="state.selectedCard?.location" placeholder="Ville" />
            </div>
          </div>
          <div class="flex gap-[14px]">
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Type de contrat</label>
              <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Stage / Alternance / CDI" />
            </div>
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Gratification</label>
              <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. 1 300 €/mois" />
            </div>
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Plateforme de l'offre</label>
            <select class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]">
              <option>Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)</option>
              <option>Site carrière de l'entreprise (formulaire sur le site)</option>
              <option>Mail (candidature directe par email)</option>
              <option>Cooptation / Réseau (recommandation, contact perso)</option>
              <option>Forum / Salon emploi (rencontre physique)</option>
              <option>École (job board, offres des profs / asso)</option>
              <option>Autre</option>
            </select>
          </div>
          <div class="mb-0">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Lien de l'offre</label>
            <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://..." />
          </div>
        </div>

        <!-- Notes -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">NOTES</div>
          <textarea class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-[100px]" placeholder="Notes libres, retours d'entretien, contacts..." />
        </div>

        <!-- Statut & Suivi -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">STATUT &amp; SUIVI</div>

          <!-- Mail de candidature (always first) -->
          <div class="flex items-center gap-3 bg-neutral-50 border border-neutral-200 rounded-10 px-[14px] py-3 mb-[10px]">
            <div class="w-9 h-9 rounded-9 bg-white border border-neutral-200 flex items-center justify-center text-[15px] flex-none">✉</div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold">Mail de candidature</div>
              <div class="text-[12px] text-neutral-500 font-mono mt-0.5">Par défaut · En attente d'envoi</div>
            </div>
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
              @click="openDetailPopup('mail')"
            >Rédiger le mail →</button>
          </div>

          <!-- Dynamic events -->
          <div v-for="ev in state.detailEvents" :key="ev.id" class="flex items-center gap-3 bg-neutral-50 border border-neutral-200 rounded-10 px-[14px] py-3 mb-[10px]">
            <div class="w-9 h-9 rounded-9 bg-white border border-neutral-200 flex items-center justify-center flex-none font-mono text-[10px] font-bold text-neutral-700">
              {{ eventIcon(ev.type) }}
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold">{{ eventLabel(ev.type) }}</div>
              <div class="text-[12px] text-neutral-500 font-mono mt-0.5">{{ ev.type === 'entretien' ? 'Date à définir' : 'En attente' }}</div>
            </div>
            <div class="flex items-center gap-[10px]">
              <button
                class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
                @click="openDetailPopup('mail')"
              >{{ eventAction(ev.type) }}</button>
              <button
                class="h-9 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100 px-[10px] text-neutral-300"
                @click="removeEvent(ev.id)"
              >✕</button>
            </div>
          </div>

          <button
            class="w-full justify-center mt-1 h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
            @click="openDetailPopup('addEvent')"
          >
            + Ajouter un événement
          </button>
        </div>
      </div>

      <!-- Right column -->
      <div>
        <!-- Documents -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DOCUMENTS</div>
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">CV_AlexMartin_2025.pdf</div>
              <div class="font-mono text-[11px] text-neutral-500">Joint · 184 Ko</div>
            </div>
            <button class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-neutral-100" title="Visualiser">
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">Lettre_motivation.pdf</div>
              <div class="font-mono text-[11px] text-neutral-500">Joint · 92 Ko</div>
            </div>
            <button class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-neutral-100" title="Visualiser">
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Contacts recruteur -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">CONTACTS RECRUTEUR</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
              @click="openDetailPopup('addRecruiter')"
            >+ Ajouter</button>
          </div>
          <div v-for="rec in state.detailRecruiters" :key="rec.id" class="flex items-center gap-3 py-[10px] border-b border-neutral-100">
            <div class="w-[34px] h-[34px] rounded-full flex-none bg-neutral-200 flex items-center justify-center font-semibold text-[12px] text-neutral-700">{{ initials(rec.nom) }}</div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">{{ rec.nom }}</div>
              <div class="text-[12px] text-neutral-500 mt-0.5">{{ rec.poste }}</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ rec.email }}</div>
            </div>
          </div>
        </div>

        <!-- Historique -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">HISTORIQUE</div>
          <div class="flex flex-col">
            <div class="flex gap-3">
              <div class="flex flex-col items-center">
                <div class="w-[13px] h-[13px] rounded-full border-2 flex-none mt-0.5 bg-neutral-950 border-neutral-950" />
                <div class="flex-1 w-0.5 bg-neutral-200 min-h-[22px] my-0.5" />
              </div>
              <div>
                <div class="text-[13px] font-semibold">Candidature créée</div>
                <div class="text-[11.5px] text-neutral-400 mt-px font-mono pb-[14px]">{{ state.selectedCard?.date ?? '—' }}</div>
              </div>
            </div>
            <div class="flex gap-3">
              <div class="flex flex-col items-center">
                <div
                  class="w-[13px] h-[13px] rounded-full border-2 bg-white flex-none mt-0.5"
                  :class="selectedCardColumn !== 'À postuler' ? 'bg-neutral-950 border-neutral-950' : 'border-neutral-300'"
                />
                <div class="flex-1 w-0.5 bg-neutral-200 min-h-[22px] my-0.5" />
              </div>
              <div>
                <div class="text-[13px] font-semibold">Candidature envoyée</div>
                <div class="text-[11.5px] text-neutral-400 mt-px font-mono pb-[14px]">{{ selectedCardColumn !== 'À postuler' ? selectedCardColumn : 'À planifier' }}</div>
              </div>
            </div>
            <div class="flex gap-3">
              <div class="flex flex-col items-center">
                <div
                  class="w-[13px] h-[13px] rounded-full border-2 bg-white flex-none mt-0.5"
                  :class="selectedCardColumn === 'Entretien' || selectedCardColumn === 'Accepté' ? 'bg-neutral-950 border-neutral-950' : 'border-neutral-300'"
                />
              </div>
              <div>
                <div class="text-[13px] font-semibold">Entretien</div>
                <div class="text-[11.5px] text-neutral-400 mt-px font-mono pb-[14px]">{{ selectedCardColumn === 'Entretien' || selectedCardColumn === 'Accepté' ? 'Planifié' : 'À planifier' }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Action footer -->
  <div v-if="selectedCardColumn === 'Accepté'" class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <div class="flex-1 text-[13px] text-green-700 font-mono">
      ✓ Candidature acceptée — session verrouillée.
    </div>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
      @click="cancelAcceptation"
    >Annuler l'acceptation</button>
  </div>
  <div v-else class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-neutral-100 hover:text-neutral-800"
      @click="trashCard"
    >Refusé entreprise</button>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-neutral-100 hover:text-neutral-800"
      @click="trashCard"
    >Refus de ma part</button>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-950 bg-neutral-950 text-[13px] font-medium text-white inline-flex items-center cursor-pointer font-sans hover:bg-neutral-800"
      @click="acceptCard"
    >Acceptation</button>
  </div>

  <!-- Popups -->
  <DetailPopup />
</template>
