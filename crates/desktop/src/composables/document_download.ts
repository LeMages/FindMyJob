/**
 * Téléchargement local des documents PDF d'une candidature (CV, lettre de
 * motivation), partagé par la page de détail, la modale d'aperçu et le flux
 * « Postuler ». Seul point d'appel de la commande Tauri `download_document`.
 */

// Importation des modules Vue / framework
import { invoke } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'

// Importation des modules internes
import { buildApplyFolderName, buildDocumentFileName, type DownloadDocType } from '@/composables/document_file_name'
import type { CandidatureDetail } from '@/types'

/** Identité de l'utilisateur, utilisée dans le nom des fichiers. */
export interface DocumentOwner {
  nom: string | null
  prenom: string | null
}

/** Texte de candidature rédigé par l'IA (mail ou message), exporté en `.txt`. */
export interface ApplyMessageExport {
  fileName: string
  content: string
}

/** Sous-dossier des téléchargements dédié aux documents de postulation. */
const APPLY_ROOT_FOLDER = 'FindMyJob'

/**
 * Télécharge un document d'une candidature dans un dossier local, sous son
 * nom normalisé (`CV_…` / `LM_…`). Un fichier existant est écrasé.
 *
 * @param docType - `'cv'` ou `'lm'`.
 * @param docPath - Chemin du document côté agent.
 * @param candidature - Candidature d'origine (entreprise, poste).
 * @param owner - Identité de l'utilisateur.
 * @param targetDir - Dossier de destination (défaut : dossier de téléchargements).
 * @returns Chemin complet du fichier sauvegardé.
 */
export async function downloadCandidatureDocument(
  docType: DownloadDocType,
  docPath: string,
  candidature: CandidatureDetail,
  owner: DocumentOwner,
  targetDir?: string,
): Promise<string> {
  const dir = targetDir ?? await invoke<string>('get_downloads_dir')
  const fileName = buildDocumentFileName(docType, owner.nom, owner.prenom, candidature.entreprise, candidature.poste)
  const savePath = await join(dir, fileName)
  return invoke<string>('download_document', { docPath, savePath })
}

/**
 * Télécharge le CV et la lettre de motivation d'une candidature dans
 * `Téléchargements/FindMyJob/{Entreprise}_{Poste}/`, avec le mail ou message
 * rédigé par l'IA en `.txt` s'il y en a un. Un document absent (chemin
 * `null`) est simplement ignoré.
 *
 * @param candidature - Candidature venant d'être postulée.
 * @param owner - Identité de l'utilisateur.
 * @param message - Mail ou message à exporter, `null` pour « postuler sans message ».
 * @returns Dossier de destination, ou `null` si rien n'était à exporter.
 */
export async function downloadApplyDocuments(
  candidature: CandidatureDetail,
  owner: DocumentOwner,
  message: ApplyMessageExport | null,
): Promise<string | null> {
  const docs: [DownloadDocType, string | null][] = [
    ['cv', candidature.cv_path],
    ['lm', candidature.lettre_motivation_path],
  ]
  const available = docs.filter((d): d is [DownloadDocType, string] => d[1] !== null)
  if (available.length === 0 && !message) return null

  const downloadsDir = await invoke<string>('get_downloads_dir')
  const targetDir = await join(downloadsDir, APPLY_ROOT_FOLDER, buildApplyFolderName(candidature.entreprise, candidature.poste))
  const writes: Promise<string>[] = available.map(
    ([docType, path]) => downloadCandidatureDocument(docType, path, candidature, owner, targetDir),
  )
  if (message) {
    const savePath = await join(targetDir, message.fileName)
    writes.push(invoke<string>('save_text_file', { savePath, content: message.content }))
  }
  await Promise.all(writes)
  return targetDir
}
