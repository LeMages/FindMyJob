<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CreateUserInput, UserView } from '@/types/index'

const emit = defineEmits<{
  (e: 'completed', user: UserView): void
}>()

const STEPS = ['Identité', 'Contact', 'Localisation', 'À propos', 'Liens']

const currentStep = ref(0)
const errorMessage = ref<string | null>(null)
const submitting = ref(false)

const formRaw = ref({
  nom: '',
  prenom: '',
  email: '',
  telephone: '',
  ville: '',
  permis: null as boolean | null,
  description: '',
  linkedin: '',
  github: '',
  portfolio: '',
})

const canNext = computed(() => {
  if (currentStep.value === 0) {
    return formRaw.value.nom.trim() !== '' && formRaw.value.prenom.trim() !== ''
  }
  if (currentStep.value === 1) {
    return formRaw.value.email.trim() !== ''
  }
  return true
})

const isLastStep = computed(() => currentStep.value === STEPS.length - 1)

function goNext() {
  if (canNext.value && !isLastStep.value) {
    currentStep.value++
    errorMessage.value = null
  }
}

function goPrev() {
  if (currentStep.value > 0) {
    currentStep.value--
    errorMessage.value = null
  }
}

async function finish() {
  submitting.value = true
  errorMessage.value = null
  const input: CreateUserInput = {
    nom: formRaw.value.nom,
    prenom: formRaw.value.prenom,
    email: formRaw.value.email,
    telephone: formRaw.value.telephone || null,
    ville: formRaw.value.ville || null,
    permis: formRaw.value.permis,
    description: formRaw.value.description || null,
    linkedin: formRaw.value.linkedin || null,
    github: formRaw.value.github || null,
    portfolio: formRaw.value.portfolio || null,
  }
  try {
    const user = await invoke<UserView>('create_user', { input })
    emit('completed', user)
  } catch (e: unknown) {
    if (e && typeof e === 'object' && 'message' in e) {
      errorMessage.value = (e as { message: string }).message
    } else if (typeof e === 'string') {
      errorMessage.value = e
    } else {
      errorMessage.value = 'Une erreur inattendue est survenue.'
    }
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="wiz-screen">
    <div class="wiz-card">

      <!-- Header -->
      <div class="wiz-header">
        <div class="wiz-brandmark">F</div>
        <div class="wiz-title">Bienvenue dans FindMyJob</div>
        <div class="wiz-sub">Configurons ton profil pour commencer.</div>
      </div>

      <!-- Progress dots -->
      <div class="wiz-progress">
        <div
          v-for="(_, i) in STEPS"
          :key="i"
          class="wiz-dot"
          :class="{ 'wiz-dot--active': i === currentStep, 'wiz-dot--done': i < currentStep }"
        />
      </div>
      <div class="wiz-step-label">
        Étape {{ currentStep + 1 }} sur {{ STEPS.length }} — {{ STEPS[currentStep] }}
      </div>

      <!-- Step 0: Identité -->
      <div v-if="currentStep === 0" class="wiz-fields">
        <div class="frow">
          <div class="field">
            <label class="flabel">Prénom <span class="wiz-req">*</span></label>
            <input
              v-model="formRaw.prenom"
              class="input"
              type="text"
              placeholder="Jean"
              autocomplete="given-name"
            />
          </div>
          <div class="field">
            <label class="flabel">Nom <span class="wiz-req">*</span></label>
            <input
              v-model="formRaw.nom"
              class="input"
              type="text"
              placeholder="Dupont"
              autocomplete="family-name"
            />
          </div>
        </div>
      </div>

      <!-- Step 1: Contact -->
      <div v-else-if="currentStep === 1" class="wiz-fields">
        <div class="field">
          <label class="flabel">E-mail <span class="wiz-req">*</span></label>
          <input
            v-model="formRaw.email"
            class="input"
            type="email"
            placeholder="jean.dupont@email.com"
            autocomplete="email"
          />
        </div>
        <div class="field">
          <label class="flabel">Téléphone</label>
          <input
            v-model="formRaw.telephone"
            class="input"
            type="tel"
            placeholder="+33 6 12 34 56 78"
            autocomplete="tel"
          />
        </div>
      </div>

      <!-- Step 2: Localisation -->
      <div v-else-if="currentStep === 2" class="wiz-fields">
        <div class="field">
          <label class="flabel">Ville</label>
          <input
            v-model="formRaw.ville"
            class="input"
            type="text"
            placeholder="Paris"
          />
        </div>
        <div class="field">
          <label class="flabel">Permis de conduire</label>
          <div class="wiz-toggle">
            <button
              class="btn"
              :class="{ pri: formRaw.permis === true }"
              type="button"
              @click="formRaw.permis = formRaw.permis === true ? null : true"
            >Oui</button>
            <button
              class="btn"
              :class="{ pri: formRaw.permis === false }"
              type="button"
              @click="formRaw.permis = formRaw.permis === false ? null : false"
            >Non</button>
          </div>
        </div>
      </div>

      <!-- Step 3: Description -->
      <div v-else-if="currentStep === 3" class="wiz-fields">
        <div class="field">
          <label class="flabel">À propos de toi</label>
          <textarea
            v-model="formRaw.description"
            class="input"
            style="height: 140px"
            placeholder="Étudiant·e en M2 Data Science à la recherche d'un stage…"
          />
        </div>
      </div>

      <!-- Step 4: Liens -->
      <div v-else-if="currentStep === 4" class="wiz-fields">
        <div class="field">
          <label class="flabel">LinkedIn</label>
          <input
            v-model="formRaw.linkedin"
            class="input"
            type="url"
            placeholder="linkedin.com/in/jean-dupont"
          />
        </div>
        <div class="field">
          <label class="flabel">GitHub</label>
          <input
            v-model="formRaw.github"
            class="input"
            type="url"
            placeholder="github.com/jeandupont"
          />
        </div>
        <div class="field">
          <label class="flabel">Portfolio</label>
          <input
            v-model="formRaw.portfolio"
            class="input"
            type="url"
            placeholder="jeandupont.dev"
          />
        </div>
      </div>

      <!-- Error -->
      <div v-if="errorMessage" class="wiz-error">{{ errorMessage }}</div>

      <!-- Navigation -->
      <div class="wiz-nav">
        <button
          v-if="currentStep > 0"
          class="btn"
          type="button"
          @click="goPrev"
        >← Précédent</button>
        <div class="f1" />
        <button
          v-if="!isLastStep"
          class="btn pri"
          type="button"
          :class="{ disabled: !canNext }"
          :disabled="!canNext"
          @click="goNext"
        >Suivant →</button>
        <button
          v-else
          class="btn pri"
          type="button"
          :class="{ disabled: !canNext || submitting }"
          :disabled="!canNext || submitting"
          @click="finish"
        >{{ submitting ? 'Création…' : 'Terminer' }}</button>
      </div>

    </div>
  </div>
</template>

<style scoped>
.wiz-screen {
  position: fixed;
  inset: 0;
  background: #f8f8f6;
  display: flex;
  align-items: center;
  justify-content: center;
}

.wiz-card {
  background: #fff;
  border: 1px solid #e6e6e1;
  border-radius: 16px;
  padding: 36px 40px;
  width: 100%;
  max-width: 480px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.07);
}

.wiz-header {
  text-align: center;
  margin-bottom: 28px;
}

.wiz-brandmark {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: #1d1d1b;
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 17px;
  margin-bottom: 14px;
}

.wiz-title {
  font-size: 20px;
  font-weight: 700;
  color: #1d1d1b;
  margin-bottom: 6px;
}

.wiz-sub {
  font-size: 13.5px;
  color: #76766f;
}

.wiz-progress {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin-bottom: 6px;
}

.wiz-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ddddd8;
  transition: all 0.2s;
}
.wiz-dot--done {
  background: #9b9b93;
}
.wiz-dot--active {
  background: #1d1d1b;
  width: 24px;
  border-radius: 4px;
}

.wiz-step-label {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  color: #9b9b95;
  text-align: center;
  margin-bottom: 24px;
  letter-spacing: 0.04em;
}

.wiz-fields {
  margin-bottom: 16px;
  min-height: 160px;
}

.wiz-fields .field {
  margin-bottom: 14px;
}

.wiz-toggle {
  display: flex;
  gap: 8px;
}

.wiz-error {
  font-size: 12.5px;
  color: #b94a48;
  background: #fff5f5;
  border: 1px solid #f5c6cb;
  border-radius: 8px;
  padding: 8px 12px;
  margin-bottom: 16px;
}

.wiz-nav {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 8px;
}

.wiz-req {
  color: #b94a48;
}
</style>
