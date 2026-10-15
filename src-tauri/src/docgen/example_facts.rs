//! Valeurs d'un document (`CvData`, `LmData`) qui ont un champ dans le
//! schéma, rangées en deux familles :
//! - les informations (identité, lieu, entrées du profil, destinataire),
//!   que le profil ou l'offre remplacent à la génération ;
//! - la prose (titre, résumé, objet, paragraphes), rédigée pour chaque
//!   offre, et les bullets des expériences et projets, remplacés par les
//!   descriptions du profil.
//!
//! Elles servent à vérifier qu'aucune n'est restée écrite en dur dans un
//! template, et à repérer les mots d'information qu'une zone `[FIXE]` peut
//! mettre à jour.

// Importation des modules internes
use crate::docgen::text_diff::{normalize_quotes, word_key, words};
use crate::models::{CvData, Identity, LmData};

/// Longueur minimale d'une valeur d'un seul mot pour être retenue : en
/// dessous (« B2 », « 1 »), elle se confond avec du texte ordinaire.
const MIN_SINGLE_WORD_CHARS: usize = 3;

/// Informations de l'identité (le permis est un booléen, affiché via un
/// libellé).
fn identity_values(identity: &Identity) -> Vec<&str> {
    vec![
        &identity.prenom,
        &identity.nom,
        &identity.email,
        &identity.telephone,
        &identity.ville,
        &identity.zip_code,
        &identity.linkedin,
        &identity.github,
        &identity.portfolio,
    ]
}

/// Informations d'un CV : identité, puis intitulés, organisations, lieux,
/// détails, compétences, langues et centres d'intérêt. Les périodes sont
/// exclues : faites de nombres, elles se confondent avec du texte daté
/// ordinaire.
///
/// # Arguments
///
/// * `data` - Données du CV.
///
/// # Returns
///
/// Valeurs retenues (non vides, assez longues), dans l'ordre du schéma.
pub fn cv_information_values(data: &CvData) -> Vec<String> {
    let mut values = identity_values(&data.identity);
    for experience in &data.experiences {
        values.extend([experience.poste.as_str(), &experience.entreprise, &experience.lieu]);
    }
    for project in &data.projects {
        values.extend([project.titre.as_str(), &project.pour_qui, &project.stack, &project.url]);
    }
    for formation in &data.formations {
        values.extend([formation.titre.as_str(), &formation.ecole]);
        values.extend(formation.details.iter().map(String::as_str));
    }
    values.extend(data.skills_groups.iter().flat_map(|g| g.skills.iter().map(String::as_str)));
    for langue in &data.langues {
        values.extend([langue.nom.as_str(), &langue.level]);
    }
    values.extend(data.interets.iter().map(String::as_str));
    retained(values)
}

/// Prose d'un CV : titre, résumé et bullets.
///
/// # Arguments
///
/// * `data` - Données du CV.
///
/// # Returns
///
/// Valeurs retenues, dans l'ordre du schéma.
pub fn cv_prose_values(data: &CvData) -> Vec<String> {
    let mut values = vec![data.identity.title.as_str(), &data.summary];
    for bullets in data.experiences.iter().map(|e| &e.bullets).chain(data.projects.iter().map(|p| &p.bullets)) {
        values.extend(bullets.iter().map(String::as_str));
    }
    retained(values)
}

/// Informations d'une lettre : identité et destinataire.
///
/// # Arguments
///
/// * `data` - Données de la lettre.
///
/// # Returns
///
/// Valeurs retenues, dans l'ordre du schéma.
pub fn lm_information_values(data: &LmData) -> Vec<String> {
    let mut values = identity_values(&data.identity);
    values.extend([data.recipient.company.as_str(), &data.recipient.location]);
    values.extend(data.recipient.extra_lines.iter().map(String::as_str));
    retained(values)
}

/// Prose d'une lettre : titre, objet et paragraphes.
///
/// # Arguments
///
/// * `data` - Données de la lettre.
///
/// # Returns
///
/// Valeurs retenues, dans l'ordre du schéma.
pub fn lm_prose_values(data: &LmData) -> Vec<String> {
    let mut values = vec![data.identity.title.as_str(), &data.subject];
    values.extend(data.paragraphs.iter().map(String::as_str));
    retained(values)
}

/// Clés de comparaison des mots d'un texte : sans ponctuation de bord, en
/// minuscules, guillemets normalisés.
///
/// # Arguments
///
/// * `text` - Texte quelconque.
///
/// # Returns
///
/// Clés, dans l'ordre du texte.
pub fn word_keys(text: &str) -> Vec<String> {
    words(&normalize_quotes(text)).into_iter().map(|w| word_key(w).to_lowercase()).collect()
}

/// Valeurs non vides, sans les valeurs d'un seul mot trop courtes.
fn retained(values: Vec<&str>) -> Vec<String> {
    values
        .into_iter()
        .map(str::trim)
        .filter(|value| {
            let keys = word_keys(value);
            match keys.as_slice() {
                [] => false,
                [single] => single.chars().count() >= MIN_SINGLE_WORD_CHARS,
                _ => true,
            }
        })
        .map(str::to_string)
        .collect()
}
