<script setup lang="ts">
/**
 * Vue « Profil » : affiche et permet d'éditer les informations de
 * l'utilisateur (identité, description, expériences, formations, projets,
 * compétences, langues). La persistance de chaque section est déléguée aux
 * composants sous `components/` ; cette vue ne fait que les composer et
 * orchestrer la sauvegarde globale.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { setUserInfo } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import ExperiencesSection from './components/ExperiencesSection.vue'
import FormationsSection from './components/FormationsSection.vue'
import ProjetsSection from './components/ProjetsSection.vue'
import SkillsSection from './components/SkillsSection.vue'
import LanguesSection from './components/LanguesSection.vue'
import type {
  ProfileDto,
  UserView,
  UpdateUserInfoInput,
  ExperienceView,
  FormationView,
  ProjetView,
  SkillView,
  LangView,
} from '@/types/index'

const profile = ref<ProfileDto | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const linkError = ref<string | null>(null)

/**
 * Ouvre une URL du profil (LinkedIn, GitHub, portfolio) dans le navigateur
 * par défaut de l'utilisateur, via la commande Tauri `open_url` — un simple
 * `<a target="_blank">` ne fonctionne pas dans la webview.
 *
 * @param url - URL à ouvrir.
 */
async function openExternalLink(url: string): Promise<void> {
  linkError.value = null
  try {
    await invoke('open_url', { url })
  } catch (e: unknown) {
    linkError.value = extractErrorMessage(e, "Impossible d'ouvrir ce lien.")
  }
}

// ── Mode édition ────────────────────────────────────────────────────────────

const editMode = ref(false)
const saving = ref(false)
const saveError = ref<string | null>(null)
const draftDescription = ref('')

interface UserInfoDraft {
  prenom: string
  nom: string
  email: string
  telephone: string
  permis: boolean
  linkedin: string
  github: string
  portfolio: string
}

const draftUserInfo = ref<UserInfoDraft>({
  prenom: '',
  nom: '',
  email: '',
  telephone: '',
  permis: false,
  linkedin: '',
  github: '',
  portfolio: '',
})

function isUserInfoDirty(): boolean {
  const u = profile.value?.user
  if (!u) return false
  return (
    draftUserInfo.value.prenom !== (u.prenom ?? '') ||
    draftUserInfo.value.nom !== (u.nom ?? '') ||
    draftUserInfo.value.email !== (u.email ?? '') ||
    draftUserInfo.value.telephone !== (u.telephone ?? '') ||
    draftUserInfo.value.permis !== (u.permis ?? false) ||
    draftUserInfo.value.linkedin !== (u.linkedin ?? '') ||
    draftUserInfo.value.github !== (u.github ?? '') ||
    draftUserInfo.value.portfolio !== (u.portfolio ?? '')
  )
}

// Références vers les composants de section, utilisées pour orchestrer la
// sauvegarde et la réinitialisation globales (voir saveAll / cancelEdit).
const experiencesSection = ref<InstanceType<typeof ExperiencesSection> | null>(null)
const formationsSection = ref<InstanceType<typeof FormationsSection> | null>(null)
const projetsSection = ref<InstanceType<typeof ProjetsSection> | null>(null)
const skillsSection = ref<InstanceType<typeof SkillsSection> | null>(null)
const languesSection = ref<InstanceType<typeof LanguesSection> | null>(null)

const isDirty = computed(() =>
  isUserInfoDirty() ||
  draftDescription.value !== (profile.value?.user.description ?? '') ||
  (experiencesSection.value?.hasChanges ?? false) ||
  (formationsSection.value?.hasChanges ?? false) ||
  (projetsSection.value?.hasChanges ?? false) ||
  (skillsSection.value?.hasChanges ?? false) ||
  (languesSection.value?.hasChanges ?? false),
)

function enterEditMode() {
  const u = profile.value?.user
  draftUserInfo.value = {
    prenom: u?.prenom ?? '',
    nom: u?.nom ?? '',
    email: u?.email ?? '',
    telephone: u?.telephone ?? '',
    permis: u?.permis ?? false,
    linkedin: u?.linkedin ?? '',
    github: u?.github ?? '',
    portfolio: u?.portfolio ?? '',
  }
  draftDescription.value = u?.description ?? ''
  saveError.value = null
  editMode.value = true
}

function cancelEdit() {
  experiencesSection.value?.reset()
  formationsSection.value?.reset()
  projetsSection.value?.reset()
  skillsSection.value?.reset()
  languesSection.value?.reset()
  saveError.value = null
  editMode.value = false
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
  setUserInfo(name, initials || '?')
}

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

/** Sauvegarde les informations de contact du profil, si elles ont changé. */
async function saveUserInfo(): Promise<void> {
  if (!isUserInfoDirty()) return
  const d = draftUserInfo.value
  const linkedin = d.linkedin.trim()
  const github = d.github.trim()
  const portfolio = d.portfolio.trim()

  if (!isValidUrlOrEmpty(linkedin)) throw new Error('Le lien LinkedIn n\'est pas une URL valide.')
  if (!isValidUrlOrEmpty(github)) throw new Error('Le lien GitHub n\'est pas une URL valide.')
  if (!isValidUrlOrEmpty(portfolio)) throw new Error('Le lien portfolio n\'est pas une URL valide.')

  const input: UpdateUserInfoInput = {
    prenom: d.prenom.trim() || null,
    nom: d.nom.trim() || null,
    email: d.email.trim() || null,
    telephone: d.telephone.trim() || null,
    permis: d.permis,
    linkedin: linkedin || null,
    github: github || null,
    portfolio: portfolio || null,
  }
  const updated = await invoke<UserView>('update_user_info', { input })
  profile.value!.user = updated
  syncUserInfo(updated)
}

/** Sauvegarde la description libre du profil, si elle a changé. */
async function saveDescription(): Promise<void> {
  if (draftDescription.value === (profile.value?.user.description ?? '')) return
  const updated = await invoke<UserView>('update_description', {
    description: draftDescription.value.trim() || null,
  })
  profile.value!.user = updated
  syncUserInfo(updated)
}

/**
 * Orchestre la sauvegarde de toutes les sections du profil : informations
 * utilisateur, description, puis chaque liste éditable (expériences,
 * formations, projets, compétences, langues).
 */
async function saveAll() {
  if (!isDirty.value || saving.value) return
  saving.value = true
  saveError.value = null
  try {
    // 1. Informations utilisateur (identité + description)
    await saveUserInfo()
    await saveDescription()

    // 2. Listes éditables — chaque section gère ses propres ajouts/modifications/suppressions
    if (profile.value) {
      const experiences = await experiencesSection.value?.save()
      if (experiences) profile.value.experiences = experiences as ExperienceView[]

      const formations = await formationsSection.value?.save()
      if (formations) profile.value.formations = formations as FormationView[]

      const projets = await projetsSection.value?.save()
      if (projets) profile.value.projets = projets as ProjetView[]

      const skills = await skillsSection.value?.save()
      if (skills) profile.value.skills = skills as SkillView[]

      const langues = await languesSection.value?.save()
      if (langues) profile.value.langues = langues as LangView[]
    }

    editMode.value = false
  } catch (e: unknown) {
    saveError.value = extractErrorMessage(e, 'Erreur lors de la sauvegarde.')
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    const data = await invoke<ProfileDto>('get_profile')
    profile.value = data
    syncUserInfo(data.user)
  } catch (e: unknown) {
    loadError.value = extractErrorMessage(e, 'Impossible de charger le profil.')
  } finally {
    loading.value = false
  }
})

const initials = computed(() => {
  const u = profile.value?.user
  if (!u) return '?'
  return ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase() || '?'
})
</script>

<template>
  <!-- Loading -->
  <div v-if="loading" class="px-9 pt-[26px] pb-11">
    <div class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">Chargement…</div>
  </div>

  <!-- Load error -->
  <div v-else-if="loadError" class="px-9 pt-[26px] pb-11">
    <div class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>
  </div>

  <!-- Profile -->
  <div v-else-if="profile" class="px-9 pt-[26px]" :class="editMode ? 'pb-[90px]' : 'pb-11'">
    <h1 class="text-2xl font-semibold leading-tight">Mon profil</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] font-mono mb-[18px]">Ces informations pré-remplissent tes candidatures.</div>

    <!-- Header -->
    <div class="flex items-center gap-5 bg-surface-1 border border-neutral-200 rounded-xl p-6">
      <div class="w-[84px] h-[84px] rounded-full flex-none flex items-center justify-center font-semibold text-[26px] bg-neutral-200 text-neutral-700">
        {{ initials }}
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[22px] font-semibold">
          {{ [profile.user.prenom, profile.user.nom].filter(Boolean).join(' ') || '—' }}
        </div>
        <div class="flex gap-[18px] mt-[11px] font-mono text-[12px] text-neutral-600 flex-wrap">
          <span v-if="profile.user.email">{{ profile.user.email }}</span>
          <span v-if="profile.user.telephone">{{ profile.user.telephone }}</span>
          <span v-if="profile.user.ville">{{ profile.user.ville }}</span>
          <a
            v-if="profile.user.linkedin"
            href="#"
            :title="profile.user.linkedin"
            class="inline-flex items-center gap-1.5 hover:text-neutral-900 cursor-pointer"
            @click.prevent="openExternalLink(profile.user.linkedin!)"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z" />
              <rect width="4" height="12" x="2" y="9" />
              <circle cx="4" cy="4" r="2" />
            </svg>
            LinkedIn
          </a>
          <a
            v-if="profile.user.github"
            href="#"
            :title="profile.user.github"
            class="inline-flex items-center gap-1.5 hover:text-neutral-900 cursor-pointer"
            @click.prevent="openExternalLink(profile.user.github!)"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
              <path d="M9 18c-4.51 2-5-2-7-2" />
            </svg>
            GitHub
          </a>
          <a
            v-if="profile.user.portfolio"
            href="#"
            :title="profile.user.portfolio"
            class="inline-flex items-center gap-1.5 hover:text-neutral-900 cursor-pointer"
            @click.prevent="openExternalLink(profile.user.portfolio!)"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
              <path d="M2 12h20" />
            </svg>
            Portfolio
          </a>
          <span v-if="profile.user.permis === true">Permis B</span>
        </div>
        <div v-if="linkError" class="text-[12px] text-red-500 mt-[7px]">{{ linkError }}</div>
      </div>
      <button
        v-if="!editMode"
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="enterEditMode"
      >Modifier</button>
    </div>

    <!-- Section infos de base (visible uniquement en mode édition) -->
    <div v-if="editMode" class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px] mt-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">INFORMATIONS</div>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Prénom</label>
          <input v-model="draftUserInfo.prenom" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Prénom" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Nom</label>
          <input v-model="draftUserInfo.nom" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Nom" />
        </div>
      </div>
      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Adresse e-mail</label>
          <input v-model="draftUserInfo.email" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" type="email" placeholder="prenom.nom@email.com" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Téléphone</label>
          <input v-model="draftUserInfo.telephone" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="+33 6 00 00 00 00" />
        </div>
      </div>
      <label class="inline-flex items-center gap-2 text-[13.5px] cursor-pointer select-none mt-0.5">
        <input type="checkbox" v-model="draftUserInfo.permis" />
        Permis B
      </label>
    </div>

    <!-- Section liens (visible uniquement en mode édition) -->
    <div v-if="editMode" class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">LIENS</div>
      <div class="flex gap-[14px]">
        <div class="flex-1">
          <label class="text-[12px] text-neutral-600 mb-[5px] flex items-center gap-1.5">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z" />
              <rect width="4" height="12" x="2" y="9" />
              <circle cx="4" cy="4" r="2" />
            </svg>
            LinkedIn
          </label>
          <input v-model="draftUserInfo.linkedin" type="url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://linkedin.com/in/..." />
        </div>
        <div class="flex-1">
          <label class="text-[12px] text-neutral-600 mb-[5px] flex items-center gap-1.5">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
              <path d="M9 18c-4.51 2-5-2-7-2" />
            </svg>
            GitHub
          </label>
          <input v-model="draftUserInfo.github" type="url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://github.com/..." />
        </div>
        <div class="flex-1">
          <label class="text-[12px] text-neutral-600 mb-[5px] flex items-center gap-1.5">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
              <path d="M2 12h20" />
            </svg>
            Portfolio
          </label>
          <input v-model="draftUserInfo.portfolio" type="url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://monsite.com" />
        </div>
      </div>
    </div>

    <div class="grid grid-cols-[1fr_300px] gap-6 mt-[18px] items-start">
      <!-- ── Colonne gauche ── -->
      <div>
        <!-- À propos -->
        <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">À PROPOS</div>
          <template v-if="!editMode">
            <div v-if="profile.user.description" class="text-[13px] text-neutral-700 mt-[7px] leading-[1.55]">{{ profile.user.description }}</div>
            <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Non renseigné</div>
          </template>
          <textarea
            v-else
            v-model="draftDescription"
            class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[140px]"
            placeholder="Décris ton parcours, tes objectifs…"
          />
        </div>

        <ExperiencesSection ref="experiencesSection" :items="profile.experiences" :edit-mode="editMode" />
        <FormationsSection ref="formationsSection" :items="profile.formations" :edit-mode="editMode" />
        <ProjetsSection ref="projetsSection" :items="profile.projets" :edit-mode="editMode" />
      </div>

      <!-- ── Colonne droite ── -->
      <div>
        <SkillsSection ref="skillsSection" :items="profile.skills" :edit-mode="editMode" />
        <LanguesSection ref="languesSection" :items="profile.langues" :edit-mode="editMode" />
      </div>
    </div>

    <!-- Edit footer -->
    <div v-if="editMode" class="fixed bottom-0 left-[264px] right-0 h-[60px] bg-surface-1 border-t border-neutral-200 flex items-center gap-[10px] px-9 z-10">
      <span v-if="saveError" class="font-mono text-[12px] text-red-500">{{ saveError }}</span>
      <div class="flex-1 min-w-0" />
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-red-400 text-white border-red-400 hover:bg-red-600 hover:border-red-600"
        type="button"
        @click="cancelEdit"
      >Annuler</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !isDirty || saving }"
        type="button"
        :disabled="!isDirty || saving"
        @click="saveAll"
      >{{ saving ? 'Sauvegarde…' : 'Valider' }}</button>
    </div>
  </div>
</template>
