/**
 * État de préparation des documents de référence (CV, lettre de motivation)
 * de la session courante — conditionne la création d'une candidature.
 * État partagé (singleton au niveau module) entre la page de suivi
 * (bouton « Nouvelle candidature ») et le formulaire de création.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { t } from '@/i18n'
import { currentSession } from '@/stores/app'
import type { DocumentsStatus } from '@/types'

const documentsStatus = ref<DocumentsStatus | null>(null)

/**
 * Recharge l'état des documents de référence de la session courante.
 * Échec silencieux volontaire (comme `loadSessions` dans `stores/app.ts`) : le garde-fou serveur reste la source de vérité,
 * cet état n'est qu'un confort d'affichage.
 */
async function loadDocumentsStatus() {
  const sid = currentSession.value?.id
  if (!sid) {
    documentsStatus.value = null
    return
  }
  try {
    documentsStatus.value = await invoke<DocumentsStatus>('get_documents_status', { sessionId: sid })
  } catch {
    documentsStatus.value = null
  }
}

/**
 * Vrai lorsqu'une session est sélectionnée mais que ses documents de
 * référence ne sont pas encore prêts — bloque alors la création d'une
 * candidature pour cette session.
 */
const documentsBlocking = computed(() =>
  currentSession.value != null && documentsStatus.value !== null && !documentsStatus.value.all_ready
)

/**
 * Liste les documents de référence manquants pour la session courante, pour
 * affichage (re-traduit au changement de langue, `t` étant réactif).
 */
const missingDocumentsMessage = computed(() => {
  const status = documentsStatus.value
  if (!status) return ''
  let list: string
  if (!status.cv_ready && !status.lm_ready) list = t('documents.missing.both')
  else if (!status.cv_ready) list = t('documents.missing.cv')
  else if (!status.lm_ready) list = t('documents.missing.lm')
  else list = ''
  return t('documents.missing.message', { list })
})

/**
 * Expose l'état de préparation des documents de référence (réactif) et la
 * fonction pour le rafraîchir. Chaque vue appelante déclenche elle-même le
 * chargement (`onMounted` + `watch` sur la session), comme `useFonts`.
 */
export function useDocumentsStatus() {
  return { documentsStatus, documentsBlocking, missingDocumentsMessage, loadDocumentsStatus }
}
