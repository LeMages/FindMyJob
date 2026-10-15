//! Tests de la sélection des entrées du profil et de la validation du
//! choix des projets par le matching (`findmyjob_lib::docgen::selection`).

// Importation des modules internes
use crate::common;
use findmyjob_lib::docgen::selection::{expected_project_count, project_id_problems, select};

#[test]
fn toutes_les_experiences_sont_conservees() {
    let selection = select(&common::profile(), &[22, 20]);
    assert_eq!(selection.experience_ids(), vec![10, 11]);
}

#[test]
fn projets_du_matching_dans_l_ordre_du_profil() {
    // Le matching désigne 22 puis 20 : l'affichage suit l'ordre du profil.
    let selection = select(&common::profile(), &[22, 20]);
    assert_eq!(selection.project_ids(), vec![20, 22]);
}

#[test]
fn aucun_complement_par_les_projets_recents() {
    let selection = select(&common::profile(), &[22]);
    assert_eq!(selection.project_ids(), vec![22], "seuls les projets choisis sont affichés");
}

#[test]
fn aucun_projet_si_rien_n_est_choisi() {
    assert!(select(&common::profile(), &[]).project_ids().is_empty());
}

#[test]
fn nombre_attendu_borne_par_le_profil() {
    let profile = common::profile();
    assert_eq!(expected_project_count(&profile, 2), 2);
    assert_eq!(expected_project_count(&profile, 9), 4, "jamais plus que les projets du profil");
    assert_eq!(expected_project_count(&profile, 0), 0);
    let mut empty = common::profile();
    empty.projets.clear();
    assert_eq!(expected_project_count(&empty, 2), 0);
}

#[test]
fn choix_valide_sans_probleme() {
    assert!(project_id_problems(&[22, 20], &common::profile(), 2).is_empty());
    assert!(project_id_problems(&[], &common::profile(), 0).is_empty());
}

#[test]
fn identifiant_inconnu_signale() {
    let problems = project_id_problems(&[999, 20], &common::profile(), 2);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("l'id 999 n'est pas un projet du profil"), "{problems:?}");
}

#[test]
fn doublon_signale() {
    let problems = project_id_problems(&[20, 20], &common::profile(), 2);
    assert_eq!(problems, vec!["project_ids : l'id 20 apparaît plusieurs fois.".to_string()]);
}

#[test]
fn nombre_incorrect_signale() {
    let too_few = project_id_problems(&[20], &common::profile(), 2);
    assert_eq!(too_few, vec!["project_ids contient 1 projets, attendu exactement 2.".to_string()]);
    let too_many = project_id_problems(&[20, 21, 22], &common::profile(), 2);
    assert!(too_many[0].contains("attendu exactement 2"), "{too_many:?}");
}
