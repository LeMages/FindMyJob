//! Tests de la sélection des entrées du profil
//! (`findmyjob_core::docgen::selection`).

mod common;

// Importation des modules internes
use findmyjob_core::docgen::selection::select;

#[test]
fn toutes_les_experiences_sont_conservees() {
    let selection = select(&common::profile(), &common::matching(), 2);
    assert_eq!(selection.experience_ids(), vec![10, 11]);
}

#[test]
fn projets_du_matching_dans_l_ordre_du_profil() {
    // Le matching désigne 22 puis 20 : l'affichage suit l'ordre du profil.
    let selection = select(&common::profile(), &common::matching(), 2);
    assert_eq!(selection.project_ids(), vec![20, 22]);
    assert!(selection.projects.iter().all(|p| p.notes.is_some()));
}

#[test]
fn complement_par_les_projets_les_plus_recents() {
    let selection = select(&common::profile(), &common::matching(), 3);
    assert_eq!(selection.project_ids(), vec![20, 21, 22]);
}

#[test]
fn identifiants_inconnus_ignores() {
    let mut matching = common::matching();
    matching.relevant_projects.insert(0, common::matched(999));
    let selection = select(&common::profile(), &matching, 1);
    assert_eq!(selection.project_ids(), vec![22]);
}

#[test]
fn aucun_projet_si_la_reference_n_en_a_pas() {
    let selection = select(&common::profile(), &common::matching(), 0);
    assert!(selection.project_ids().is_empty());
}
