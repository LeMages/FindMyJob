/**
 * Interprétation des problèmes relevés à l'import d'une référence (CV ou
 * lettre) au contrat FindMyJob. Le backend renvoie des codes typés
 * (`{ kind: 'InvalidReference', message: [{ code, path, line, detail }] }`
 * pour un refus, `warnings` sur la vue après un import réussi) : c'est ici
 * que l'on choisit le texte affiché, dans la langue de l'interface.
 */

// Importation des modules internes
import { t } from '@/i18n'
import type { ReferenceIssue } from '@/types'

/**
 * Lit les problèmes d'une erreur d'import, s'il s'agit d'une erreur
 * `InvalidReference` du backend.
 *
 * @param e - Erreur interceptée autour d'un `invoke()`.
 * @returns Les problèmes relevés, ou `null` pour toute autre erreur.
 */
export function referenceIssuesOf(e: unknown): ReferenceIssue[] | null {
  if (!e || typeof e !== 'object' || !('kind' in e) || !('message' in e)) return null
  const { kind, message } = e as { kind: unknown; message: unknown }
  if (kind !== 'InvalidReference' || !Array.isArray(message)) return null
  return message as ReferenceIssue[]
}

/**
 * Texte lisible d'un problème : explication du code, suivie de son contexte
 * (ligne, champ, détail brut) quand il est connu.
 *
 * @param issue - Problème relevé par la validation.
 * @returns Le message à afficher.
 */
export function referenceIssueMessage(issue: ReferenceIssue): string {
  const context: string[] = []
  if (issue.line != null) context.push(t('documents.reference.issue_context.line', { line: issue.line }))
  if (issue.path) context.push(t('documents.reference.issue_context.path', { path: issue.path }))
  if (issue.detail) context.push(issue.detail)
  const explanation = t(`documents.reference.issue.${issue.code}`)
  return context.length > 0 ? `${explanation} (${context.join(' · ')})` : explanation
}
