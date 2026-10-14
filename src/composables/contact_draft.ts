/**
 * Lignes de saisie d'un contact (`ContactDraft`) partagées par le formulaire
 * de nouvelle candidature et la popup de contact du détail : création d'une
 * ligne vide, pré-remplie depuis un contact existant ou extrait d'une offre,
 * et conversion vers les DTO d'entrée du backend. Le trim et la validation
 * du nom restent faits par le backend.
 */

// Importation des modules internes
import type { CandidatureContactInput, ContactDraft, ContactInput, ContactView, ScrapedContact } from '@/types'

let nextDraftKey = 0

/**
 * Crée une ligne de saisie vide, en mode « nouveau contact ».
 *
 * @returns Ligne vide avec une clé locale unique.
 */
export function emptyContactDraft(): ContactDraft {
  return {
    key: nextDraftKey++,
    mode: 'new',
    contact_id: null,
    nom: '',
    role: '',
    email: '',
    telephone: '',
    notes: '',
  }
}

/**
 * Crée une ligne pré-remplie avec un contact existant, pour le modifier.
 *
 * @param contact - Contact du carnet.
 * @returns Ligne en mode « nouveau contact » portant ses valeurs.
 */
export function draftFromContact(contact: ContactView): ContactDraft {
  return {
    ...emptyContactDraft(),
    nom: contact.nom,
    role: contact.role ?? '',
    email: contact.email ?? '',
    telephone: contact.telephone ?? '',
    notes: contact.notes ?? '',
  }
}

/**
 * Crée une ligne pré-remplie avec le contact extrait d'une offre.
 *
 * @param contact - Contact extrait.
 * @returns Ligne en mode « nouveau contact ».
 */
export function draftFromScraped(contact: ScrapedContact): ContactDraft {
  return {
    ...emptyContactDraft(),
    nom: contact.nom ?? '',
    role: contact.role ?? '',
    email: contact.email ?? '',
    telephone: contact.telephone ?? '',
  }
}

/**
 * Convertit les champs d'une ligne en saisie de contact.
 *
 * @param draft - Ligne de saisie.
 * @returns Saisie envoyée au backend (champs vides en `null`).
 */
export function draftToContactInput(draft: ContactDraft): ContactInput {
  return {
    nom: draft.nom,
    role: draft.role || null,
    email: draft.email || null,
    telephone: draft.telephone || null,
    notes: draft.notes || null,
  }
}

/**
 * Convertit une ligne en contact à lier à une nouvelle candidature.
 *
 * @param draft - Ligne de saisie.
 * @returns Contact existant (si un contact du carnet est choisi) ou nouveau.
 */
export function draftToCandidatureContact(draft: ContactDraft): CandidatureContactInput {
  if (draft.mode === 'existing' && draft.contact_id !== null) {
    return { kind: 'existing', contact_id: draft.contact_id }
  }
  return { kind: 'new', contact: draftToContactInput(draft) }
}
