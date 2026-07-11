<script setup lang="ts">
/**
 * Vue « Paramètres » : configuration de l'IA, des relances, de l'apparence,
 * de la signature et du stockage. Interface de maquette — les champs ne sont
 * pas encore persistés (aucune commande Tauri branchée).
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

const showApiKey = ref(false)
const iaMode = ref<'cloud' | 'local'>('cloud')
const theme = ref<'clair' | 'sombre' | 'systeme'>('clair')
const relanceDelay = ref(7)
</script>

<template>
  <div class="page">
    <h1 class="h1">Paramètres</h1>
    <div class="sub" style="margin-bottom:22px">Configuration de l'application FindMyJob.</div>

    <!-- IA -->
    <div class="fieldset">
      <div class="fstitle">INTELLIGENCE ARTIFICIELLE</div>

      <div class="field">
        <label class="flabel">Clé API Ollama Cloud</label>
        <div style="display:flex;gap:8px">
          <input
            class="input"
            :type="showApiKey ? 'text' : 'password'"
            value="sk-ollama-xxxxxxxxxxxxxxxxxxx"
            style="flex:1;font-family:'IBM Plex Mono',monospace"
          />
          <button class="btn" @click="showApiKey = !showApiKey">
            {{ showApiKey ? 'Masquer' : 'Afficher' }}
          </button>
        </div>
        <div class="cmeta" style="margin-top:6px">Stockée dans le trousseau de l'OS — jamais en base de données.</div>
      </div>

      <div class="frow">
        <div class="field">
          <label class="flabel">Modèle</label>
          <div class="select">
            <span>llama3.2</span>
            <span class="cmeta">▾</span>
          </div>
        </div>

        <div class="field">
          <label class="flabel">Mode IA</label>
          <div style="display:flex;gap:8px">
            <div
              style="flex:1;height:38px;border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:13px;font-weight:600;cursor:pointer"
              :style="iaMode === 'cloud'
                ? 'border:2px solid #1d1d1b;background:#1d1d1b;color:#fff'
                : 'border:1px solid #e2e2dc;color:#bcbcb5'"
              @click="iaMode = 'cloud'"
            >
              Cloud
            </div>
            <div
              style="flex:1;height:38px;border:1px solid #e2e2dc;border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:13px;color:#bcbcb5;cursor:not-allowed;gap:6px"
            >
              Local
              <span style="font-size:10px;font-family:'IBM Plex Mono',monospace;background:#f1f1ee;padding:2px 6px;border-radius:4px">bientôt</span>
            </div>
          </div>
        </div>
      </div>

      <div class="setrow" style="border-bottom:none;padding:8px 0 0">
        <div>
          <div class="setl">Tester la connexion</div>
          <div class="setd">Vérifie que la clé et le modèle sélectionné répondent correctement.</div>
        </div>
        <button class="btn">Tester →</button>
      </div>
    </div>

    <!-- Relances -->
    <div class="fieldset">
      <div class="fstitle">RELANCES</div>
      <div class="field" style="margin-bottom:0">
        <label class="flabel">Délai avant relance suggérée</label>
        <div style="display:flex;align-items:center;gap:12px">
          <input
            v-model="relanceDelay"
            class="input"
            type="number"
            style="width:90px"
          />
          <span class="sub" style="margin:0">jours sans réponse avant d'afficher une suggestion de relance.</span>
        </div>
      </div>
    </div>

    <!-- Apparence -->
    <div class="fieldset">
      <div class="fstitle">APPARENCE</div>
      <div class="field" style="margin-bottom:0">
        <label class="flabel">Thème</label>
        <div style="display:flex;gap:8px">
          <div
            v-for="t in (['clair', 'sombre', 'systeme'] as const)"
            :key="t"
            style="flex:1;height:38px;border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:13px;cursor:pointer;text-transform:capitalize"
            :style="theme === t
              ? 'border:2px solid #1d1d1b;font-weight:600'
              : 'border:1px solid #d4d4ce'"
            @click="theme = t"
          >
            {{ t.charAt(0).toUpperCase() + t.slice(1) }}
          </div>
        </div>
      </div>
    </div>

    <!-- Signature -->
    <div class="fieldset">
      <div class="fstitle">SIGNATURE DE CANDIDATURE</div>
      <div class="field" style="margin-bottom:0">
        <label class="flabel">Texte de clôture des mails</label>
        <textarea class="input">Cordialement, Alex Martin</textarea>
      </div>
    </div>

    <!-- Données & stockage -->
    <div class="fieldset">
      <div class="fstitle">DONNÉES &amp; STOCKAGE</div>

      <div class="field">
        <label class="flabel">Base de données</label>
        <div style="display:flex;gap:8px">
          <input
            class="input"
            value="~/Library/Application Support/FindMyJob/db.sqlite"
            style="flex:1;font-family:'IBM Plex Mono',monospace;font-size:12px;color:#83837e"
            readonly
          />
          <button class="btn">Ouvrir le dossier</button>
        </div>
      </div>

      <div class="field">
        <label class="flabel">Dossier documents</label>
        <div style="display:flex;gap:8px">
          <input
            class="input"
            value="~/Library/Application Support/FindMyJob/documents/"
            style="flex:1;font-family:'IBM Plex Mono',monospace;font-size:12px;color:#83837e"
            readonly
          />
          <button class="btn">Ouvrir le dossier</button>
        </div>
      </div>

      <div class="setrow" style="padding:10px 0">
        <div>
          <div class="setl">Sauvegarde / Export</div>
          <div class="setd">Exporter toutes les données en JSON pour archivage.</div>
        </div>
        <button class="btn">Exporter</button>
      </div>

      <div class="setrow" style="border-bottom:none;padding:10px 0">
        <div>
          <div class="setl">Réinitialiser l'application</div>
          <div class="setd">Supprimer toutes les données locales. Action irréversible.</div>
        </div>
        <button class="btn-refuse">Réinitialiser</button>
      </div>
    </div>
  </div>
</template>
