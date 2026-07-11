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
  <div v-if="state.detailPopup" class="popup-backdrop" @click="closeDetailPopup">
    <div class="popup-box" @click.stop>

      <!-- Mail popup -->
      <template v-if="state.detailPopup === 'mail'">
        <div class="popup-hd">
          <div class="popup-title">Rédiger le mail de candidature</div>
          <button class="popup-close" @click="closeDetailPopup">×</button>
        </div>
        <div class="field">
          <label class="flabel">Objet</label>
          <input
            class="input"
            :value="`Candidature — ${state.selectedCard?.role ?? ''} — Alex Martin`"
          />
        </div>
        <div class="field">
          <label class="flabel">Corps du mail</label>
          <textarea class="input" style="height:180px" placeholder="Madame, Monsieur, ..." />
        </div>
        <button class="btn" style="width:100%;justify-content:center;margin-bottom:4px">
          ✦ Générer par IA
        </button>
        <div class="popup-foot">
          <button class="btn" @click="closeDetailPopup">Annuler</button>
          <button class="btn pri" @click="closeDetailPopup">Enregistrer</button>
        </div>
      </template>

      <!-- Add recruiter popup -->
      <template v-else-if="state.detailPopup === 'addRecruiter'">
        <div class="popup-hd">
          <div class="popup-title">Ajouter un recruteur</div>
          <button class="popup-close" @click="closeDetailPopup">×</button>
        </div>
        <div class="field">
          <label class="flabel">Nom complet</label>
          <input class="input" placeholder="Prénom Nom" />
        </div>
        <div class="field">
          <label class="flabel">Poste</label>
          <input class="input" placeholder="Ex. Tech Recruiter" />
        </div>
        <div class="field">
          <label class="flabel">Email</label>
          <input class="input" placeholder="contact@entreprise.com" />
        </div>
        <div class="popup-foot">
          <button class="btn" @click="closeDetailPopup">Annuler</button>
          <button class="btn pri" @click="addRecruiter">Confirmer</button>
        </div>
      </template>

      <!-- Offer content popup -->
      <template v-else-if="state.detailPopup === 'offer'">
        <div class="popup-hd">
          <div class="popup-title">Contenu de l'offre</div>
          <button class="popup-close" @click="closeDetailPopup">×</button>
        </div>
        <div style="background:#f8f8f6;border:1px solid #e6e6e1;border-radius:10px;padding:18px;font-size:13px;line-height:1.7;color:#56564f;min-height:200px;font-family:'IBM Plex Mono',monospace">
          Le contenu complet de l'offre apparaît ici, renseigné à la création de la candidature.
        </div>
        <div class="popup-foot">
          <button class="btn pri" @click="closeDetailPopup">Fermer</button>
        </div>
      </template>

      <!-- Add event popup -->
      <template v-else-if="state.detailPopup === 'addEvent'">
        <div class="popup-hd">
          <div class="popup-title">Ajouter un événement</div>
          <button class="popup-close" @click="closeDetailPopup">×</button>
        </div>
        <div class="sub" style="margin:-4px 0 18px">Quel type d'événement veux-tu ajouter ?</div>

        <div class="evt-tile" @click="addEvent('mail_relance')">
          <div class="evt-tile-icon">ML</div>
          <div>
            <div class="evt-tile-title">Mail de relance</div>
            <div class="evt-tile-desc">Relancer le recruteur après un délai sans réponse.</div>
          </div>
        </div>

        <div class="evt-tile" @click="addEvent('mail_reponse')">
          <div class="evt-tile-icon">MR</div>
          <div>
            <div class="evt-tile-title">Mail de réponse</div>
            <div class="evt-tile-desc">Enregistrer une réponse reçue du recruteur.</div>
          </div>
        </div>

        <div class="evt-tile" @click="addEvent('entretien')">
          <div class="evt-tile-icon">RV</div>
          <div>
            <div class="evt-tile-title">Entretien</div>
            <div class="evt-tile-desc">Planifier et préparer un entretien téléphonique ou visio.</div>
          </div>
        </div>
      </template>

    </div>
  </div>
</template>
