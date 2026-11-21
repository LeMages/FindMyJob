<script setup lang="ts">
/**
 * Vue « Profil » : colonne fixe à gauche (identité, compétences, langues) et
 * contenu à droite (à propos, expériences, formation, projets), sur une seule
 * colonne quand la place manque. Chaque carte s'édite séparément, une seule à
 * la fois : la persistance est déléguée à chaque carte (`save()`), cette vue
 * orchestre l'édition et la sauvegarde de la carte ouverte.
 */

// Importation des modules Vue / framework
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AboutCard from './components/AboutCard.vue'
import ExperiencesSection from './components/ExperiencesSection.vue'
import FormationsSection from './components/FormationsSection.vue'
import IdentityCard from './components/IdentityCard.vue'
import LanguesSection from './components/LanguesSection.vue'
import ProjetsSection from './components/ProjetsSection.vue'
import SkillsSection from './components/SkillsSection.vue'

// Importation des composables
import { backendErrorMessage } from '@/composables/backend_error_message'
import { cardMode, type ProfileSection } from './components/profile_sections'

// Importation des modules internes
import { setUserInfo } from '@/stores/app'

// Importation des types
import type { ProfileDto, UserView } from '@/types/index'

const { t } = useI18n()

const profile = ref<ProfileDto | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const linkError = ref<string | null>(null)

/**
 * Ouvre une URL du profil (LinkedIn, GitHub, portfolio, lien de projet) dans
 * le navigateur par défaut, via la commande Tauri `open_url` — un simple
 * `<a target="_blank">` ne fonctionne pas dans la webview.
 *
 * @param url - URL à ouvrir.
 */
async function openExternalLink(url: string): Promise<void> {
  linkError.value = null
  try {
    await invoke('open_url', { url })
  } catch (e: unknown) {
    linkError.value = backendErrorMessage(e, t('error.fallback.open_link'))
  }
}

/**
 * Synchronise le nom d'affichage et les initiales de l'utilisateur dans le
 * store global (utilisés par la barre latérale).
 *
 * @param u - Utilisateur à jour.
 */
function syncUserInfo(u: UserView) {
  const name = [u.prenom, u.nom].filter(Boolean).join(' ')
  const initials = ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase()
  setUserInfo(name, initials || '?', u.nom, u.prenom)
}

// ======================================================================
// Édition section par section
// ======================================================================

const editing = ref<ProfileSection | null>(null)
const saving = ref(false)
const saveError = ref<string | null>(null)

const identityCard = ref<InstanceType<typeof IdentityCard> | null>(null)
const aboutCard = ref<InstanceType<typeof AboutCard> | null>(null)
const experiencesSection = ref<InstanceType<typeof ExperiencesSection> | null>(null)
const formationsSection = ref<InstanceType<typeof FormationsSection> | null>(null)
const projetsSection = ref<InstanceType<typeof ProjetsSection> | null>(null)
const skillsSection = ref<InstanceType<typeof SkillsSection> | null>(null)
const languesSection = ref<InstanceType<typeof LanguesSection> | null>(null)

/**
 * Carte d'une section (brouillon local, réinitialisation, sauvegarde).
 *
 * @param section - Section visée.
 * @returns Le composant monté, ou `null`.
 */
function cardOf(section: ProfileSection): { reset: () => void } | null {
  const cards = {
    info: identityCard,
    about: aboutCard,
    experiences: experiencesSection,
    formations: formationsSection,
    projets: projetsSection,
    skills: skillsSection,
    langues: languesSection,
  }
  return cards[section].value
}

/**
 * Passe une carte en édition (ignoré si une autre carte est déjà en édition).
 *
 * @param section - Section à éditer.
 */
function startEdit(section: ProfileSection) {
  if (editing.value !== null) return
  saveError.value = null
  editing.value = section
}

/** Abandonne les modifications de la carte en édition. */
function cancelEdit() {
  if (editing.value !== null) cardOf(editing.value)?.reset()
  saveError.value = null
  editing.value = null
}

/**
 * Enregistre la carte en édition et met à jour le profil affiché.
 *
 * @param p - Profil chargé.
 * @param section - Section à enregistrer.
 */
async function persistSection(p: ProfileDto, section: ProfileSection): Promise<void> {
  switch (section) {
    case 'info':
    case 'about': {
      const card = section === 'info' ? identityCard.value : aboutCard.value
      if (!card) return
      p.user = await card.save()
      syncUserInfo(p.user)
      return
    }
    case 'experiences':
      if (experiencesSection.value) p.experiences = await experiencesSection.value.save()
      return
    case 'formations':
      if (formationsSection.value) p.formations = await formationsSection.value.save()
      return
    case 'projets':
      if (projetsSection.value) p.projets = await projetsSection.value.save()
      return
    case 'skills':
      if (skillsSection.value) p.skills = await skillsSection.value.save()
      return
    case 'langues':
      if (languesSection.value) p.langues = await languesSection.value.save()
      return
  }
}

/** Valide la carte en édition : sauvegarde puis retour en lecture. */
async function saveEdit() {
  const section = editing.value
  if (section === null || !profile.value || saving.value) return
  saving.value = true
  saveError.value = null
  try {
    await persistSection(profile.value, section)
    editing.value = null
  } catch (e: unknown) {
    saveError.value = backendErrorMessage(e, t('error.fallback.save_profile'))
  } finally {
    saving.value = false
  }
}

/**
 * Propriétés communes passées à la carte d'une section.
 *
 * @param section - Section de la carte.
 * @returns Mode, état de sauvegarde et erreur de la carte.
 */
function cardProps(section: ProfileSection) {
  const isEditing = editing.value === section
  return {
    mode: cardMode(section, editing.value),
    saving: isEditing && saving.value,
    error: isEditing ? saveError.value : null,
  }
}

onMounted(async () => {
  try {
    const data = await invoke<ProfileDto>('get_profile')
    profile.value = data
    syncUserInfo(data.user)
  } catch (e: unknown) {
    loadError.value = backendErrorMessage(e, t('error.fallback.load_profile'))
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <!-- Loading -->
  <div v-if="loading" class="px-5 sm:px-9 pt-[26px] pb-11">
    <div class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">{{ t('common.loading') }}</div>
  </div>

  <!-- Load error -->
  <div v-else-if="loadError" class="px-5 sm:px-9 pt-[26px] pb-11">
    <div class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>
  </div>

  <!-- Profile -->
  <div v-else-if="profile" class="px-5 sm:px-9 pt-[26px] pb-11 @container">
    <h1 class="text-2xl font-semibold leading-tight">{{ t('profile.title') }}</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] font-mono mb-[18px]">{{ t('profile.subtitle') }}</div>

    <div class="grid grid-cols-1 @[760px]:grid-cols-[300px_minmax(0,1fr)] gap-x-6 items-start">
      <!-- Colonne fixe : identité, compétences, langues -->
      <aside class="min-w-0" :class="editing === null && '@[760px]:sticky @[760px]:top-5'">
        <IdentityCard
          ref="identityCard"
          :user="profile.user"
          v-bind="cardProps('info')"
          :link-error="linkError"
          @edit="startEdit('info')"
          @cancel="cancelEdit"
          @save="saveEdit"
          @open-url="openExternalLink"
        />
        <SkillsSection ref="skillsSection" :items="profile.skills" v-bind="cardProps('skills')" @edit="startEdit('skills')" @cancel="cancelEdit" @save="saveEdit" />
        <LanguesSection ref="languesSection" :items="profile.langues" v-bind="cardProps('langues')" @edit="startEdit('langues')" @cancel="cancelEdit" @save="saveEdit" />
      </aside>

      <!-- Contenu : à propos, expériences, formation, projets -->
      <div class="min-w-0">
        <AboutCard ref="aboutCard" :user="profile.user" v-bind="cardProps('about')" @edit="startEdit('about')" @cancel="cancelEdit" @save="saveEdit" />
        <ExperiencesSection ref="experiencesSection" :items="profile.experiences" v-bind="cardProps('experiences')" @edit="startEdit('experiences')" @cancel="cancelEdit" @save="saveEdit" />
        <FormationsSection ref="formationsSection" :items="profile.formations" v-bind="cardProps('formations')" @edit="startEdit('formations')" @cancel="cancelEdit" @save="saveEdit" />
        <ProjetsSection
          ref="projetsSection"
          :items="profile.projets"
          v-bind="cardProps('projets')"
          @edit="startEdit('projets')"
          @cancel="cancelEdit"
          @save="saveEdit"
          @open-url="openExternalLink"
        />
      </div>
    </div>
  </div>
</template>
