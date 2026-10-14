//! Consignes de rédaction communes au CV et à la lettre, et mise en forme
//! des blocs de prompt partagés (langue, modèle de style, feedback).
//!
//! Les consignes sont écrites en phrases simples, sans tiret cadratin : le
//! style du prompt déteint sur celui de la sortie.

// Importation des modules internes
use crate::llm::writing_loop::WritingFeedback;
use crate::models::ProseGuidance;

/// Consignes de style et d'honnêteté communes aux deux documents.
pub const COMMON_RULES: &str = "Consignes de rédaction.
Imite la voix du modèle de style : même registre, même personne grammaticale, même temps, même longueur de phrases, même façon d'ouvrir les bullets.
Suis sa structure : même nombre d'éléments, chaque élément jouant le même rôle que celui de même rang dans le modèle.
Ne reprends du modèle ni ses faits, ni ses chiffres, ni ses phrases entières. Il ne sert qu'au style.
Tous les faits viennent du profil. Un chiffre n'apparaît que s'il figure dans le profil.
Un bullet énonce un fait. Il ne commente jamais son adéquation au poste, par exemple avec « directement transférable à » ou « ce qui correspond à ».
Les mots-clés de l'offre ne sont repris que là où ils décrivent un fait réel du profil.
Les éléments « à mettre en avant » t'indiquent quoi dire : exprime-les avec tes propres mots à partir des faits du profil, ne les recopie pas.
Respecte les budgets comme des limites dures.
N'utilise ni tiret cadratin, ni tiret demi-cadratin, ni gras Markdown, ni backticks, sauf dans un texte que tu dois recopier.";

/// Règles des zones balisées de la référence.
const ZONE_RULES: &str = "Règles des zones.
[FIXE] : recopie le texte de la zone mot pour mot. Seules les informations qui diffèrent du profil (ville, école, entreprise, intitulé, chiffre) y sont remplacées par celles du profil.
[VARIABLE] : remplace le texte de la zone par les informations tirées de l'offre, en suivant sa consigne.
[ADAPTABLE] : rédige la zone à partir du profil selon l'offre, en suivant sa consigne, sans rien inventer.
Les zones d'un champ s'enchaînent dans le même ordre, sans rien ajouter entre elles. Un champ sans zone est rédigé à partir du profil, comme une zone [ADAPTABLE].";

/// Nom lisible d'une langue de document, pour les prompts.
fn language_name(lang: &str) -> &'static str {
    match lang {
        "en" => "anglais",
        _ => "français",
    }
}

/// Consigne de langue : langue cible, et transposition de la voix si le
/// modèle de style est dans une autre langue.
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
        format!("Rédige en {target}.")
    } else {
        format!(
            "Rédige en {target}. Le modèle de style est en {} : transpose sa voix en {target} (registre, personne, temps, longueur des phrases) au lieu de le traduire.",
            language_name(example_lang)
        )
    }
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
/// balisées de chaque champ de prose, avec les règles des zones. Elles
/// priment sur les consignes de style, y compris sur l'interdiction de
/// reprendre les phrases du modèle.
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
        "CONSIGNES DE LA RÉFÉRENCE. Elles priment sur les consignes de rédaction et sur le modèle de style :          une zone [FIXE] est recopiée même si le modèle de style demande de ne pas reprendre ses phrases."
            .to_string(),
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
