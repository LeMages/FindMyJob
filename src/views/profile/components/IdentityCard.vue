<script setup lang="ts">
/**
 * Carte « Identité » du profil : avatar, nom, langue du profil et coordonnées
 * en liste à icônes ; en édition, formulaire des informations de contact,
 * des liens, de la langue du profil et du permis. Le brouillon est tenu en
 * local jusqu'à l'appel de `save()` par le parent (bouton Valider).
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import ContactLine from './ContactLine.vue'
import ProfileCard from './ProfileCard.vue'
import ProfileEditButton from './ProfileEditButton.vue'

// Importation des types
import type { UpdateUserInfoInput, UserView } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const props = defineProps<{
  user: UserView
  mode: ProfileCardMode
  saving: boolean
  error: string | null
  /** Erreur d'ouverture d'un lien du profil. */
  linkError: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
  'open-url': [url: string]
}>()

const { t } = useI18n()

interface UserInfoDraft {
  prenom: string
  nom: string
  email: string
  telephone: string
  permis: boolean
  linkedin: string
  github: string
  portfolio: string
  langue_profil: string
}

/**
 * Brouillon initialisé depuis l'utilisateur enregistré.
 *
 * @param u - Utilisateur enregistré.
 * @returns Brouillon éditable.
 */
function draftFrom(u: UserView): UserInfoDraft {
  return {
    prenom: u.prenom ?? '',
    nom: u.nom ?? '',
    email: u.email ?? '',
    telephone: u.telephone ?? '',
    permis: u.permis ?? false,
    linkedin: u.linkedin ?? '',
    github: u.github ?? '',
    portfolio: u.portfolio ?? '',
    langue_profil: u.langue_profil,
  }
}

const draft = ref<UserInfoDraft>(draftFrom(props.user))

/** Repart de l'utilisateur enregistré (Annuler, ou entrée en édition). */
function reset() {
  draft.value = draftFrom(props.user)
}

watch(() => props.mode, (mode) => {
  if (mode === 'editing') reset()
})

const initials = computed(() => ((props.user.prenom?.[0] ?? '') + (props.user.nom?.[0] ?? '')).toUpperCase() || '?')
const fullName = computed(() => [props.user.prenom, props.user.nom].filter(Boolean).join(' ') || '—')
const writtenIn = computed(() =>
  props.user.langue_profil === 'en' ? t('profile.identity.written_in_en') : t('profile.identity.written_in_fr'),
)

const isInfoDirty = computed(() => {
  const d = draft.value
  const u = props.user
  return (
    d.prenom !== (u.prenom ?? '') ||
    d.nom !== (u.nom ?? '') ||
    d.email !== (u.email ?? '') ||
    d.telephone !== (u.telephone ?? '') ||
    d.permis !== (u.permis ?? false) ||
    d.linkedin !== (u.linkedin ?? '') ||
    d.github !== (u.github ?? '') ||
    d.portfolio !== (u.portfolio ?? '')
  )
})
const isLanguageDirty = computed(() => draft.value.langue_profil !== props.user.langue_profil)
const hasChanges = computed(() => isInfoDirty.value || isLanguageDirty.value)

/**
 * Vérifie qu'une valeur non vide est une URL valide.
 *
 * @param value - Valeur saisie (déjà trim).
 * @returns `true` si `value` est vide ou constitue une URL valide.
 */
function isValidUrlOrEmpty(value: string): boolean {
  if (!value) return true
  try {
    new URL(value)
    return true
  } catch {
    return false
  }
}

/**
 * Valide le brouillon puis enregistre les informations de contact et la
 * langue du profil, si elles ont changé.
 *
 * @returns Utilisateur à jour (à assigner par le parent).
 */
async function save(): Promise<UserView> {
  let updated = props.user
  if (isInfoDirty.value) {
    const d = draft.value
    const linkedin = d.linkedin.trim()
    const github = d.github.trim()
    const portfolio = d.portfolio.trim()
    if (!d.prenom.trim()) throw new Error(t('profile.info.first_name_required'))
    if (!d.nom.trim()) throw new Error(t('profile.info.last_name_required'))
    if (!isValidUrlOrEmpty(linkedin)) throw new Error(t('profile.links.invalid_linkedin'))
    if (!isValidUrlOrEmpty(github)) throw new Error(t('profile.links.invalid_github'))
    if (!isValidUrlOrEmpty(portfolio)) throw new Error(t('profile.links.invalid_portfolio'))

    const input: UpdateUserInfoInput = {
      prenom: d.prenom.trim(),
      nom: d.nom.trim(),
      email: d.email.trim() || null,
      telephone: d.telephone.trim() || null,
      permis: d.permis,
      linkedin: linkedin || null,
      github: github || null,
      portfolio: portfolio || null,
    }
    updated = await invoke<UserView>('update_user_info', { input })
  }
  if (isLanguageDirty.value) {
    updated = await invoke<UserView>('update_profile_language', { langue: draft.value.langue_profil })
  }
  return updated
}

defineExpose({ hasChanges, reset, save })

const inputClass = 'w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400'
</script>

<template>
  <ProfileCard
    :title="t('profile.info.section')"
    :mode="mode"
    :dirty="hasChanges"
    :saving="saving"
    :error="error"
    @edit="emit('edit')"
    @cancel="emit('cancel')"
    @save="emit('save')"
  >
    <!-- Lecture : en-tête avatar + nom, coordonnées en liste à icônes -->
    <template v-if="mode !== 'editing'" #header>
      <div class="flex items-start gap-[14px]">
        <div class="w-14 h-14 rounded-full flex-none flex items-center justify-center font-semibold text-[19px] bg-neutral-200 text-neutral-700">{{ initials }}</div>
        <div class="flex-1 min-w-0 pt-[3px]">
          <div class="text-[18px] font-semibold leading-tight break-words">{{ fullName }}</div>
          <div class="font-mono text-[11px] text-neutral-500 mt-[5px]">{{ writtenIn }}</div>
        </div>
        <ProfileEditButton v-if="mode === 'idle'" @click="emit('edit')" />
      </div>
    </template>

    <template v-if="mode !== 'editing'">
      <div class="flex flex-col gap-0.5 mt-4 pt-[14px] border-t border-neutral-100">
        <ContactLine icon="mail" :value="user.email" :placeholder="t('profile.identity.add_email')" />
        <ContactLine icon="phone" :value="user.telephone" :placeholder="t('profile.identity.add_phone')" />
        <ContactLine icon="linkedin" :value="user.linkedin" :placeholder="t('profile.identity.add_linkedin')" link @open="url => emit('open-url', url)" />
        <ContactLine icon="github" :value="user.github" :placeholder="t('profile.identity.add_github')" link @open="url => emit('open-url', url)" />
        <ContactLine icon="globe" :value="user.portfolio" :placeholder="t('profile.identity.add_portfolio')" link @open="url => emit('open-url', url)" />
        <ContactLine
          icon="car"
          :value="user.permis === true ? t('profile.driving_licence') : user.permis === false ? t('profile.identity.no_licence') : null"
          :placeholder="t('profile.identity.licence_unknown')"
        />
      </div>
      <div v-if="linkError" class="text-[12px] text-red-500 mt-[7px]">{{ linkError }}</div>
    </template>

    <!-- Édition -->
    <div v-else class="mt-[14px]">
      <div class="flex gap-[10px]">
        <div class="flex-1 min-w-0 mb-3">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.info.first_name') }}</label>
          <input v-model="draft.prenom" :class="inputClass" :placeholder="t('profile.placeholders.first_name')" />
        </div>
        <div class="flex-1 min-w-0 mb-3">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.info.last_name') }}</label>
          <input v-model="draft.nom" :class="inputClass" :placeholder="t('profile.placeholders.last_name')" />
        </div>
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.info.email') }}</label>
        <input v-model="draft.email" type="email" :class="inputClass" :placeholder="t('profile.placeholders.email')" />
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.info.phone') }}</label>
        <input v-model="draft.telephone" :class="inputClass" :placeholder="t('profile.placeholders.phone')" />
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">LinkedIn</label>
        <input v-model="draft.linkedin" type="url" :class="inputClass" placeholder="https://linkedin.com/in/..." />
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">GitHub</label>
        <input v-model="draft.github" type="url" :class="inputClass" placeholder="https://github.com/..." />
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Portfolio</label>
        <input v-model="draft.portfolio" type="url" :class="inputClass" :placeholder="t('profile.placeholders.portfolio')" />
      </div>
      <div class="mb-3">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.info.profile_language') }}</label>
        <select v-model="draft.langue_profil" :class="inputClass">
          <option value="fr">{{ t('profile.info.lang_fr') }}</option>
          <option value="en">{{ t('profile.info.lang_en') }}</option>
        </select>
      </div>
      <label class="inline-flex items-center gap-2 text-[13.5px] cursor-pointer select-none">
        <input v-model="draft.permis" type="checkbox" />
        {{ t('profile.driving_licence') }}
      </label>
    </div>
  </ProfileCard>
</template>
