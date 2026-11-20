/**
 * Liste de vérification « Prêt à générer ? » du formulaire de nouvelle
 * candidature : ce qui manque avant de pouvoir créer la candidature et
 * générer ses documents.
 */

/** Point vérifié avant la génération. */
export type ChecklistKey = 'fields' | 'documents' | 'city' | 'session'

/** Un point de la liste de vérification. */
export interface ChecklistItem {
  key: ChecklistKey
  ok: boolean
  /** Le point se corrige dans les paramètres de la session (lien affiché s'il manque). */
  fixInSessionSettings: boolean
}

/** État du formulaire et de la session lu par la liste de vérification. */
export interface ChecklistInput {
  /** Les champs obligatoires du formulaire sont-ils remplis ? */
  fieldsValid: boolean
  /** Les modèles de CV et de lettre de la session sont-ils prêts ? */
  documentsReady: boolean
  /** Ville de la session (expéditeur des documents), `null` si absente. */
  sessionCity: string | null
  /** La session est-elle en lecture seule (en pause ou terminée) ? */
  sessionLocked: boolean
}

/** Résultat de la vérification : les points à afficher et l'autorisation de créer. */
export interface Checklist {
  items: ChecklistItem[]
  canGenerate: boolean
}

/**
 * Construit la liste de vérification affichée à côté du formulaire. Le
 * point « session » n'apparaît que lorsque la session est verrouillée (il
 * n'a rien à vérifier sinon) ; tous les points doivent être remplis pour
 * créer la candidature, la ville comprise (le backend la refuse sans).
 *
 * @param input - État du formulaire et de la session.
 * @returns Les points et l'autorisation de lancer la génération.
 */
export function buildGenerationChecklist(input: ChecklistInput): Checklist {
  const items: ChecklistItem[] = [
    { key: 'fields', ok: input.fieldsValid, fixInSessionSettings: false },
    { key: 'documents', ok: input.documentsReady, fixInSessionSettings: true },
    { key: 'city', ok: (input.sessionCity ?? '').trim() !== '', fixInSessionSettings: true },
  ]
  if (input.sessionLocked) items.push({ key: 'session', ok: false, fixInSessionSettings: true })
  return { items, canGenerate: items.every((item) => item.ok) }
}
