<script setup lang="ts">
/**
 * Assistant de première configuration (onboarding) : recueille les
 * informations du profil en plusieurs étapes puis crée l'utilisateur unique
 * de l'application.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
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

/** Avance à l'étape suivante si l'étape courante est valide. */
function goNext() {
  if (canNext.value && !isLastStep.value) {
    currentStep.value++
    errorMessage.value = null
  }
}

/** Revient à l'étape précédente. */
function goPrev() {
  if (currentStep.value > 0) {
    currentStep.value--
    errorMessage.value = null
  }
}

/** Crée l'utilisateur à partir du formulaire et notifie le parent (`completed`). */
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
    errorMessage.value = extractErrorMessage(e, 'Une erreur inattendue est survenue.')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="fixed inset-0 bg-neutral-50 flex items-center justify-center">
    <div class="bg-surface-1 border border-neutral-200 rounded-2xl py-9 px-10 w-full max-w-[480px] shadow-[0_4px_24px_rgba(0,0,0,0.07)]">

      <!-- Header -->
      <div class="text-center mb-7">
        <div class="w-[38px] h-[38px] rounded-10 bg-neutral-950 text-white inline-flex items-center justify-center font-bold text-[17px] mb-[14px]">F</div>
        <div class="text-[20px] font-bold text-neutral-950 mb-[6px]">Bienvenue dans FindMyJob</div>
        <div class="text-[13.5px] text-neutral-600">Configurons ton profil pour commencer.</div>
      </div>

      <!-- Progress dots -->
      <div class="flex items-center justify-center gap-[6px] mb-[6px]">
        <div
          v-for="(_, i) in STEPS"
          :key="i"
          class="h-2 transition-all duration-200"
          :class="i === currentStep ? 'w-6 rounded bg-neutral-950' : (i < currentStep ? 'w-2 rounded-full bg-neutral-400' : 'w-2 rounded-full bg-neutral-200')"
        />
      </div>
      <div class="font-mono text-[11px] text-neutral-400 text-center mb-6 tracking-[0.04em]">
        Étape {{ currentStep + 1 }} sur {{ STEPS.length }} — {{ STEPS[currentStep] }}
      </div>

      <!-- Step 0: Identité -->
      <div v-if="currentStep === 0" class="mb-4 min-h-[160px]">
        <div class="flex gap-[14px]">
          <div class="mb-[14px] flex-1">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Prénom <span class="text-red-500">*</span></label>
            <input
              v-model="formRaw.prenom"
              class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
              type="text"
              placeholder="Jean"
              autocomplete="given-name"
            />
          </div>
          <div class="mb-[14px] flex-1">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Nom <span class="text-red-500">*</span></label>
            <input
              v-model="formRaw.nom"
              class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
              type="text"
              placeholder="Dupont"
              autocomplete="family-name"
            />
          </div>
        </div>
      </div>

      <!-- Step 1: Contact -->
      <div v-else-if="currentStep === 1" class="mb-4 min-h-[160px]">
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">E-mail <span class="text-red-500">*</span></label>
          <input
            v-model="formRaw.email"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="email"
            placeholder="jean.dupont@email.com"
            autocomplete="email"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Téléphone</label>
          <input
            v-model="formRaw.telephone"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="tel"
            placeholder="+33 6 12 34 56 78"
            autocomplete="tel"
          />
        </div>
      </div>

      <!-- Step 2: Localisation -->
      <div v-else-if="currentStep === 2" class="mb-4 min-h-[160px]">
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Ville</label>
          <input
            v-model="formRaw.ville"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="text"
            placeholder="Paris"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Permis de conduire</label>
          <div class="flex gap-2">
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap"
              :class="formRaw.permis === true ? 'border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800' : 'border border-neutral-300 bg-surface-2 text-neutral-900 hover:bg-surface-3'"
              type="button"
              @click="formRaw.permis = formRaw.permis === true ? null : true"
            >Oui</button>
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap"
              :class="formRaw.permis === false ? 'border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800' : 'border border-neutral-300 bg-surface-2 text-neutral-900 hover:bg-surface-3'"
              type="button"
              @click="formRaw.permis = formRaw.permis === false ? null : false"
            >Non</button>
          </div>
        </div>
      </div>

      <!-- Step 3: Description -->
      <div v-else-if="currentStep === 3" class="mb-4 min-h-[160px]">
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">À propos de toi</label>
          <textarea
            v-model="formRaw.description"
            class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[140px]"
            placeholder="Étudiant·e en M2 Data Science à la recherche d'un stage…"
          />
        </div>
      </div>

      <!-- Step 4: Liens -->
      <div v-else-if="currentStep === 4" class="mb-4 min-h-[160px]">
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">LinkedIn</label>
          <input
            v-model="formRaw.linkedin"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="url"
            placeholder="linkedin.com/in/jean-dupont"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">GitHub</label>
          <input
            v-model="formRaw.github"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="url"
            placeholder="github.com/jeandupont"
          />
        </div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Portfolio</label>
          <input
            v-model="formRaw.portfolio"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="url"
            placeholder="jeandupont.dev"
          />
        </div>
      </div>

      <!-- Error -->
      <div v-if="errorMessage" class="text-[12.5px] text-red-500 bg-red-50 border border-red-200 rounded-lg px-3 py-2 mb-4">{{ errorMessage }}</div>

      <!-- Navigation -->
      <div class="flex items-center gap-[10px] mt-2">
        <button
          v-if="currentStep > 0"
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="goPrev"
        >← Précédent</button>
        <div class="flex-1 min-w-0" />
        <button
          v-if="!isLastStep"
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !canNext }"
          type="button"
          :disabled="!canNext"
          @click="goNext"
        >Suivant →</button>
        <button
          v-else
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !canNext || submitting }"
          type="button"
          :disabled="!canNext || submitting"
          @click="finish"
        >{{ submitting ? 'Création…' : 'Terminer' }}</button>
      </div>

    </div>
  </div>
</template>
