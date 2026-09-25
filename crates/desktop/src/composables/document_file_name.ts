/**
 * Nommage des documents PDF téléchargés depuis une candidature (CV, lettre
 * de motivation) : `CV_{Nom}_{Prenom}_{Entreprise}_{Poste}.pdf`.
 */

/** Type de document téléchargeable, utilisé comme préfixe du nom de fichier. */
export type DownloadDocType = 'cv' | 'lm'

/** Longueur max d'un segment de nom, pour garder des noms de fichier lisibles. */
const MAX_PART_LENGTH = 40

/** Segment utilisé quand une information est absente ou vide après nettoyage. */
const EMPTY_PART_FALLBACK = 'Inconnu'

/**
 * Nettoie une chaîne pour l'utiliser comme segment de nom de fichier :
 * accents retirés, caractères hors `[A-Za-z0-9-]` remplacés par `_`,
 * tronquée à `MAX_PART_LENGTH` caractères.
 *
 * @param str - Chaîne brute (peut être `null` ou vide).
 * @returns Segment sûr pour tous les OS, jamais vide.
 */
export function sanitizeFileNamePart(str: string | null | undefined): string {
  const cleaned = (str ?? '')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^A-Za-z0-9-]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, MAX_PART_LENGTH)
    .replace(/_+$/, '')
  return cleaned || EMPTY_PART_FALLBACK
}

/**
 * Construit le nom de fichier d'un document téléchargé.
 *
 * @param docType - `'cv'` ou `'lm'`.
 * @param lastName - Nom de l'utilisateur.
 * @param firstName - Prénom de l'utilisateur.
 * @param company - Entreprise de la candidature.
 * @param jobTitle - Intitulé du poste.
 * @returns Nom de fichier, ex. `CV_Cupif_Loic_Google_Data_Engineer.pdf`.
 */
export function buildDocumentFileName(
  docType: DownloadDocType,
  lastName: string | null,
  firstName: string | null,
  company: string | null,
  jobTitle: string | null,
): string {
  const prefix = docType === 'cv' ? 'CV' : 'LM'
  const parts = [lastName, firstName, company, jobTitle].map(sanitizeFileNamePart)
  return `${prefix}_${parts.join('_')}.pdf`
}

/**
 * Construit le nom du sous-dossier regroupant les documents d'une candidature
 * lors de la postulation.
 *
 * @param company - Entreprise de la candidature.
 * @param jobTitle - Intitulé du poste.
 * @returns Nom de dossier, ex. `Google_Data_Engineer`.
 */
export function buildApplyFolderName(company: string | null, jobTitle: string | null): string {
  return `${sanitizeFileNamePart(company)}_${sanitizeFileNamePart(jobTitle)}`
}
