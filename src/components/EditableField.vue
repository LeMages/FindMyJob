<script setup lang="ts">
/**
 * Champ affiché en lecture seule, qui bascule en édition (uniquement si
 * `editable`) sur double-clic — ou sur simple clic pour un `select`, ouvert
 * directement comme un menu déroulant classique. Valide au blur ou à la
 * touche Entrée ; en `textarea`, Entrée seule valide aussi (Maj+Entrée pour
 * insérer un saut de ligne, comme Notion/Slack), annule sur Échap.
 *
 * La sauvegarde est déléguée au parent via `onSave` (fonction async qui
 * retourne un message d'erreur en cas d'échec, ou rien en cas de succès) :
 * en cas d'échec, le message s'affiche sous le champ pendant 5 secondes.
 */

// Importation des modules Vue / framework
import { computed, nextTick, onUnmounted, ref } from 'vue'

export interface EditableFieldOption {
  value: string
  label: string
}

interface Props {
  modelValue: string | null
  editable: boolean
  /** Sauvegarde la nouvelle valeur ; retourne un message d'erreur en cas d'échec. */
  onSave: (value: string) => Promise<string | null | undefined>
  type?: 'text' | 'select' | 'textarea'
  options?: EditableFieldOption[]
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  options: () => [],
  placeholder: '',
})

const ERROR_DISPLAY_MS = 5000

const isEditing = ref(false)
const draft = ref('')
const fieldRef = ref<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | null>(null)
const error = ref<string | null>(null)
let errorTimeoutId: ReturnType<typeof setTimeout> | undefined

// Champ vide : affiché comme un véritable champ de formulaire non rempli
// (placeholder natif du navigateur), jamais comme du texte inséré à la main —
// sinon le placeholder se confond visuellement avec une vraie valeur enregistrée.
const isEmpty = computed(() => !(props.modelValue ?? '').trim())

const displayLabel = computed(() => {
  if (props.type === 'select') {
    return props.options.find(o => o.value === props.modelValue)?.label ?? props.modelValue
  }
  return props.modelValue
})

const baseClass = 'w-full border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 focus:outline-none focus:border-neutral-400'
const inputClass = `${baseClass} h-[38px] block`
const textareaClass = `${baseClass} py-[10px] block resize-none leading-[1.5] h-[100px]`
const selectClass = `${inputClass} appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]`

const interactiveClass = computed(() => props.editable ? 'cursor-pointer hover:bg-surface-3' : 'cursor-default')
const emptyInputClass = computed(() => `${inputClass} ${interactiveClass.value}`)
const emptyTextareaClass = computed(() => `${textareaClass} ${interactiveClass.value}`)

const displayClass = computed(() => {
  if (props.type === 'textarea') {
    return `${baseClass} py-[10px] h-[100px] overflow-y-auto whitespace-pre-wrap leading-[1.5] ${interactiveClass.value}`
  }
  // select-none : un select est un menu à ouvrir, pas du texte à sélectionner
  // (un double-clic sur du texte le surlignait avant de basculer en édition).
  const selectable = props.type === 'select' ? 'select-none' : ''
  return `${baseClass} h-[38px] flex items-center truncate ${selectable} ${interactiveClass.value}`
})

// Un select s'ouvre au premier clic, comme un menu déroulant classique ; les
// champs texte attendent un double-clic pour éviter les entrées en édition
// accidentelles au simple clic (sélection, focus de lecture...).
const editHint = computed(() => {
  if (!props.editable) return 'Modifiable uniquement au statut « À postuler »'
  return props.type === 'select' ? 'Cliquer pour modifier' : 'Double-cliquer pour modifier'
})

/** Affiche un message d'erreur sous le champ pendant `ERROR_DISPLAY_MS`. */
function showError(message: string) {
  error.value = message
  clearTimeout(errorTimeoutId)
  errorTimeoutId = setTimeout(() => { error.value = null }, ERROR_DISPLAY_MS)
}

/** Bascule le champ en mode édition si celui-ci est modifiable, et ouvre
 * directement la liste déroulante pour un champ de type `select`. */
function startEdit() {
  if (!props.editable || isEditing.value) return
  draft.value = props.modelValue ?? ''
  isEditing.value = true
  void nextTick(() => {
    const el = fieldRef.value
    el?.focus()
    if (el instanceof HTMLSelectElement) {
      // showPicker() peut être refusé (navigateur trop ancien, hors geste
      // utilisateur) : le select reste alors focus, ouvrable au clic/clavier.
      try {
        el.showPicker()
      } catch {
        /* ignoré volontairement, voir commentaire ci-dessus */
      }
    }
  })
}

/** Valide la saisie en cours et délègue la sauvegarde au parent si la valeur a changé. */
async function commit() {
  if (!isEditing.value) return
  isEditing.value = false
  const next = draft.value.trim()
  if (next === (props.modelValue ?? '')) return
  const message = await props.onSave(next)
  if (message) showError(message)
}

/** Annule la saisie en cours sans sauvegarder. */
function cancel() {
  isEditing.value = false
}

onUnmounted(() => clearTimeout(errorTimeoutId))
</script>

<template>
  <div>
    <input
      v-if="isEditing && type === 'text'"
      ref="fieldRef"
      v-model="draft"
      :class="inputClass"
      :placeholder="placeholder"
      @blur="commit"
      @keyup.enter="commit"
      @keyup.esc="cancel"
    />
    <textarea
      v-else-if="isEditing && type === 'textarea'"
      ref="fieldRef"
      v-model="draft"
      :class="textareaClass"
      :placeholder="placeholder"
      @blur="commit"
      @keydown.enter.exact.prevent="commit"
      @keyup.esc="cancel"
    />
    <select
      v-else-if="isEditing && type === 'select'"
      ref="fieldRef"
      v-model="draft"
      :class="selectClass"
      @change="commit"
      @blur="commit"
      @keyup.esc="cancel"
    >
      <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
    </select>
    <input
      v-else-if="type === 'text' && isEmpty"
      readonly
      value=""
      :class="emptyInputClass"
      :placeholder="placeholder"
      :title="editHint"
      @dblclick="startEdit"
    />
    <textarea
      v-else-if="type === 'textarea' && isEmpty"
      readonly
      value=""
      :class="emptyTextareaClass"
      :placeholder="placeholder"
      :title="editHint"
      @dblclick="startEdit"
    />
    <div
      v-else-if="type === 'select'"
      :class="displayClass"
      :title="editHint"
      @click="startEdit"
    >{{ displayLabel }}</div>
    <div
      v-else
      :class="displayClass"
      :title="editHint"
      @dblclick="startEdit"
    >{{ displayLabel }}</div>

    <div v-if="error" class="text-[11.5px] text-red-500 font-mono mt-1">{{ error }}</div>
  </div>
</template>
