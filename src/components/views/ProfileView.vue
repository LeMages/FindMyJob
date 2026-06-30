<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ProfileDto, UserView, UpdateUserInfoInput, ExperienceView, CreateExperienceInput, FormationView, CreateFormationInput, ProjetView, CreateProjetInput, LangView, CreateLangInput, SkillView, CreateSkillInput, CvView } from '@/types/index'
import { setUserInfo } from '@/stores/app'
import { parseBulletDescription } from '@/composables/parseBulletDescription'
import { insertBullet } from '@/composables/insertBullet'

const profile = ref<ProfileDto | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)

// ── Edit mode ─────────────────────────────────────────────────────────────────

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

const isDirty = computed(() =>
  isUserInfoDirty() ||
  cvJustSaved.value ||
  draftDescription.value !== (profile.value?.user.description ?? '') ||
  pendingExperiences.value.length > 0 ||
  deletedExpIds.value.length > 0 ||
  Object.keys(editedExps.value).length > 0 ||
  pendingFormations.value.length > 0 ||
  deletedFormIds.value.length > 0 ||
  Object.keys(editedForms.value).length > 0 ||
  pendingProjets.value.length > 0 ||
  deletedProjetIds.value.length > 0 ||
  Object.keys(editedProjets.value).length > 0 ||
  pendingSkills.value.length > 0 ||
  deletedSkillIds.value.length > 0 ||
  Object.keys(editedSkills.value).length > 0 ||
  pendingLangs.value.length > 0 ||
  deletedLangIds.value.length > 0 ||
  Object.keys(editedLangs.value).length > 0,
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
  initCvFromProfile()
  cvError.value = null
  saveError.value = null
  editMode.value = true
}

function cancelEdit() {
  pendingExperiences.value = []
  deletedExpIds.value = []
  editedExps.value = {}
  pendingFormations.value = []
  deletedFormIds.value = []
  editedForms.value = {}
  pendingProjets.value = []
  deletedProjetIds.value = []
  editedProjets.value = {}
  pendingSkills.value = []
  deletedSkillIds.value = []
  editedSkills.value = {}
  pendingLangs.value = []
  deletedLangIds.value = []
  editedLangs.value = {}
  cvJustSaved.value = false
  cvError.value = null
  editMode.value = false
  saveError.value = null
}

async function saveDescription() {
  if (!isDirty.value || saving.value) return
  saving.value = true
  saveError.value = null
  try {
    const pad = (n: number) => String(n).padStart(2, '0')
    const draftToInput = (d: ExpDraft): CreateExperienceInput => ({
      poste: d.poste.trim(),
      entreprise: d.entreprise.trim() || null,
      lieu: d.lieu.trim() || null,
      description: d.description.trim() || null,
      date_debut: `${d.debut_annee}-${pad(d.debut_mois)}-01`,
      date_fin: d.en_cours ? null : `${d.fin_annee}-${pad(d.fin_mois)}-01`,
      type_contrat: d.type_contrat || null,
    })

    // Infos utilisateur de base
    if (isUserInfoDirty()) {
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

    // Description
    if (draftDescription.value !== (profile.value?.user.description ?? '')) {
      const updated = await invoke<UserView>('update_description', {
        description: draftDescription.value.trim() || null,
      })
      profile.value!.user = updated
      syncUserInfo(updated)
    }

    // Suppressions
    for (const id of deletedExpIds.value) {
      await invoke('delete_experience', { id })
    }
    profile.value!.experiences = profile.value!.experiences.filter(
      (e: ExperienceView) => !deletedExpIds.value.includes(e.id),
    )
    deletedExpIds.value = []

    // Modifications
    for (const [rawId, draft] of Object.entries(editedExps.value) as [string, ExpDraft][]) {
      const id = Number(rawId)
      const updated = await invoke<ExperienceView>('update_experience', {
        id,
        input: draftToInput(draft),
      })
      const idx = profile.value!.experiences.findIndex((e: ExperienceView) => e.id === id)
      if (idx >= 0) profile.value!.experiences[idx] = updated
    }
    editedExps.value = {}

    // Nouveaux ajouts d'expériences (du plus ancien au plus récent)
    for (const d of [...pendingExperiences.value].reverse()) {
      const created = await invoke<ExperienceView>('add_experience', {
        input: draftToInput(d),
      })
      profile.value!.experiences.unshift(created)
    }
    pendingExperiences.value = []

    // ── Compétences ─────────────────────────────────────────────────────────
    const skillToInput = (d: SkillDraft): CreateSkillInput => ({
      nom: d.nom.trim(),
      domaine: d.domaine.trim(),
    })

    for (const id of deletedSkillIds.value) {
      await invoke('delete_skill', { id })
    }
    profile.value!.skills = profile.value!.skills.filter(
      (s: SkillView) => !deletedSkillIds.value.includes(s.id),
    )
    deletedSkillIds.value = []

    for (const [rawId, draft] of Object.entries(editedSkills.value) as [string, SkillDraft][]) {
      const id = Number(rawId)
      const updated = await invoke<SkillView>('update_skill', { id, input: skillToInput(draft) })
      const idx = profile.value!.skills.findIndex((s: SkillView) => s.id === id)
      if (idx >= 0) profile.value!.skills[idx] = updated
    }
    editedSkills.value = {}

    for (const d of pendingSkills.value) {
      const created = await invoke<SkillView>('add_skill', { input: skillToInput(d) })
      profile.value!.skills.push(created)
    }
    pendingSkills.value = []

    // ── Formations ──────────────────────────────────────────────────────────
    const formToInput = (d: FormDraft): CreateFormationInput => ({
      titre: d.titre.trim(),
      ecole: d.ecole.trim() || null,
      date_debut: `${d.debut_annee}-01-01`,
      date_fin: d.en_cours ? null : `${d.fin_annee}-01-01`,
    })

    for (const id of deletedFormIds.value) {
      await invoke('delete_formation', { id })
    }
    profile.value!.formations = profile.value!.formations.filter(
      (f: FormationView) => !deletedFormIds.value.includes(f.id),
    )
    deletedFormIds.value = []

    for (const [rawId, draft] of Object.entries(editedForms.value) as [string, FormDraft][]) {
      const id = Number(rawId)
      const updated = await invoke<FormationView>('update_formation', {
        id,
        input: formToInput(draft),
      })
      const idx = profile.value!.formations.findIndex((f: FormationView) => f.id === id)
      if (idx >= 0) profile.value!.formations[idx] = updated
    }
    editedForms.value = {}

    for (const d of [...pendingFormations.value].reverse()) {
      const created = await invoke<FormationView>('add_formation', {
        input: formToInput(d),
      })
      profile.value!.formations.unshift(created)
    }
    pendingFormations.value = []

    // ── Projets ──────────────────────────────────────────────────────────────
    const projetToInput = (d: ProjetDraft): CreateProjetInput => ({
      titre: d.titre.trim(),
      description: d.description.trim() || null,
      stack: d.stack.trim() || null,
      url: d.url.trim() || null,
      pour_qui: d.pour_qui.trim() || null,
      date_debut: `${d.debut_annee}-01-01`,
      date_fin: d.en_cours ? null : `${d.fin_annee}-01-01`,
    })

    for (const id of deletedProjetIds.value) {
      await invoke('delete_projet', { id })
    }
    profile.value!.projets = profile.value!.projets.filter(
      (p: ProjetView) => !deletedProjetIds.value.includes(p.id),
    )
    deletedProjetIds.value = []

    for (const [rawId, draft] of Object.entries(editedProjets.value) as [string, ProjetDraft][]) {
      const id = Number(rawId)
      const updated = await invoke<ProjetView>('update_projet', {
        id,
        input: projetToInput(draft),
      })
      const idx = profile.value!.projets.findIndex((p: ProjetView) => p.id === id)
      if (idx >= 0) profile.value!.projets[idx] = updated
    }
    editedProjets.value = {}

    for (const d of [...pendingProjets.value].reverse()) {
      const created = await invoke<ProjetView>('add_projet', {
        input: projetToInput(d),
      })
      profile.value!.projets.unshift(created)
    }
    pendingProjets.value = []

    // ── Langues ─────────────────────────────────────────────────────────────
    const langToInput = (d: LangDraft): CreateLangInput => ({
      nom: d.nom.trim(),
      level: d.level.trim() || null,
    })

    for (const id of deletedLangIds.value) {
      await invoke('delete_lang', { id })
    }
    profile.value!.langues = profile.value!.langues.filter(
      (l: LangView) => !deletedLangIds.value.includes(l.id),
    )
    deletedLangIds.value = []

    for (const [rawId, draft] of Object.entries(editedLangs.value) as [string, LangDraft][]) {
      const id = Number(rawId)
      const updated = await invoke<LangView>('update_lang', { id, input: langToInput(draft) })
      const idx = profile.value!.langues.findIndex((l: LangView) => l.id === id)
      if (idx >= 0) profile.value!.langues[idx] = updated
    }
    editedLangs.value = {}

    for (const d of pendingLangs.value) {
      const created = await invoke<LangView>('add_lang', { input: langToInput(d) })
      profile.value!.langues.push(created)
    }
    pendingLangs.value = []

    cvJustSaved.value = false
    editMode.value = false
  } catch (e: unknown) {
    saveError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors de la sauvegarde.'
  } finally {
    saving.value = false
  }
}

// ── Expériences ───────────────────────────────────────────────────────────────

interface ExpDraft {
  poste: string
  entreprise: string
  lieu: string
  type_contrat: string
  debut_mois: number
  debut_annee: number
  fin_mois: number
  fin_annee: number
  en_cours: boolean
  description: string
}

type ExpEditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

interface ExpDisplayItem extends ExperienceView {
  isPending?: true
  pendingKind?: 'new' | 'edited'
}

// Nouvelles expériences ajoutées localement, pas encore en base
const pendingExperiences = ref<ExpDraft[]>([])
// IDs d'expériences sauvegardées marquées pour suppression
const deletedExpIds = ref<number[]>([])
// Modifications en attente sur des expériences sauvegardées : id → draft
const editedExps = ref<Record<number, ExpDraft>>({})

const expModal = ref(false)
const expDraft = ref<ExpDraft>(defaultExpDraft())
const expDescTextarea = ref<HTMLTextAreaElement | null>(null)
const expEditTarget = ref<ExpEditTarget>({ kind: 'new' })

// Vue unifiée : nouvelles (en tête) + sauvegardées (filtrées + modifiées)
const allExperiences = computed<ExpDisplayItem[]>(() => {
  const pad = (n: number) => String(n).padStart(2, '0')

  const pending: ExpDisplayItem[] = pendingExperiences.value.map((d: ExpDraft, i: number) => ({
    id: -(i + 1),
    poste: d.poste,
    entreprise: d.entreprise || null,
    lieu: d.lieu || null,
    description: d.description || null,
    date_debut: `${d.debut_annee}-${pad(d.debut_mois)}-01`,
    date_fin: d.en_cours ? null : `${d.fin_annee}-${pad(d.fin_mois)}-01`,
    type_contrat: d.type_contrat || null,
    isPending: true,
    pendingKind: 'new',
  }))

  const saved: ExpDisplayItem[] = (profile.value?.experiences ?? [])
    .filter((e: ExperienceView) => !deletedExpIds.value.includes(e.id))
    .map((e: ExperienceView) => {
      const edit = editedExps.value[e.id]
      if (edit) {
        return {
          id: e.id,
          poste: edit.poste,
          entreprise: edit.entreprise || null,
          lieu: edit.lieu || null,
          description: edit.description || null,
          date_debut: `${edit.debut_annee}-${pad(edit.debut_mois)}-01`,
          date_fin: edit.en_cours ? null : `${edit.fin_annee}-${pad(edit.fin_mois)}-01`,
          type_contrat: edit.type_contrat || null,
          isPending: true as const,
          pendingKind: 'edited' as const,
        }
      }
      return { ...e }
    })

  return [...pending, ...saved]
})

function defaultExpDraft(): ExpDraft {
  const now = new Date()
  return {
    poste: '',
    entreprise: '',
    lieu: '',
    type_contrat: '',
    debut_mois: now.getMonth() + 1,
    debut_annee: now.getFullYear(),
    fin_mois: now.getMonth() + 1,
    fin_annee: now.getFullYear(),
    en_cours: false,
    description: '',
  }
}

function expViewToDraft(e: ExperienceView): ExpDraft {
  const now = new Date()
  const parseDate = (d: string | null) => {
    if (!d) return { mois: now.getMonth() + 1, annee: now.getFullYear() }
    const parts = d.split('-')
    return {
      mois: parseInt(parts[1] ?? '1', 10),
      annee: parseInt(parts[0] ?? String(now.getFullYear()), 10),
    }
  }
  const debut = parseDate(e.date_debut)
  const fin = parseDate(e.date_fin)
  return {
    poste: e.poste,
    entreprise: e.entreprise ?? '',
    lieu: e.lieu ?? '',
    type_contrat: e.type_contrat ?? '',
    debut_mois: debut.mois,
    debut_annee: debut.annee,
    fin_mois: fin.mois,
    fin_annee: fin.annee,
    en_cours: e.date_fin === null,
    description: e.description ?? '',
  }
}

function openAddExpModal() {
  expEditTarget.value = { kind: 'new' }
  expDraft.value = defaultExpDraft()
  expModal.value = true
}

function openEditExpModal(item: ExpDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    expEditTarget.value = { kind: 'pending', index }
    expDraft.value = { ...pendingExperiences.value[index] }
  } else {
    expEditTarget.value = { kind: 'saved', id: item.id }
    const existingEdit = editedExps.value[item.id]
    if (existingEdit) {
      expDraft.value = { ...existingEdit }
    } else {
      const saved = profile.value?.experiences.find((e: ExperienceView) => e.id === item.id)
      expDraft.value = saved ? expViewToDraft(saved) : defaultExpDraft()
    }
  }
  expModal.value = true
}

function closeExpModal() {
  expModal.value = false
}

function saveExp() {
  if (!expDraft.value.poste.trim()) return
  const target = expEditTarget.value
  if (target.kind === 'new') {
    pendingExperiences.value.unshift({ ...expDraft.value })
  } else if (target.kind === 'pending') {
    pendingExperiences.value[target.index] = { ...expDraft.value }
  } else {
    editedExps.value = { ...editedExps.value, [target.id]: { ...expDraft.value } }
  }
  expModal.value = false
}

function deleteExp(item: ExpDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    pendingExperiences.value.splice(index, 1)
  } else {
    deletedExpIds.value = [...deletedExpIds.value, item.id]
    const updated = { ...editedExps.value }
    delete updated[item.id]
    editedExps.value = updated
  }
}

// ── Formations ────────────────────────────────────────────────────────────────

interface FormDraft {
  titre: string
  ecole: string
  debut_annee: number
  fin_annee: number
  en_cours: boolean
}

type FormEditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

interface FormDisplayItem extends FormationView {
  isPending?: true
  pendingKind?: 'new' | 'edited'
}

const pendingFormations = ref<FormDraft[]>([])
const deletedFormIds = ref<number[]>([])
const editedForms = ref<Record<number, FormDraft>>({})

const formModal = ref(false)
const formDraft = ref<FormDraft>(defaultFormDraft())
const formEditTarget = ref<FormEditTarget>({ kind: 'new' })

const allFormations = computed<FormDisplayItem[]>(() => {
  const toDateStr = (y: number) => `${y}-01-01`

  const pending: FormDisplayItem[] = pendingFormations.value.map((d: FormDraft, i: number) => ({
    id: -(i + 1),
    titre: d.titre,
    ecole: d.ecole || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
    isPending: true,
    pendingKind: 'new',
  }))

  const saved: FormDisplayItem[] = (profile.value?.formations ?? [])
    .filter((f: FormationView) => !deletedFormIds.value.includes(f.id))
    .map((f: FormationView) => {
      const edit = editedForms.value[f.id]
      if (edit) {
        return {
          id: f.id,
          titre: edit.titre,
          ecole: edit.ecole || null,
          date_debut: toDateStr(edit.debut_annee),
          date_fin: edit.en_cours ? null : toDateStr(edit.fin_annee),
          isPending: true as const,
          pendingKind: 'edited' as const,
        }
      }
      return { ...f }
    })

  return [...pending, ...saved]
})

function defaultFormDraft(): FormDraft {
  return {
    titre: '',
    ecole: '',
    debut_annee: new Date().getFullYear(),
    fin_annee: new Date().getFullYear(),
    en_cours: false,
  }
}

function formViewToDraft(f: FormationView): FormDraft {
  const curYear = new Date().getFullYear()
  const parseYear = (d: string | null) => {
    if (!d) return curYear
    return parseInt(d.split('-')[0] ?? String(curYear), 10)
  }
  return {
    titre: f.titre,
    ecole: f.ecole ?? '',
    debut_annee: parseYear(f.date_debut),
    fin_annee: parseYear(f.date_fin),
    en_cours: f.date_fin === null,
  }
}

function openAddFormModal() {
  formEditTarget.value = { kind: 'new' }
  formDraft.value = defaultFormDraft()
  formModal.value = true
}

function openEditFormModal(item: FormDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    formEditTarget.value = { kind: 'pending', index }
    formDraft.value = { ...pendingFormations.value[index] }
  } else {
    formEditTarget.value = { kind: 'saved', id: item.id }
    const existingEdit = editedForms.value[item.id]
    if (existingEdit) {
      formDraft.value = { ...existingEdit }
    } else {
      const saved = profile.value?.formations.find((f: FormationView) => f.id === item.id)
      formDraft.value = saved ? formViewToDraft(saved) : defaultFormDraft()
    }
  }
  formModal.value = true
}

function closeFormModal() {
  formModal.value = false
}

function saveForm() {
  if (!formDraft.value.titre.trim()) return
  const target = formEditTarget.value
  if (target.kind === 'new') {
    pendingFormations.value.unshift({ ...formDraft.value })
  } else if (target.kind === 'pending') {
    pendingFormations.value[target.index] = { ...formDraft.value }
  } else {
    editedForms.value = { ...editedForms.value, [target.id]: { ...formDraft.value } }
  }
  formModal.value = false
}

function deleteForm(item: FormDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    pendingFormations.value.splice(index, 1)
  } else {
    deletedFormIds.value = [...deletedFormIds.value, item.id]
    const updated = { ...editedForms.value }
    delete updated[item.id]
    editedForms.value = updated
  }
}

// ── Projets ───────────────────────────────────────────────────────────────────

interface ProjetDraft {
  titre: string
  description: string
  stack: string
  url: string
  pour_qui: string
  debut_annee: number
  fin_annee: number
  en_cours: boolean
}

type ProjetEditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

interface ProjetDisplayItem extends ProjetView {
  isPending?: true
  pendingKind?: 'new' | 'edited'
}

const pendingProjets = ref<ProjetDraft[]>([])
const deletedProjetIds = ref<number[]>([])
const editedProjets = ref<Record<number, ProjetDraft>>({})

const projetModal = ref(false)
const projetDraft = ref<ProjetDraft>(defaultProjetDraft())
const projetDescTextarea = ref<HTMLTextAreaElement | null>(null)
const projetEditTarget = ref<ProjetEditTarget>({ kind: 'new' })

const allProjets = computed<ProjetDisplayItem[]>(() => {
  const toDateStr = (y: number) => `${y}-01-01`

  const pending: ProjetDisplayItem[] = pendingProjets.value.map((d: ProjetDraft, i: number) => ({
    id: -(i + 1),
    titre: d.titre,
    description: d.description || null,
    stack: d.stack || null,
    url: d.url || null,
    pour_qui: d.pour_qui || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
    isPending: true,
    pendingKind: 'new',
  }))

  const saved: ProjetDisplayItem[] = (profile.value?.projets ?? [])
    .filter((p: ProjetView) => !deletedProjetIds.value.includes(p.id))
    .map((p: ProjetView) => {
      const edit = editedProjets.value[p.id]
      if (edit) {
        return {
          id: p.id,
          titre: edit.titre,
          description: edit.description || null,
          stack: edit.stack || null,
          url: edit.url || null,
          pour_qui: edit.pour_qui || null,
          date_debut: toDateStr(edit.debut_annee),
          date_fin: edit.en_cours ? null : toDateStr(edit.fin_annee),
          isPending: true as const,
          pendingKind: 'edited' as const,
        }
      }
      return { ...p }
    })

  return [...pending, ...saved]
})

function defaultProjetDraft(): ProjetDraft {
  return {
    titre: '',
    description: '',
    stack: '',
    url: '',
    pour_qui: '',
    debut_annee: new Date().getFullYear(),
    fin_annee: new Date().getFullYear(),
    en_cours: false,
  }
}

function projetViewToDraft(p: ProjetView): ProjetDraft {
  const curYear = new Date().getFullYear()
  const parseYear = (d: string | null) =>
    d ? parseInt(d.split('-')[0] ?? String(curYear), 10) : curYear
  return {
    titre: p.titre,
    description: p.description ?? '',
    stack: p.stack ?? '',
    url: p.url ?? '',
    pour_qui: p.pour_qui ?? '',
    debut_annee: parseYear(p.date_debut),
    fin_annee: parseYear(p.date_fin),
    en_cours: p.date_fin === null,
  }
}

function openAddProjetModal() {
  projetEditTarget.value = { kind: 'new' }
  projetDraft.value = defaultProjetDraft()
  projetModal.value = true
}

function openEditProjetModal(item: ProjetDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    projetEditTarget.value = { kind: 'pending', index }
    projetDraft.value = { ...pendingProjets.value[index] }
  } else {
    projetEditTarget.value = { kind: 'saved', id: item.id }
    const existingEdit = editedProjets.value[item.id]
    if (existingEdit) {
      projetDraft.value = { ...existingEdit }
    } else {
      const saved = profile.value?.projets.find((p: ProjetView) => p.id === item.id)
      projetDraft.value = saved ? projetViewToDraft(saved) : defaultProjetDraft()
    }
  }
  projetModal.value = true
}

function closeProjetModal() {
  projetModal.value = false
}

function saveProjet() {
  if (!projetDraft.value.titre.trim()) return
  const target = projetEditTarget.value
  if (target.kind === 'new') {
    pendingProjets.value.unshift({ ...projetDraft.value })
  } else if (target.kind === 'pending') {
    pendingProjets.value[target.index] = { ...projetDraft.value }
  } else {
    editedProjets.value = { ...editedProjets.value, [target.id]: { ...projetDraft.value } }
  }
  projetModal.value = false
}

function deleteProjet(item: ProjetDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    pendingProjets.value.splice(index, 1)
  } else {
    deletedProjetIds.value = [...deletedProjetIds.value, item.id]
    const updated = { ...editedProjets.value }
    delete updated[item.id]
    editedProjets.value = updated
  }
}

// ── CV ────────────────────────────────────────────────────────────────────────

const cvReference = ref<CvView | null>(null)
const cvSaving = ref(false)
const cvError = ref<string | null>(null)
const cvFileInput = ref<HTMLInputElement | null>(null)
const cvJustSaved = ref(false)

const cvPreviewVisible = ref(false)
const cvPreviewLoading = ref(false)
const cvPreviewError = ref<string | null>(null)
const cvPreviewData = ref<string | null>(null)

function initCvFromProfile() {
  cvReference.value = profile.value?.cv_reference ?? null
}

function triggerCvPicker() {
  cvError.value = null
  cvFileInput.value?.click()
}

async function onCvFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  if (!file.name.toLowerCase().endsWith('.typ')) {
    cvError.value = 'Seuls les fichiers .typ sont acceptés.'
    input.value = ''
    return
  }

  cvSaving.value = true
  cvError.value = null

  try {
    const content = await file.text()
    const saved = await invoke<CvView>('save_cv', {
      filename: file.name,
      content,
    })
    cvReference.value = saved
    if (profile.value) profile.value.cv_reference = saved
    cvJustSaved.value = true
  } catch (e: unknown) {
    cvError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors de la sauvegarde du CV.'
  } finally {
    cvSaving.value = false
    input.value = ''
  }
}

async function previewCv() {
  if (!cvReference.value) return
  cvPreviewVisible.value = true
  cvPreviewLoading.value = true
  cvPreviewError.value = null
  cvPreviewData.value = null
  try {
    const b64 = await invoke<string>('compile_cv')
    cvPreviewData.value = b64
  } catch (e: unknown) {
    cvPreviewError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Erreur lors de la compilation du CV.'
  } finally {
    cvPreviewLoading.value = false
  }
}

// ── Compétences ───────────────────────────────────────────────────────────────

interface SkillDraft {
  nom: string
  domaine: string
}

type SkillEditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

interface SkillDisplayItem extends SkillView {
  isPending?: true
  pendingKind?: 'new' | 'edited'
}

const pendingSkills = ref<SkillDraft[]>([])
const deletedSkillIds = ref<number[]>([])
const editedSkills = ref<Record<number, SkillDraft>>({})

const skillModal = ref(false)
const skillDraft = ref<SkillDraft>({ nom: '', domaine: '' })
const skillEditTarget = ref<SkillEditTarget>({ kind: 'new' })
const domainDropdownOpen = ref(false)

const allSkills = computed<SkillDisplayItem[]>(() => {
  const pending: SkillDisplayItem[] = pendingSkills.value.map((d: SkillDraft, i: number) => ({
    id: -(i + 1),
    nom: d.nom,
    domaine: d.domaine,
    isPending: true,
    pendingKind: 'new',
  }))

  const saved: SkillDisplayItem[] = (profile.value?.skills ?? [])
    .filter((s: SkillView) => !deletedSkillIds.value.includes(s.id))
    .map((s: SkillView) => {
      const edit = editedSkills.value[s.id]
      if (edit) {
        return {
          id: s.id,
          nom: edit.nom,
          domaine: edit.domaine,
          isPending: true as const,
          pendingKind: 'edited' as const,
        }
      }
      return { ...s }
    })

  return [...pending, ...saved]
})

// Domaines existants extraits de allSkills (pour le combobox)
const existingDomains = computed<string[]>(() => {
  const set = new Set<string>()
  for (const s of allSkills.value) {
    if (s.domaine) set.add(s.domaine)
  }
  return Array.from(set).sort()
})

const filteredDomains = computed<string[]>(() => {
  const q = skillDraft.value.domaine.trim().toLowerCase()
  if (!q) return existingDomains.value
  return existingDomains.value.filter((d: string) => d.toLowerCase().includes(q))
})

const exactDomainMatch = computed(() =>
  existingDomains.value.some(
    (d: string) => d.toLowerCase() === skillDraft.value.domaine.trim().toLowerCase(),
  ),
)

function pickDomain(d: string) {
  skillDraft.value.domaine = d
  domainDropdownOpen.value = false
}

function onDomainBlur() {
  setTimeout(() => { domainDropdownOpen.value = false }, 150)
}

function openAddSkillModal() {
  skillEditTarget.value = { kind: 'new' }
  skillDraft.value = { nom: '', domaine: '' }
  domainDropdownOpen.value = false
  skillModal.value = true
}

function openEditSkillModal(item: SkillDisplayItem) {
  domainDropdownOpen.value = false
  if (item.id < 0) {
    const index = -item.id - 1
    skillEditTarget.value = { kind: 'pending', index }
    skillDraft.value = { ...pendingSkills.value[index] }
  } else {
    skillEditTarget.value = { kind: 'saved', id: item.id }
    const existingEdit = editedSkills.value[item.id]
    if (existingEdit) {
      skillDraft.value = { ...existingEdit }
    } else {
      const saved = profile.value?.skills.find((s: SkillView) => s.id === item.id)
      skillDraft.value = { nom: saved?.nom ?? '', domaine: saved?.domaine ?? '' }
    }
  }
  skillModal.value = true
}

function closeSkillModal() {
  skillModal.value = false
  domainDropdownOpen.value = false
}

function saveSkill() {
  if (!skillDraft.value.nom.trim() || !skillDraft.value.domaine.trim()) return
  const target = skillEditTarget.value
  if (target.kind === 'new') {
    pendingSkills.value.push({ ...skillDraft.value })
  } else if (target.kind === 'pending') {
    pendingSkills.value[target.index] = { ...skillDraft.value }
  } else {
    editedSkills.value = { ...editedSkills.value, [target.id]: { ...skillDraft.value } }
  }
  skillModal.value = false
}

function deleteSkill(item: SkillDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    pendingSkills.value.splice(index, 1)
  } else {
    deletedSkillIds.value = [...deletedSkillIds.value, item.id]
    const updated = { ...editedSkills.value }
    delete updated[item.id]
    editedSkills.value = updated
  }
}

// ── Langues ───────────────────────────────────────────────────────────────────

interface LangDraft {
  nom: string
  level: string
}

type LangEditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

interface LangDisplayItem extends LangView {
  isPending?: true
  pendingKind?: 'new' | 'edited'
}

const LANG_LEVELS = [
  'Langue maternelle',
  'Bilingue',
  'Courant (C2)',
  'Avancé (C1)',
  'Intermédiaire avancé (B2)',
  'Intermédiaire (B1)',
  'Élémentaire (A2)',
  'Débutant (A1)',
]

const pendingLangs = ref<LangDraft[]>([])
const deletedLangIds = ref<number[]>([])
const editedLangs = ref<Record<number, LangDraft>>({})

const langModal = ref(false)
const langDraft = ref<LangDraft>({ nom: '', level: '' })
const langEditTarget = ref<LangEditTarget>({ kind: 'new' })

const allLangs = computed<LangDisplayItem[]>(() => {
  const pending: LangDisplayItem[] = pendingLangs.value.map((d: LangDraft, i: number) => ({
    id: -(i + 1),
    nom: d.nom || null,
    level: d.level || null,
    isPending: true,
    pendingKind: 'new',
  }))

  const saved: LangDisplayItem[] = (profile.value?.langues ?? [])
    .filter((l: LangView) => !deletedLangIds.value.includes(l.id))
    .map((l: LangView) => {
      const edit = editedLangs.value[l.id]
      if (edit) {
        return {
          id: l.id,
          nom: edit.nom || null,
          level: edit.level || null,
          isPending: true as const,
          pendingKind: 'edited' as const,
        }
      }
      return { ...l }
    })

  return [...pending, ...saved]
})

function openAddLangModal() {
  langEditTarget.value = { kind: 'new' }
  langDraft.value = { nom: '', level: '' }
  langModal.value = true
}

function openEditLangModal(item: LangDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    langEditTarget.value = { kind: 'pending', index }
    langDraft.value = { ...pendingLangs.value[index] }
  } else {
    langEditTarget.value = { kind: 'saved', id: item.id }
    const existingEdit = editedLangs.value[item.id]
    if (existingEdit) {
      langDraft.value = { ...existingEdit }
    } else {
      const saved = profile.value?.langues.find((l: LangView) => l.id === item.id)
      langDraft.value = { nom: saved?.nom ?? '', level: saved?.level ?? '' }
    }
  }
  langModal.value = true
}

function closeLangModal() {
  langModal.value = false
}

function saveLang() {
  if (!langDraft.value.nom.trim()) return
  const target = langEditTarget.value
  if (target.kind === 'new') {
    pendingLangs.value.push({ ...langDraft.value })
  } else if (target.kind === 'pending') {
    pendingLangs.value[target.index] = { ...langDraft.value }
  } else {
    editedLangs.value = { ...editedLangs.value, [target.id]: { ...langDraft.value } }
  }
  langModal.value = false
}

function deleteLang(item: LangDisplayItem) {
  if (item.id < 0) {
    const index = -item.id - 1
    pendingLangs.value.splice(index, 1)
  } else {
    deletedLangIds.value = [...deletedLangIds.value, item.id]
    const updated = { ...editedLangs.value }
    delete updated[item.id]
    editedLangs.value = updated
  }
}

// ── Types d'expérience ────────────────────────────────────────────────────────

const TYPE_LABELS: Record<string, string> = {
  stage: 'Stage',
  alternance: 'Alternance',
  interim: 'Intérim',
  cdd: 'CDD',
  cdi: 'CDI',
}

function typeLabel(t: string | null): string {
  if (!t) return ''
  return TYPE_LABELS[t] ?? t
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function syncUserInfo(u: UserView) {
  const name = [u.prenom, u.nom].filter(Boolean).join(' ')
  const initials = ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase()
  setUserInfo(name, initials || '?')
}

onMounted(async () => {
  try {
    const data = await invoke<ProfileDto>('get_profile')
    profile.value = data
    syncUserInfo(data.user)
    cvReference.value = data.cv_reference ?? null
  } catch (e: unknown) {
    loadError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Impossible de charger le profil.'
  } finally {
    loading.value = false
  }
})

const initials = computed(() => {
  const u = profile.value?.user
  if (!u) return '?'
  return ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase() || '?'
})

const skillsByDomain = computed<Record<string, SkillDisplayItem[]>>(() => {
  const groups: Record<string, SkillDisplayItem[]> = {}
  for (const s of allSkills.value) {
    if (!groups[s.domaine]) groups[s.domaine] = []
    groups[s.domaine].push(s)
  }
  return groups
})

const MONTHS = ['Jan', 'Fév', 'Mar', 'Avr', 'Mai', 'Juin', 'Juil', 'Août', 'Sept', 'Oct', 'Nov', 'Déc']

const yearRange = computed(() => {
  const cur = new Date().getFullYear()
  const years: number[] = []
  for (let y = cur + 2; y >= cur - 30; y--) years.push(y)
  return years
})

function fmtDate(d: string | null): string {
  if (!d) return '?'
  const parts = d.split('-')
  if (parts.length < 2) return d
  return `${MONTHS[parseInt(parts[1], 10) - 1] ?? parts[1]} ${parts[0]}`
}

function fmtRange(start: string | null, end: string | null): string {
  return `${fmtDate(start)} — ${end ? fmtDate(end) : 'Présent'}`
}

// Pour les formations : affiche uniquement l'année (extrait la partie YYYY de "YYYY-MM-DD")
function fmtYear(d: string | null): string {
  if (!d) return '?'
  return d.split('-')[0] ?? d
}

function fmtYearRange(start: string | null, end: string | null): string {
  return `${fmtYear(start)} — ${end ? fmtYear(end) : 'Présent'}`
}
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

        <!-- Expériences -->
        <div class="fieldset">
          <div :class="editMode ? 'fstitle-row' : ''">
            <div class="fstitle">EXPÉRIENCES</div>
            <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddExpModal">
              + Ajouter
            </button>
          </div>
          <template v-if="allExperiences.length">
            <div
              v-for="(exp, i) in allExperiences"
              :key="exp.id"
              class="exp"
              :class="{ 'exp-pending': exp.isPending }"
              :style="i === allExperiences.length - 1 ? 'border-bottom:none' : ''"
            >
              <div class="explogo ph" />
              <div class="f1">
                <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
                  <div class="exprole">{{ exp.poste }}</div>
                  <span v-if="exp.type_contrat" class="tag">{{ typeLabel(exp.type_contrat) }}</span>
                  <span v-if="exp.isPending" class="tag exp-pending-tag">
                    {{ exp.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
                  </span>
                </div>
                <div class="expco">
                  {{ [exp.entreprise, exp.lieu].filter(Boolean).join(' — ') || '—' }}
                </div>
                <div class="expmeta">{{ fmtRange(exp.date_debut, exp.date_fin) }}</div>
                <template v-if="exp.description">
                  <ul v-if="exp.description.includes('•')" class="expd-bullets">
                    <li v-for="(b, i) in parseBulletDescription(exp.description)" :key="i">{{ b }}</li>
                  </ul>
                  <div v-else class="expd">{{ exp.description }}</div>
                </template>
              </div>
              <!-- Boutons édition (visibles uniquement en mode édition) -->
              <div v-if="editMode" class="exp-actions">
                <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditExpModal(exp)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>
                <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteExp(exp)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                    <path d="M10 11v6M14 11v6"/>
                    <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </template>
          <div v-else class="cempty">Aucune expérience renseignée</div>
        </div>

        <!-- Formations -->
        <div class="fieldset">
          <div :class="editMode ? 'fstitle-row' : ''">
            <div class="fstitle">FORMATION</div>
            <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddFormModal">
              + Ajouter
            </button>
          </div>
          <template v-if="allFormations.length">
            <div
              v-for="(form, i) in allFormations"
              :key="form.id"
              class="exp"
              :class="{ 'exp-pending': form.isPending }"
              :style="i === allFormations.length - 1 ? 'border-bottom:none' : ''"
            >
              <div class="explogo ph" />
              <div class="f1">
                <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
                  <div class="exprole">{{ form.titre }}</div>
                  <span v-if="form.isPending" class="tag exp-pending-tag">
                    {{ form.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
                  </span>
                </div>
                <div v-if="form.ecole" class="expco">{{ form.ecole }}</div>
                <div class="expmeta">{{ fmtYearRange(form.date_debut, form.date_fin) }}</div>
              </div>
              <div v-if="editMode" class="exp-actions">
                <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditFormModal(form)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>
                <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteForm(form)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                    <path d="M10 11v6M14 11v6"/>
                    <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </template>
          <div v-else class="cempty">Aucune formation renseignée</div>
        </div>

        <!-- Popup formation -->
        <div v-if="formModal" class="popup-backdrop">
          <div class="popup-box">
            <div class="popup-hd">
              <div class="popup-title">
                {{ formEditTarget.kind === 'new' ? 'Ajouter une formation' : 'Modifier une formation' }}
              </div>
              <button class="popup-close" type="button" @click="closeFormModal">×</button>
            </div>

            <div class="field">
              <label class="flabel">Titre / Diplôme *</label>
              <input v-model="formDraft.titre" class="input" placeholder="Master Informatique" />
            </div>

            <div class="field">
              <label class="flabel">École / Établissement</label>
              <input v-model="formDraft.ecole" class="input" placeholder="EFREI Paris" />
            </div>

            <div class="frow">
              <div class="field">
                <label class="flabel">Année de début</label>
                <select v-model="formDraft.debut_annee" class="input">
                  <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                </select>
              </div>
              <div class="field">
                <label class="flabel">Année de fin</label>
                <div style="display:flex;gap:8px;align-items:center">
                  <select v-if="!formDraft.en_cours" v-model="formDraft.fin_annee" class="input" style="flex:1">
                    <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                  </select>
                  <label class="en-cours-label" :style="formDraft.en_cours ? '' : 'flex:none'">
                    <input type="checkbox" v-model="formDraft.en_cours" />
                    En cours
                  </label>
                </div>
              </div>
            </div>

            <div class="popup-foot">
              <button class="btn" type="button" @click="closeFormModal">Annuler</button>
              <button
                class="btn edit-save"
                type="button"
                :class="{ disabled: !formDraft.titre.trim() }"
                :disabled="!formDraft.titre.trim()"
                @click="saveForm"
              >Valider</button>
            </div>
          </div>
        </div>

        <!-- Projets -->
        <div class="fieldset">
          <div :class="editMode ? 'fstitle-row' : ''">
            <div class="fstitle">PROJETS</div>
            <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddProjetModal">
              + Ajouter
            </button>
          </div>
          <template v-if="allProjets.length">
            <div
              v-for="(proj, i) in allProjets"
              :key="proj.id"
              class="exp"
              :class="{ 'exp-pending': proj.isPending }"
              :style="i === allProjets.length - 1 ? 'border-bottom:none' : ''"
            >
              <div class="explogo ph" />
              <div class="f1">
                <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
                  <div class="exprole">{{ proj.titre }}</div>
                  <span v-if="proj.pour_qui" class="tag">{{ proj.pour_qui }}</span>
                  <span v-if="proj.isPending" class="tag exp-pending-tag">
                    {{ proj.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
                  </span>
                </div>
                <div v-if="proj.stack" class="expco proj-stack">{{ proj.stack }}</div>
                <div class="expmeta">{{ fmtYearRange(proj.date_debut, proj.date_fin) }}</div>
                <template v-if="proj.description">
                  <ul v-if="proj.description.includes('•')" class="expd-bullets">
                    <li v-for="(b, i) in parseBulletDescription(proj.description)" :key="i">{{ b }}</li>
                  </ul>
                  <div v-else class="expd">{{ proj.description }}</div>
                </template>
                <button v-if="proj.url" class="proj-link" type="button" @click="invoke('open_url', { url: proj.url })">{{ proj.url }}</button>
              </div>
              <div v-if="editMode" class="exp-actions">
                <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditProjetModal(proj)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>
                <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteProjet(proj)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                    <path d="M10 11v6M14 11v6"/>
                    <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </template>
          <div v-else class="cempty">Aucun projet renseigné</div>
        </div>

        <!-- Popup projet -->
        <div v-if="projetModal" class="popup-backdrop">
          <div class="popup-box">
            <div class="popup-hd">
              <div class="popup-title">
                {{ projetEditTarget.kind === 'new' ? 'Ajouter un projet' : 'Modifier un projet' }}
              </div>
              <button class="popup-close" type="button" @click="closeProjetModal">×</button>
            </div>

            <div class="field">
              <label class="flabel">Titre *</label>
              <input v-model="projetDraft.titre" class="input" placeholder="Mon super projet" />
            </div>

            <div class="field">
              <label class="flabel">Pour qui</label>
              <input v-model="projetDraft.pour_qui" class="input" placeholder="Perso, École, Client..." />
            </div>

            <div class="field">
              <label class="flabel">Stack utilisé</label>
              <input v-model="projetDraft.stack" class="input" placeholder="Vue 3, Rust, SQLite..." />
            </div>

            <div class="field">
              <label class="flabel">Lien</label>
              <input v-model="projetDraft.url" class="input" placeholder="https://github.com/..." />
            </div>

            <div class="frow">
              <div class="field">
                <label class="flabel">Année de début</label>
                <select v-model="projetDraft.debut_annee" class="input">
                  <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                </select>
              </div>
              <div class="field">
                <label class="flabel">Année de fin</label>
                <div style="display:flex;gap:8px;align-items:center">
                  <select v-if="!projetDraft.en_cours" v-model="projetDraft.fin_annee" class="input" style="flex:1">
                    <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                  </select>
                  <label class="en-cours-label" :style="projetDraft.en_cours ? '' : 'flex:none'">
                    <input type="checkbox" v-model="projetDraft.en_cours" />
                    En cours
                  </label>
                </div>
              </div>
            </div>

            <div class="field">
              <div class="desc-field-hd">
                <label class="flabel" style="margin-bottom:0">Description</label>
                <button
                  type="button"
                  class="bullet-btn"
                  aria-label="Ajouter une puce"
                  @click="projetDescTextarea && insertBullet(projetDescTextarea, projetDraft.description, v => projetDraft.description = v)"
                ><span aria-hidden="true">•</span> Puce</button>
              </div>
              <textarea ref="projetDescTextarea" v-model="projetDraft.description" class="input" rows="3" placeholder="Ce projet consiste à..."></textarea>
            </div>

            <div class="popup-foot">
              <button class="btn" type="button" @click="closeProjetModal">Annuler</button>
              <button
                class="btn edit-save"
                type="button"
                :class="{ disabled: !projetDraft.titre.trim() }"
                :disabled="!projetDraft.titre.trim()"
                @click="saveProjet"
              >Valider</button>
            </div>
          </div>
        </div>

        <!-- Popup ajout d'expérience -->
        <div v-if="expModal" class="popup-backdrop">
          <div class="popup-box">
            <div class="popup-hd">
              <div class="popup-title">
                {{ expEditTarget.kind === 'new' ? 'Ajouter une expérience' : 'Modifier une expérience' }}
              </div>
              <button class="popup-close" type="button" @click="closeExpModal">×</button>
            </div>

            <div class="field">
              <label class="flabel">Intitulé du poste *</label>
              <input v-model="expDraft.poste" class="input" placeholder="Développeur Full-Stack" />
            </div>

            <div class="frow">
              <div class="field">
                <label class="flabel">Entreprise</label>
                <input v-model="expDraft.entreprise" class="input" placeholder="Nom de l'entreprise" />
              </div>
              <div class="field">
                <label class="flabel">Lieu</label>
                <input v-model="expDraft.lieu" class="input" placeholder="Paris, France" />
              </div>
            </div>

            <div class="field">
              <label class="flabel">Type de contrat</label>
              <select v-model="expDraft.type_contrat" class="input">
                <option value="">-- Sélectionner --</option>
                <option value="stage">Stage</option>
                <option value="alternance">Alternance</option>
                <option value="interim">Intérim</option>
                <option value="cdd">CDD</option>
                <option value="cdi">CDI</option>
              </select>
            </div>

            <div class="frow">
              <div class="field">
                <label class="flabel">Début</label>
                <div style="display:flex;gap:8px">
                  <select v-model="expDraft.debut_mois" class="input" style="flex:1">
                    <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
                  </select>
                  <select v-model="expDraft.debut_annee" class="input" style="flex:1">
                    <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                  </select>
                </div>
              </div>
              <div class="field">
                <label class="flabel">Fin</label>
                <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">
                  <template v-if="!expDraft.en_cours">
                    <select v-model="expDraft.fin_mois" class="input" style="flex:1;min-width:80px">
                      <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
                    </select>
                    <select v-model="expDraft.fin_annee" class="input" style="flex:1;min-width:80px">
                      <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                    </select>
                  </template>
                  <label class="en-cours-label">
                    <input type="checkbox" v-model="expDraft.en_cours" />
                    En cours
                  </label>
                </div>
              </div>
            </div>

            <div class="field" style="margin-bottom:0">
              <div class="desc-field-hd">
                <label class="flabel" style="margin-bottom:0">Description</label>
                <button
                  type="button"
                  class="bullet-btn"
                  aria-label="Ajouter une puce"
                  @click="expDescTextarea && insertBullet(expDescTextarea, expDraft.description, v => expDraft.description = v)"
                ><span aria-hidden="true">•</span> Puce</button>
              </div>
              <textarea
                ref="expDescTextarea"
                v-model="expDraft.description"
                class="input"
                style="height:100px"
                placeholder="Décris tes missions, technologies utilisées…"
              />
            </div>

            <div class="popup-foot">
              <button class="btn" type="button" @click="closeExpModal">Annuler</button>
              <button
                class="btn edit-save"
                type="button"
                :class="{ disabled: !expDraft.poste.trim() }"
                :disabled="!expDraft.poste.trim()"
                @click="saveExp"
              >Valider</button>
            </div>
          </div>
        </div>
      </div>

      <!-- ── Colonne droite ── -->
      <div>
        <!-- Compétences -->
        <div class="fieldset">
          <div :class="editMode ? 'fstitle-row' : ''">
            <div class="fstitle">COMPÉTENCES</div>
            <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddSkillModal">
              + Ajouter
            </button>
          </div>

          <!-- Vue normale : chips groupées par domaine -->
          <template v-if="!editMode">
            <template v-if="allSkills.length">
              <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
                <div class="cmeta" style="margin-bottom:6px;margin-top:10px">{{ domaine }}</div>
                <div class="chips" style="margin-bottom:4px">
                  <span v-for="s in items" :key="s.id" class="chip">{{ s.nom }}</span>
                </div>
              </template>
            </template>
            <div v-else class="cempty">Aucune compétence renseignée</div>
          </template>

          <!-- Vue édition : liste avec boutons édition/suppression -->
          <template v-else>
            <template v-if="allSkills.length">
              <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
                <div class="cmeta" style="margin-top:12px;margin-bottom:4px">{{ domaine }}</div>
                <div
                  v-for="s in items"
                  :key="s.id"
                  class="skill-edit-row"
                  :class="{ 'exp-pending': s.isPending }"
                >
                  <span class="skill-nom">{{ s.nom }}</span>
                  <span v-if="s.isPending" class="tag exp-pending-tag" style="flex:none">
                    {{ s.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
                  </span>
                  <div class="exp-actions">
                    <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditSkillModal(s)">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                      </svg>
                    </button>
                    <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteSkill(s)">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="3 6 5 6 21 6"/>
                        <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                        <path d="M10 11v6M14 11v6"/>
                        <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                      </svg>
                    </button>
                  </div>
                </div>
              </template>
            </template>
            <div v-else class="cempty">Aucune compétence renseignée</div>
          </template>
        </div>

        <!-- Popup compétence -->
        <div v-if="skillModal" class="popup-backdrop">
          <div class="popup-box" style="width:440px">
            <div class="popup-hd">
              <div class="popup-title">
                {{ skillEditTarget.kind === 'new' ? 'Ajouter une compétence' : 'Modifier une compétence' }}
              </div>
              <button class="popup-close" type="button" @click="closeSkillModal">×</button>
            </div>

            <div class="field">
              <label class="flabel">Nom de la compétence *</label>
              <input v-model="skillDraft.nom" class="input" placeholder="Python, Vue.js, Figma…" />
            </div>

            <div class="field" style="margin-bottom:0">
              <label class="flabel">Domaine *</label>
              <div class="domain-combo-wrap">
                <input
                  v-model="skillDraft.domaine"
                  class="input"
                  placeholder="Langages, Frameworks, Outils…"
                  autocomplete="off"
                  @focus="domainDropdownOpen = true"
                  @input="domainDropdownOpen = true"
                  @blur="onDomainBlur"
                />
                <div
                  v-if="domainDropdownOpen && (filteredDomains.length > 0 || skillDraft.domaine.trim())"
                  class="domain-dropdown"
                >
                  <div
                    v-for="d in filteredDomains"
                    :key="d"
                    class="domain-opt"
                    @mousedown.prevent="pickDomain(d)"
                  >{{ d }}</div>
                  <div
                    v-if="skillDraft.domaine.trim() && !exactDomainMatch"
                    class="domain-opt domain-opt-new"
                    @mousedown.prevent="domainDropdownOpen = false"
                  >
                    Créer <strong>« {{ skillDraft.domaine.trim() }} »</strong>
                  </div>
                  <div
                    v-if="!filteredDomains.length && !skillDraft.domaine.trim()"
                    class="domain-opt-empty"
                  >Aucun domaine existant</div>
                </div>
              </div>
            </div>

            <div class="popup-foot">
              <button class="btn" type="button" @click="closeSkillModal">Annuler</button>
              <button
                class="btn edit-save"
                type="button"
                :class="{ disabled: !skillDraft.nom.trim() || !skillDraft.domaine.trim() }"
                :disabled="!skillDraft.nom.trim() || !skillDraft.domaine.trim()"
                @click="saveSkill"
              >Valider</button>
            </div>
          </div>
        </div>

        <!-- Langues -->
        <div class="fieldset">
          <div :class="editMode ? 'fstitle-row' : ''">
            <div class="fstitle">LANGUES</div>
            <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddLangModal">
              + Ajouter
            </button>
          </div>
          <template v-if="allLangs.length">
            <div
              v-for="(lang, i) in allLangs"
              :key="lang.id"
              class="setrow lang-row"
              :class="{ 'exp-pending': lang.isPending }"
              :style="i === allLangs.length - 1 ? 'border-bottom:none' : ''"
            >
              <span class="setl">{{ lang.nom ?? '—' }}</span>
              <span class="cmeta" style="flex:1;text-align:right">{{ lang.level ?? '' }}</span>
              <span v-if="lang.isPending" class="tag exp-pending-tag" style="flex:none">
                {{ lang.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
              </span>
              <div v-if="editMode" class="exp-actions" style="margin-left:8px">
                <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditLangModal(lang)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>
                <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteLang(lang)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                    <path d="M10 11v6M14 11v6"/>
                    <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </template>
          <div v-else class="cempty">Aucune langue renseignée</div>
        </div>

        <!-- Popup langue -->
        <div v-if="langModal" class="popup-backdrop">
          <div class="popup-box" style="width:420px">
            <div class="popup-hd">
              <div class="popup-title">
                {{ langEditTarget.kind === 'new' ? 'Ajouter une langue' : 'Modifier une langue' }}
              </div>
              <button class="popup-close" type="button" @click="closeLangModal">×</button>
            </div>

            <div class="field">
              <label class="flabel">Langue *</label>
              <input v-model="langDraft.nom" class="input" placeholder="Anglais, Espagnol…" />
            </div>

            <div class="field" style="margin-bottom:0">
              <label class="flabel">Niveau</label>
              <select v-model="langDraft.level" class="input">
                <option value="">-- Sélectionner --</option>
                <option v-for="lvl in LANG_LEVELS" :key="lvl" :value="lvl">{{ lvl }}</option>
              </select>
            </div>

            <div class="popup-foot">
              <button class="btn" type="button" @click="closeLangModal">Annuler</button>
              <button
                class="btn edit-save"
                type="button"
                :class="{ disabled: !langDraft.nom.trim() }"
                :disabled="!langDraft.nom.trim()"
                @click="saveLang"
              >Valider</button>
            </div>
          </div>
        </div>

        <!-- CV Typst -->
        <div class="fieldset">
          <div class="fstitle">CV (TYPST)</div>

          <!-- Affichage du CV actuel -->
          <div v-if="cvReference" class="cv-current">
            <div class="cv-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
                <polyline points="10 9 9 9 8 9"/>
              </svg>
            </div>
            <div class="cv-info">
              <div class="cv-label">{{ cvReference.label ?? 'cv_reference.typ' }}</div>
              <div class="cv-meta">Enregistré le {{ new Date(cvReference.created_at).toLocaleDateString('fr-FR') }}</div>
            </div>
            <button class="cv-preview-btn" type="button" title="Visualiser le CV compilé" @click="previewCv">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            </button>
          </div>
          <div v-else class="cempty">Aucun CV renseigné</div>

          <!-- Bouton en mode édition -->
          <div v-if="editMode" class="cv-actions">
            <input
              ref="cvFileInput"
              type="file"
              accept=".typ"
              style="display:none"
              @change="onCvFileSelected"
            />
            <button
              class="btn cv-upload-btn"
              type="button"
              :disabled="cvSaving"
              @click="triggerCvPicker"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right:6px">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              {{ cvSaving ? 'Sauvegarde…' : cvReference ? 'Remplacer le CV' : 'Ajouter un CV' }}
            </button>
            <span v-if="cvError" class="cv-error">{{ cvError }}</span>
          </div>
        </div>

      </div>
    </div>

    <!-- Modale prévisualisation CV -->
    <Teleport to="body">
      <div v-if="cvPreviewVisible" class="cv-modal-overlay" @click.self="cvPreviewVisible = false">
        <div class="cv-modal">
          <div class="cv-modal-header">
            <span class="cv-modal-title">Aperçu du CV</span>
            <button class="cv-modal-close" type="button" @click="cvPreviewVisible = false">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <div class="cv-modal-body">
            <div v-if="cvPreviewLoading" class="cv-modal-loading">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="cv-spinner">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
              </svg>
              Compilation en cours…
            </div>
            <div v-else-if="cvPreviewError" class="cv-modal-error">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              {{ cvPreviewError }}
            </div>
            <iframe
              v-else-if="cvPreviewData"
              :src="`data:application/pdf;base64,${cvPreviewData}`"
              class="cv-modal-iframe"
              title="Aperçu PDF du CV"
            />
          </div>
        </div>
      </div>
    </Teleport>

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
        @click="saveDescription"
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

.edit-save {
  background: #2a9e5f;
  color: #fff;
  border-color: #2a9e5f;
}
.edit-save:hover { background: #228a50; border-color: #228a50; }
.edit-save.disabled { opacity: 0.4; cursor: not-allowed; pointer-events: none; }

.permis-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  cursor: pointer;
  user-select: none;
  margin-top: 2px;
}

.edit-cancel {
  background: #e03e3e;
  color: #fff;
  border-color: #e03e3e;
}
.edit-cancel:hover { background: #c93333; border-color: #c93333; }

.exp-add-btn {
  height: 30px;
  padding: 0 12px;
  font-size: 12px;
}

.en-cours-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  user-select: none;
}

.exp-pending { opacity: 0.72; }

.exp-pending-tag {
  border-color: #e0a050;
  color: #a06010;
  background: #fdf6ec;
}

.exp-actions {
  display: flex;
  flex-direction: row;
  gap: 5px;
  flex: none;
  align-self: center;
  margin-left: 6px;
}

.exp-btn {
  width: 30px;
  height: 30px;
  border-radius: 7px;
  border: 1px solid #e2e2dc;
  background: #fafaf8;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex: none;
}

.exp-btn-edit { color: #76766f; }
.exp-btn-edit:hover { background: #f1f1ec; color: #1d1d1b; border-color: #c2c2ba; }

.exp-btn-delete { color: #c53a3a; border-color: #f0d0d0; background: #fdf6f6; }
.exp-btn-delete:hover { background: #fce8e8; border-color: #e09090; }

/* ── CV ── */
.cv-current {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0 6px;
}

.cv-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: #f0f4ff;
  border: 1px solid #d0d8f0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4a70d9;
  flex: none;
}

.cv-info { flex: 1; min-width: 0; }

.cv-label {
  font-size: 13px;
  font-weight: 600;
  color: #1d1d1b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cv-meta {
  font-size: 11px;
  color: #888;
  margin-top: 1px;
}

.cv-actions {
  margin-top: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.cv-upload-btn {
  display: flex;
  align-items: center;
  font-size: 13px;
  padding: 7px 14px;
  border-radius: 8px;
  background: #f5f5f2;
  border: 1px solid #d4d4ce;
  color: #1d1d1b;
  cursor: pointer;
  transition: background .15s;
}
.cv-upload-btn:hover:not(:disabled) { background: #eaeae6; border-color: #b4b4aa; }
.cv-upload-btn:disabled { opacity: 0.6; cursor: default; }

.cv-error {
  font-size: 12px;
  color: #c53a3a;
}

.cv-preview-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 30px;
  height: 30px;
  border-radius: 7px;
  border: 1px solid #d0d8f0;
  background: #f0f4ff;
  color: #4a70d9;
  cursor: pointer;
  transition: background .15s, border-color .15s;
}
.cv-preview-btn:hover { background: #e4ecff; border-color: #a0b4e8; }

/* ── Modale aperçu CV ── */
.cv-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cv-modal {
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.22);
  width: min(860px, calc(100vw - 48px));
  height: min(92vh, 1000px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.cv-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid #e6e6e1;
  flex: none;
}

.cv-modal-title {
  font-size: 14px;
  font-weight: 600;
  color: #1d1d1b;
}

.cv-modal-close {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  border: 1px solid #e2e2dc;
  background: #fafaf8;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #76766f;
}
.cv-modal-close:hover { background: #f1f1ec; color: #1d1d1b; }

.cv-modal-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cv-modal-iframe {
  width: 100%;
  height: 100%;
  border: none;
  display: block;
}

.cv-modal-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  color: #666;
  font-size: 13.5px;
}

.cv-spinner {
  animation: spin 1s linear infinite;
  color: #4a70d9;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.cv-modal-error {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  color: #c53a3a;
  font-size: 13px;
  max-width: 480px;
  padding: 20px;
  background: #fdf6f6;
  border: 1px solid #f0d0d0;
  border-radius: 8px;
  line-height: 1.5;
}

/* ── Projets ── */
.proj-stack {
  font-size: 11px;
  color: #666;
  font-family: monospace;
}

.proj-link {
  font-size: 11px;
  color: #4a90d9;
  text-decoration: none;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  display: inline-block;
  margin-top: 2px;
  word-break: break-all;
  text-align: left;
}
.proj-link:hover { text-decoration: underline; }

/* ── Compétences — édition ── */
.skill-edit-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 0;
  border-bottom: 1px solid #f5f5f2;
}
.skill-edit-row:last-child { border-bottom: none; }

.skill-nom {
  flex: 1;
  font-size: 13px;
  color: #1d1d1b;
}

/* ── Combobox domaine ── */
.domain-combo-wrap { position: relative; }

.domain-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid #d4d4ce;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0,0,0,.12);
  z-index: 200;
  overflow: hidden;
  max-height: 200px;
  overflow-y: auto;
}

.domain-opt {
  padding: 9px 13px;
  font-size: 13.5px;
  cursor: pointer;
  color: #1d1d1b;
}
.domain-opt:hover { background: #f5f5f2; }

.domain-opt-new {
  border-top: 1px solid #f0f0ed;
  color: #2a9e5f;
  display: flex;
  gap: 4px;
}
.domain-opt-new:hover { background: #f0faf4; }

.domain-opt-empty {
  padding: 9px 13px;
  font-size: 12px;
  color: #aeaea7;
  font-family: 'IBM Plex Mono', monospace;
  cursor: default;
}

/* ── Bullet insert toolbar ── */
.desc-field-hd {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 5px;
}

.bullet-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  padding: 0 9px;
  font-size: 12px;
  border-radius: 5px;
  border: 1px solid #d4d4ce;
  background: #fafaf8;
  color: #56564f;
  cursor: pointer;
  line-height: 1;
  transition: background .12s, border-color .12s;
}
.bullet-btn:hover { background: #f1f1ec; border-color: #b4b4aa; color: #1d1d1b; }
</style>
