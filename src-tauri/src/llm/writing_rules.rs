//! Consignes de rédaction communes au CV et à la lettre, et mise en forme
//! des blocs de prompt partagés (langue, modèle de style, feedback).
//!
//! Les consignes sont écrites en phrases simples, sans tiret cadratin : le
//! style du prompt déteint sur celui de la sortie.

// Importation des modules internes
use crate::llm::writing_loop::WritingFeedback;
use crate::models::ProseGuidance;

/// Consignes communes aux deux documents, avec leur ordre de priorité. Une
/// seule hiérarchie, écrite une fois : aucune autre consigne ne la contredit.
pub const COMMON_RULES: &str = "Ordre de priorité des consignes, de la plus forte à la plus faible :
1. la langue du document ;
2. les zones [FIXE] de la référence ;
3. le maximum de mots de chaque champ (budgets) ;
4. les autres zones et consignes de la référence ;
5. le style du modèle.
Quand deux consignes s'opposent, la plus forte l'emporte.

Style. Imite la voix du modèle de style : même registre, même personne grammaticale, même temps, même longueur de phrases, même façon d'ouvrir les éléments. Garde le rôle et l'ordre des éléments du modèle, sauf quand une zone [ADAPTABLE] demande de choisir ou d'ordonner autrement.
Contenu. Hors des zones [FIXE], n'emprunte au modèle ni ses faits ni ses chiffres : il ne sert qu'au style. Tous les faits viennent du profil ; un chiffre n'apparaît que s'il figure dans le profil. Un élément énonce un fait, il ne commente jamais son adéquation au poste (par exemple « directement transférable à » ou « ce qui correspond à »). Les mots-clés de l'offre ne sont repris que là où ils décrivent un fait réel du profil.
Forme. N'utilise ni tiret cadratin, ni tiret demi-cadratin, ni gras Markdown, ni backticks, sauf dans un texte à recopier.";

/// Règles des zones balisées de la référence.
const ZONE_RULES: &str = "Règles des zones.
[FIXE] : reprends le texte de la zone mot pour mot, dans la langue du document. Seules les informations qui diffèrent du profil (ville, école, entreprise, intitulé, chiffre) y sont remplacées par celles du profil.
[VARIABLE] : remplace le texte de la zone par les informations tirées de l'offre, en suivant sa consigne.
[ADAPTABLE] : rédige la zone à partir du profil selon l'offre, en suivant sa consigne, sans rien inventer.
Les zones d'un champ s'enchaînent dans le même ordre, sans rien ajouter entre elles. Un champ sans zone est rédigé à partir du profil et de l'offre.";

/// Nom lisible d'une langue de document, pour les prompts.
///
/// # Arguments
///
/// * `lang` - Code de langue (`"fr"` ou `"en"`).
///
/// # Returns
///
/// « anglais » ou « français ».
pub fn language_name(lang: &str) -> &'static str {
    match lang {
        "en" => "anglais",
        _ => "français",
    }
}

/// Consigne de langue, placée en tête du prompt système : langue cible, et
/// transposition de la voix si le modèle de style est dans une autre langue.
///
/// # Arguments
///
/// * `target_lang` - Langue du document à rédiger (celle de l'offre).
/// * `example_lang` - Langue du modèle de style.
///
/// # Returns
///
/// Consigne à insérer dans le prompt.
pub fn language_rule(target_lang: &str, example_lang: &str) -> String {
    let target = language_name(target_lang);
    if target_lang == example_lang {
        format!("Langue du document : {target}. Tout ce que tu écris est en {target}.")
    } else {
        format!(
            "Langue du document : {target}. Tout ce que tu écris est en {target}, y compris les zones [FIXE], données ici dans leur traduction fidèle. Le modèle de style est en {} : transpose sa voix en {target} (registre, personne, temps, longueur des phrases) au lieu de le traduire.",
            language_name(example_lang)
        )
    }
}

/// Rappel de langue, placé en fin de prompt et en tête du feedback des
/// relances.
///
/// # Arguments
///
/// * `target_lang` - Langue du document à rédiger.
///
/// # Returns
///
/// Rappel à insérer.
pub fn language_reminder(target_lang: &str) -> String {
    format!("Rappel : le document entier est rédigé en {}.", language_name(target_lang))
}

/// Encadre la prose de la référence comme modèle de style.
///
/// # Arguments
///
/// * `body` - Prose du modèle, déjà mise en forme.
///
/// # Returns
///
/// Bloc délimité, présenté comme modèle de forme uniquement.
pub fn style_model_block(body: &str) -> String {
    format!(
        "MODÈLE DE STYLE. Ce texte vient d'un autre document de la même personne. Imite sa forme, ne reprends rien de son contenu.\n<<<MODÈLE\n{body}\nMODÈLE>>>"
    )
}

/// Bloc des consignes de la référence : consignes générales, puis zones
/// balisées de chaque champ de prose, avec les règles des zones. Leur rang
/// est celui de `COMMON_RULES` (zones [FIXE] en 2, le reste en 4).
///
/// # Arguments
///
/// * `guidance` - Consignes de la référence, champs aux chemins du schéma de
///   sortie.
///
/// # Returns
///
/// Bloc à insérer dans le message, `None` si la référence n'a aucune consigne.
pub fn guidance_block(guidance: &ProseGuidance) -> Option<String> {
    if guidance.general.trim().is_empty() && guidance.zones.is_empty() {
        return None;
    }
    let mut lines = vec![
        "CONSIGNES DE LA RÉFÉRENCE (priorités 2 et 4 de l'ordre des consignes).".to_string(),
    ];
    if !guidance.general.trim().is_empty() {
        lines.push(format!("Consignes générales :
{}", guidance.general.trim()));
    }
    if !guidance.zones.is_empty() {
        lines.push("Zones des champs de prose :".to_string());
        for zone in &guidance.zones {
            let instruction =
                if zone.instruction.trim().is_empty() { String::new() } else { format!(" Consigne : {}", zone.instruction.trim()) };
            lines.push(format!("{} {} « {} »{instruction}", zone.field, zone.mode.tag(), zone.text));
        }
    }
    lines.push(ZONE_RULES.to_string());
    Some(lines.join("
"))
}

/// Ajoute au message utilisateur le retour de l'essai précédent.
///
/// # Arguments
///
/// * `message` - Message utilisateur de base.
/// * `feedback` - Retour de l'essai précédent, `None` au premier essai.
///
/// # Returns
///
/// Message complet.
pub fn with_feedback(message: String, feedback: Option<&WritingFeedback>) -> String {
    let Some(feedback) = feedback else {
        return message;
    };
    let problems = feedback.problems.join("\n");
    format!(
        "{message}\n\nTA RÉPONSE PRÉCÉDENTE ne respecte pas ces contraintes. Corrige-la en gardant ce qui est correct.\n\
         PROBLÈMES :\n{problems}\n\nRÉPONSE PRÉCÉDENTE :\n{}",
        feedback.previous_output
    )
}
