<script setup lang="ts">
/**
 * Popups du détail de candidature : rédaction du mail, ajout de recruteur,
 * contenu de l'offre, ajout d'événement (voir `state.detailPopup`).
 * Propre à `DetailView`.
 */

// Importation des modules internes
import { state, closeDetailPopup, addEvent, addRecruiter } from '@/stores/app'
</script>

<template>
  <div v-if="state.detailPopup" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="closeDetailPopup">
    <div class="bg-white rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>

      <!-- Mail popup -->
      <template v-if="state.detailPopup === 'mail'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Rédiger le mail de candidature</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Objet</label>
          <input
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            :value="`Candidature — ${state.selectedCard?.role ?? ''} — Alex Martin`"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Corps du mail</label>
          <textarea class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-[180px]" placeholder="Madame, Monsieur, ..." />
        </div>
        <button class="w-full justify-center mb-1 h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100">
          ✦ Générer par IA
        </button>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
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
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Prénom Nom" />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Poste</label>
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Ex. Tech Recruiter" />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Email</label>
          <input class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="contact@entreprise.com" />
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
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
        <div class="bg-neutral-50 border border-neutral-200 rounded-10 p-[18px] text-[13px] leading-[1.7] text-neutral-700 min-h-[200px] font-mono">
          Le contenu complet de l'offre apparaît ici, renseigné à la création de la candidature.
        </div>
        <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            @click="closeDetailPopup"
          >Fermer</button>
        </div>
      </template>

      <!-- Add event popup -->
      <template v-else-if="state.detailPopup === 'addEvent'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Ajouter un événement</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="closeDetailPopup">×</button>
        </div>
        <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[18px]">Quel type d'événement veux-tu ajouter ?</div>

        <div class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-neutral-50 hover:border-neutral-300 hover:bg-neutral-100" @click="addEvent('mail_relance')">
          <div class="text-[13px] w-[38px] text-center font-mono font-bold text-neutral-700 bg-neutral-100 rounded-7 py-[6px] flex-none">ML</div>
          <div>
            <div class="text-[14px] font-semibold">Mail de relance</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">Relancer le recruteur après un délai sans réponse.</div>
          </div>
        </div>

        <div class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-neutral-50 hover:border-neutral-300 hover:bg-neutral-100" @click="addEvent('mail_reponse')">
          <div class="text-[13px] w-[38px] text-center font-mono font-bold text-neutral-700 bg-neutral-100 rounded-7 py-[6px] flex-none">MR</div>
          <div>
            <div class="text-[14px] font-semibold">Mail de réponse</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">Enregistrer une réponse reçue du recruteur.</div>
          </div>
        </div>

        <div class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-neutral-50 hover:border-neutral-300 hover:bg-neutral-100" @click="addEvent('entretien')">
          <div class="text-[13px] w-[38px] text-center font-mono font-bold text-neutral-700 bg-neutral-100 rounded-7 py-[6px] flex-none">RV</div>
          <div>
            <div class="text-[14px] font-semibold">Entretien</div>
            <div class="text-[12px] text-neutral-500 mt-0.5">Planifier et préparer un entretien téléphonique ou visio.</div>
          </div>
        </div>
      </template>

    </div>
  </div>
</template>
