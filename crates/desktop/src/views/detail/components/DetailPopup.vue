<script setup lang="ts">
/**
 * Popups du détail de candidature : rédaction du mail, ajout de recruteur,
 * contenu de l'offre, ajout d'événement (voir `state.detailPopup`).
 * Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules internes
import {
  state,
  closeDetailPopup,
  addRecruiter,
  deleteCandidature,
  applyWithoutMessage,
  applyByMail,
  applyByMessage,
} from '@/stores/app'

/** Confirme la suppression de la candidature ouverte et affiche l'erreur éventuelle. */
async function confirmDelete() {
  const error = await deleteCandidature()
  if (error) state.detailError = error
}

/** Voie de candidature en cours de traitement (désactive les trois options tant qu'elle est active). */
const applyingVia = ref<'mail' | 'message' | 'sans_message' | null>(null)

/**
 * Déclenche une des trois façons de postuler pour la candidature ouverte,
 * en bloquant les autres options le temps de l'appel (rédaction IA incluse
 * pour mail/message) et en affichant l'erreur éventuelle.
 *
 * @param via - Voie choisie, utilisée pour l'état de chargement affiché.
 * @param action - Fonction du store à invoquer.
 */
async function confirmApply(via: 'mail' | 'message' | 'sans_message', action: () => Promise<string | null>) {
  if (applyingVia.value) return
  applyingVia.value = via
  const error = await action()
  applyingVia.value = null
  if (error) state.detailError = error
}
</script>

<template>
  <div v-if="state.detailPopup" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="closeDetailPopup">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>

      <!-- Apply popup -->
      <template v-if="state.detailPopup === 'apply'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Postuler</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[18px]">Comment veux-tu postuler à cette offre ?</div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'mail' }"
          @click="confirmApply('mail', applyByMail)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">✉</div>
          <div>
            <div class="text-[14px] font-semibold">Postuler par mail</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">{{ applyingVia === 'mail' ? 'Rédaction du mail par l\'IA…' : 'Rédiger et envoyer un mail de candidature.' }}</div>
          </div>
        </div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'message' }"
          @click="confirmApply('message', applyByMessage)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">💬</div>
          <div>
            <div class="text-[14px] font-semibold">Postuler par message</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">{{ applyingVia === 'message' ? 'Rédaction du message par l\'IA…' : 'Envoyer un message via la plateforme de l\'offre.' }}</div>
          </div>
        </div>

        <div
          class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-surface-2 hover:border-neutral-300 hover:bg-surface-3"
          :class="{ 'opacity-40 pointer-events-none': applyingVia && applyingVia !== 'sans_message' }"
          @click="confirmApply('sans_message', applyWithoutMessage)"
        >
          <div class="text-[15px] w-[38px] text-center flex-none">→</div>
          <div>
            <div class="text-[14px] font-semibold">Postuler sans message</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">Marquer la candidature comme envoyée, sans message.</div>
          </div>
        </div>
      </template>

      <!-- Mail popup -->
      <template v-else-if="state.detailPopup === 'mail'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Rédiger le mail de candidature</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Objet</label>
          <input
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            :value="`Candidature — ${state.selectedCard?.role ?? ''} — Alex Martin`"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Corps du mail</label>
          <textarea class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[180px]" placeholder="Madame, Monsieur, ..." />
        </div>
        <button class="w-full justify-center mb-1 h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3">
          ✦ Générer par IA
        </button>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
            @click="closeDetailPopup"
          >Annuler</button>
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            @click="closeDetailPopup"
          >Enregistrer</button>
        </div>
      </template>

      <!-- Add recruiter popup -->
      <template v-else-if="state.detailPopup === 'addRecruiter'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Ajouter un recruteur</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Nom complet</label>
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Prénom Nom" />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Poste</label>
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. Tech Recruiter" />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Email</label>
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="contact@entreprise.com" />
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
            @click="closeDetailPopup"
          >Annuler</button>
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            @click="addRecruiter"
          >Confirmer</button>
        </div>
      </template>

      <!-- Offer content popup -->
      <template v-else-if="state.detailPopup === 'offer'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Contenu de l'offre</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="bg-neutral-50 border border-neutral-200 rounded-10 p-[18px] text-[13px] leading-[1.7] text-neutral-700 min-h-[200px] font-sans whitespace-pre-wrap">
          {{ state.detailCandidature?.description_offre || 'Aucun contenu d\'offre renseigné.' }}
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            @click="closeDetailPopup"
          >Fermer</button>
        </div>
      </template>

      <!-- Status history content popup -->
      <template v-else-if="state.detailPopup === 'statusHistoryContent'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Contenu envoyé</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="bg-neutral-50 border border-neutral-200 rounded-10 p-[18px] text-[13px] leading-[1.7] text-neutral-700 min-h-[200px] font-mono whitespace-pre-wrap">
          {{ state.detailStatusHistoryContent }}
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            @click="closeDetailPopup"
          >Fermer</button>
        </div>
      </template>

      <!-- Confirm delete popup -->
      <template v-else-if="state.detailPopup === 'confirmDelete'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Supprimer la candidature</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="text-[13px] text-neutral-600 leading-[1.6]">
          Cette candidature sera définitivement retirée de votre suivi. Cette action est irréversible.
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
            @click="closeDetailPopup"
          >Annuler</button>
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-red-600 bg-red-600 text-white hover:bg-red-700"
            @click="confirmDelete"
          >Supprimer</button>
        </div>
      </template>

    </div>
  </div>
</template>
