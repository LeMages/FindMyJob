<script setup lang="ts">
/**
 * Vue « Profil » : affiche et permet d'éditer les informations de
 * l'utilisateur (identité, description, expériences, formations, projets,
 * compétences, langues, CV et lettre de motivation). La persistance de
 * chaque section est déléguée aux composants sous `components/` ; cette vue
 * ne fait que les composer et orchestrer la sauvegarde globale.
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
import DocumentReferenceEditor from '@/components/DocumentReferenceEditor.vue'
import type {
  ProfileDto,
  UserView,
  UpdateUserInfoInput,
  ExperienceView,
  FormationView,
  ProjetView,
  SkillView,
  LangView,
  DocumentReferenceView,
} from '@/types/index'

const profile = ref<ProfileDto | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)

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
}

const draftUserInfo = ref<UserInfoDraft>({ prenom: '', nom: '', email: '', telephone: '', permis: false })

function isUserInfoDirty(): boolean {
  const u = profile.value?.user
  if (!u) return false
  return (
    draftUserInfo.value.prenom !== (u.prenom ?? '') ||
    draftUserInfo.value.nom !== (u.nom ?? '') ||
    draftUserInfo.value.email !== (u.email ?? '') ||
    draftUserInfo.value.telephone !== (u.telephone ?? '') ||
    draftUserInfo.value.permis !== (u.permis ?? false)
  )
}

// Références vers les composants de section, utilisées pour orchestrer la
// sauvegarde et la réinitialisation globales (voir saveAll / cancelEdit).
const experiencesSection = ref<InstanceType<typeof ExperiencesSection> | null>(null)
const formationsSection = ref<InstanceType<typeof FormationsSection> | null>(null)
const projetsSection = ref<InstanceType<typeof ProjetsSection> | null>(null)
const skillsSection = ref<InstanceType<typeof SkillsSection> | null>(null)
const languesSection = ref<InstanceType<typeof LanguesSection> | null>(null)
const cvEditor = ref<InstanceType<typeof DocumentReferenceEditor> | null>(null)
const lettreEditor = ref<InstanceType<typeof DocumentReferenceEditor> | null>(null)

const isDirty = computed(() =>
  isUserInfoDirty() ||
  draftDescription.value !== (profile.value?.user.description ?? '') ||
  (experiencesSection.value?.hasChanges ?? false) ||
  (formationsSection.value?.hasChanges ?? false) ||
  (projetsSection.value?.hasChanges ?? false) ||
  (skillsSection.value?.hasChanges ?? false) ||
  (languesSection.value?.hasChanges ?? false) ||
  (cvEditor.value?.hasChanges ?? false) ||
  (lettreEditor.value?.hasChanges ?? false),
)

function enterEditMode() {
  const u = profile.value?.user
  draftUserInfo.value = {
    prenom: u?.prenom ?? '',
    nom: u?.nom ?? '',
    email: u?.email ?? '',
    telephone: u?.telephone ?? '',
    permis: u?.permis ?? false,
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
  cvEditor.value?.reset()
  lettreEditor.value?.reset()
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

/** Sauvegarde les informations de contact du profil, si elles ont changé. */
async function saveUserInfo(): Promise<void> {
  if (!isUserInfoDirty()) return
  const d = draftUserInfo.value
  const input: UpdateUserInfoInput = {
    prenom: d.prenom.trim() || null,
    nom: d.nom.trim() || null,
    email: d.email.trim() || null,
    telephone: d.telephone.trim() || null,
    permis: d.permis,
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
 * formations, projets, compétences, langues). Les documents (CV, lettre) sont
 * déjà persistés au moment de leur sélection — seule leur marque « modifié »
 * est réinitialisée ici.
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

    // 3. Documents — déjà persistés à la sélection, on efface juste la marque « modifié »
    cvEditor.value?.reset()
    lettreEditor.value?.reset()
    editMode.value = false
  } catch (e: unknown) {
    saveError.value = extractErrorMessage(e, 'Erreur lors de la sauvegarde.')
  } finally {
    saving.value = false
  }
}

function onCvReferenceSaved(reference: DocumentReferenceView) {
  if (profile.value) profile.value.cv_reference = reference
}

function onLettreReferenceSaved(reference: DocumentReferenceView) {
  if (profile.value) profile.value.lettre_motivation_reference = reference
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
  <div v-if="loading" class="page">
    <div class="cempty" style="margin-top:60px">Chargement…</div>
  </div>

  <!-- Load error -->
  <div v-else-if="loadError" class="page">
    <div class="cempty" style="margin-top:60px;color:#b94a48">{{ loadError }}</div>
  </div>

  <!-- Profile -->
  <div v-else-if="profile" class="page" :style="editMode ? 'padding-bottom:90px' : ''">
    <h1 class="h1">Mon profil</h1>
    <div class="sub" style="margin-bottom:18px">Ces informations pré-remplissent tes candidatures.</div>

    <!-- Header -->
    <div class="phead">
      <div class="avatar pavatar" style="width:84px;height:84px;font-size:26px;border-radius:50%">
        {{ initials }}
      </div>
      <div class="f1">
        <div class="pname">
          {{ [profile.user.prenom, profile.user.nom].filter(Boolean).join(' ') || '—' }}
        </div>
        <div class="pcontact">
          <span v-if="profile.user.email">{{ profile.user.email }}</span>
          <span v-if="profile.user.telephone">{{ profile.user.telephone }}</span>
          <span v-if="profile.user.ville">{{ profile.user.ville }}</span>
          <span v-if="profile.user.github">{{ profile.user.github }}</span>
          <span v-if="profile.user.linkedin">{{ profile.user.linkedin }}</span>
          <span v-if="profile.user.permis === true">Permis B</span>
        </div>
      </div>
      <button v-if="!editMode" class="btn" @click="enterEditMode">Modifier</button>
    </div>

    <!-- Section infos de base (visible uniquement en mode édition) -->
    <div v-if="editMode" class="fieldset" style="margin-top:18px">
      <div class="fstitle">INFORMATIONS</div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Prénom</label>
          <input v-model="draftUserInfo.prenom" class="input" placeholder="Prénom" />
        </div>
        <div class="field">
          <label class="flabel">Nom</label>
          <input v-model="draftUserInfo.nom" class="input" placeholder="Nom" />
        </div>
      </div>
      <div class="frow">
        <div class="field">
          <label class="flabel">Adresse e-mail</label>
          <input v-model="draftUserInfo.email" class="input" type="email" placeholder="prenom.nom@email.com" />
        </div>
        <div class="field">
          <label class="flabel">Téléphone</label>
          <input v-model="draftUserInfo.telephone" class="input" placeholder="+33 6 00 00 00 00" />
        </div>
      </div>
      <label class="permis-label">
        <input type="checkbox" v-model="draftUserInfo.permis" />
        Permis B
      </label>
    </div>

    <div class="dgrid" style="grid-template-columns:1fr 300px; margin-top:18px">
      <!-- ── Colonne gauche ── -->
      <div>
        <!-- À propos -->
        <div class="fieldset">
          <div class="fstitle">À PROPOS</div>
          <template v-if="!editMode">
            <div v-if="profile.user.description" class="expd">{{ profile.user.description }}</div>
            <div v-else class="cempty">Non renseigné</div>
          </template>
          <textarea
            v-else
            v-model="draftDescription"
            class="input"
            style="height:140px"
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

        <DocumentReferenceEditor
          ref="cvEditor"
          doc-type="cv"
          :reference="profile.cv_reference"
          :edit-mode="editMode"
          @update:reference="onCvReferenceSaved"
        />
        <DocumentReferenceEditor
          ref="lettreEditor"
          doc-type="lettre_motivation"
          :reference="profile.lettre_motivation_reference"
          :edit-mode="editMode"
          @update:reference="onLettreReferenceSaved"
        />
      </div>
    </div>

    <!-- Edit footer -->
    <div v-if="editMode" class="edit-footer">
      <span v-if="saveError" class="edit-error">{{ saveError }}</span>
      <div class="f1" />
      <button class="btn edit-cancel" type="button" @click="cancelEdit">Annuler</button>
      <button
        class="btn edit-save"
        type="button"
        :class="{ disabled: !isDirty || saving }"
        :disabled="!isDirty || saving"
        @click="saveAll"
      >{{ saving ? 'Sauvegarde…' : 'Valider' }}</button>
    </div>
  </div>
</template>

<style scoped>
.edit-footer {
  position: fixed;
  bottom: 0;
  left: 264px;
  right: 0;
  height: 60px;
  background: #fff;
  border-top: 1px solid #e6e6e1;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 36px;
  z-index: 10;
}

.edit-error {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 12px;
  color: #b94a48;
}

.edit-cancel {
  background: #e03e3e;
  color: #fff;
  border-color: #e03e3e;
}
.edit-cancel:hover { background: #c93333; border-color: #c93333; }

.permis-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  cursor: pointer;
  user-select: none;
  margin-top: 2px;
}
</style>
