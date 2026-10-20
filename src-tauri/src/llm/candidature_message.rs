//! Rédaction de messages de candidature via l'API Claude : mail de
//! candidature et message court (LinkedIn ou équivalent). Contrairement au
//! pipeline `llm::pipeline` (CV/lettre en Typst), ces messages sont du texte
//! brut, prêt à copier-coller — un seul appel Claude, sans structured output.
//! Quand un contact est lié à la candidature, le message lui est adressé
//! (« Bonjour Prénom Nom », sans civilité devinée).

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::models::{CandidatureDetail, ContactView, ProfileDto};
use crate::llm::target::ClaudeTarget;

/// Règles communes aux deux rédactions, rappelées dans chaque prompt système
/// pour garantir l'honnêteté du contenu généré et éviter le remplissage IA
/// habituel (formules creuses, markdown, placeholders).
const RULES_COMMUNES: &str = "Règles strictes :\n\
- Tu ne peux que reformuler ce qui existe dans le profil, JAMAIS inventer une compétence, une expérience ou un projet\n\
- Pas de placeholder du type [Nom] ou [Entreprise] : utilise les informations fournies\n\
- Pas de formatage markdown (pas de **gras**, pas de listes à puces) : uniquement du texte brut\n\
- Ne signe qu'avec le prénom et le nom du candidat, sans coordonnées supplémentaires en signature";

/// Formule d'appel quand un contact destinataire est connu : son nom tel
/// qu'écrit, jamais de civilité (Madame/Monsieur) devinée d'après le prénom.
const SALUTATION_CONTACT: &str = "formule d'appel \"Bonjour\" suivie du nom du contact destinataire \
     exactement tel qu'il est fourni (par exemple \"Bonjour Camille Martin,\"), ou son équivalent direct \
     dans la langue de l'offre si elle n'est pas en français ; n'ajoute jamais de civilité \
     (Madame, Monsieur, Mr, Ms...) et ne la déduis jamais du prénom";

/// Construit le message utilisateur commun aux deux rédactions : profil du
/// candidat, détails de la candidature visée et contact destinataire.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `candidature` - Candidature visée (poste, entreprise, offre...).
/// * `contact` - Contact destinataire, s'il est connu.
///
/// # Returns
///
/// Le message utilisateur, prêt à être envoyé à `call_claude`.
fn build_user_message(
    profile: &ProfileDto,
    candidature: &CandidatureDetail,
    contact: Option<&ContactView>,
) -> Result<String, AppError> {
    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let entreprise = candidature.entreprise.as_deref().unwrap_or("l'entreprise");
    let description_offre = candidature.description_offre.as_deref().unwrap_or("Non renseignée.");

    Ok(format!(
        "Poste visé : {}\nEntreprise : {}\nLocalisation : {}\nType de contrat : {}\n{}\
         Description de l'offre :\n{}\n\nProfil du candidat :\n{profile_json}",
        candidature.poste,
        entreprise,
        candidature.localisation.as_deref().unwrap_or("Non renseignée."),
        candidature.type_contrat.as_deref().unwrap_or("Non renseigné."),
        contact_line(contact),
        description_offre,
    ))
}

/// Ligne « Contact destinataire » du message utilisateur.
///
/// # Arguments
///
/// * `contact` - Contact destinataire, s'il est connu.
///
/// # Returns
///
/// La ligne terminée par un saut de ligne (nom, puis rôle entre
/// parenthèses s'il est renseigné), ou une chaîne vide sans contact.
fn contact_line(contact: Option<&ContactView>) -> String {
    match contact {
        Some(c) => match c.role.as_deref() {
            Some(role) => format!("Contact destinataire : {} ({role})\n", c.nom),
            None => format!("Contact destinataire : {}\n", c.nom),
        },
        None => String::new(),
    }
}

/// Rédige un mail de candidature adapté à l'offre, à partir du profil du
/// candidat et des détails de la candidature.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `candidature` - Candidature visée.
/// * `contact` - Contact destinataire (premier contact lié), s'il est connu.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Le texte complet du mail (objet inclus en première ligne, puis corps).
pub async fn generate_application_email(
    profile: &ProfileDto,
    candidature: &CandidatureDetail,
    contact: Option<&ContactView>,
    target: &ClaudeTarget,
) -> Result<String, AppError> {
    let salutation = match contact {
        Some(_) => SALUTATION_CONTACT,
        None => "formule d'appel (Madame, Monsieur)",
    };
    let system_prompt = format!(
        "Tu rédiges un mail de candidature spontanée ou en réponse à une offre, au nom du candidat.\n\n\
         {RULES_COMMUNES}\n\
         - Structure : une première ligne \"Objet : ...\", une ligne vide, puis le corps du mail\n\
         - Corps : {salutation}, accroche, 1-2 paragraphes reliant le profil à l'offre, \
           formule de politesse, signature\n\
         - Ton professionnel et concis (200-300 mots pour le corps)"
    );

    let user_message = build_user_message(profile, candidature, contact)?;

    let raw_response = call_claude("message de candidature", &system_prompt, &user_message, None, target).await?;
    Ok(raw_response.trim().to_string())
}

/// Rédige un message court de candidature, destiné à un envoi via une
/// messagerie de réseau professionnel (LinkedIn ou équivalent) plutôt qu'un
/// mail formel.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `candidature` - Candidature visée.
/// * `contact` - Contact destinataire (premier contact lié), s'il est connu.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Le texte du message, prêt à être copié dans une messagerie.
pub async fn generate_application_message(
    profile: &ProfileDto,
    candidature: &CandidatureDetail,
    contact: Option<&ContactView>,
    target: &ClaudeTarget,
) -> Result<String, AppError> {
    let opening = match contact {
        Some(_) => format!("- Pas d'objet ; commence par une {SALUTATION_CONTACT}, puis un message direct et personnel"),
        None => "- Pas d'objet, pas de formule d'appel type \"Madame, Monsieur\" : un message direct et personnel"
            .to_string(),
    };
    let system_prompt = format!(
        "Tu rédiges un message court de prise de contact au sujet d'une candidature, destiné à être \
         envoyé via une messagerie de réseau professionnel (type LinkedIn) plutôt que par mail.\n\n\
         {RULES_COMMUNES}\n\
         {opening}\n\
         - 3 à 5 phrases maximum : qui est le candidat, pourquoi ce poste/cette entreprise, appel à l'échange\n\
         - Ton chaleureux mais professionnel, adapté à un premier message de réseau social"
    );

    let user_message = build_user_message(profile, candidature, contact)?;

    let raw_response = call_claude("message de candidature", &system_prompt, &user_message, None, target).await?;
    Ok(raw_response.trim().to_string())
}
