//! Consignes de rédaction communes au CV et à la lettre, et mise en forme
//! des blocs de prompt partagés (langue, modèle de style, document de
//! référence, consignes, feedback).
//!
//! Les consignes sont écrites en phrases simples, sans tiret cadratin : le
//! style du prompt déteint sur celui de la sortie.

// Importation des modules internes
use crate::llm::writing_loop::WritingFeedback;
use crate::models::doc_lang::language_name;
use crate::models::ProseGuidance;

/// Consignes communes aux deux documents, avec leur ordre de priorité. Une
/// seule hiérarchie, écrite une fois : aucune autre consigne ne la contredit.
pub const COMMON_RULES: &str = "Ordre de priorité des consignes, de la plus forte à la plus faible :
1. la langue du document ;
2. le maximum de mots de chaque champ (budgets) ;
3. les emplacements et consignes de la référence ;
4. le style du modèle.
Quand deux consignes s'opposent, la plus forte l'emporte.

Style. Imite la voix du modèle de style : même registre, même personne grammaticale, même temps, même longueur de phrases, même façon d'ouvrir les éléments. Garde le rôle et l'ordre des éléments du modèle, sauf quand un emplacement [ADAPTABLE] demande de choisir ou d'ordonner autrement.
Contenu. N'emprunte au modèle ni ses faits ni ses chiffres : il ne sert qu'au style. Tous les faits viennent du profil ; un chiffre n'apparaît que s'il figure dans le profil. Un élément énonce un fait, il ne commente jamais son adéquation au poste (par exemple « directement transférable à » ou « ce qui correspond à »). Les mots-clés de l'offre ne sont repris que là où ils décrivent un fait réel du profil.
Forme. N'écris jamais « — » (tiret cadratin), « – » (tiret demi-cadratin), « --- » ni « -- » : à leur place, mets une virgule ou des parenthèses pour une incise, un deux-points pour une précision, un point pour séparer deux idées, un trait d'union simple (-) entre deux nombres. N'utilise ni gras Markdown ni backticks.";

/// Règles des emplacements déclarés par la référence.
const ZONE_RULES: &str = "Règles des emplacements.
[VARIABLE] : remplace le texte d'exemple de l'emplacement par les informations tirées de l'offre, en suivant sa consigne.
[ADAPTABLE] : rédige l'emplacement à partir du profil selon l'offre, en suivant sa consigne, sans rien inventer.
Un champ sans emplacement est rédigé à partir du profil et de l'offre.";

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
            "Langue du document : {target}. Tout ce que tu écris est en {target}. Le modèle de style est en {} : transpose sa voix en {target} (registre, personne, temps, longueur des phrases) au lieu de le traduire.",
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

/// Encadre le rendu complet de la référence avec ses valeurs d'exemple :
/// il situe chaque champ rédigé parmi les parties fixes du document.
///
/// # Arguments
///
/// * `text` - Texte rendu de la référence.
///
/// # Returns
///
/// Bloc délimité, `None` si le rendu est vide.
pub fn reference_document_block(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    Some(format!(
        "DOCUMENT DE RÉFÉRENCE COMPLET, tel qu'il s'affiche avec ses valeurs d'exemple. Ses parties fixes (titres, formules, phrases écrites dans le modèle) sont ajoutées à part autour des champs rédigés : ne les répète pas. Il situe chaque champ dans le document, il ne fournit aucun fait.\n<<<RÉFÉRENCE\n{text}\nRÉFÉRENCE>>>"
    ))
}

/// Bloc des consignes de la référence : consignes générales, puis
/// emplacements déclarés de chaque champ de prose, avec leurs règles. Leur
/// rang est le 3 de `COMMON_RULES`.
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
        "CONSIGNES DE LA RÉFÉRENCE (priorité 3 de l'ordre des consignes).".to_string(),
    ];
    if !guidance.general.trim().is_empty() {
        lines.push(format!("Consignes générales :
{}", guidance.general.trim()));
    }
    if !guidance.zones.is_empty() {
        lines.push("Emplacements des champs de prose :".to_string());
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
