//! Formules interdites dans la prose générée : justifications plaquées
//! après un fait (« directement transférable à… », « ce qui correspond
//! à… ») et formules d'annonce relevées dans les documents générés
//! (diagnostic du pipeline, observations 5 et 7).
//!
//! Liste volontairement courte, à enrichir à partir des observations. Les
//! formules sont comparées en minuscules ; une formule présente dans la
//! prose de la référence de l'utilisateur reste autorisée (c'est son style).

/// Formules interdites, en français.
pub const FORBIDDEN_PHRASES_FR: &[&str] = &[
    "directement transférable",
    "directement réutilisable",
    "directement aligné",
    "directement utile",
    "ce qui correspond",
    "correspond précisément",
    "maîtrise confirmée",
    "maîtrises confirmées",
    "première familiarité",
    "point d'entrée",
    "m'a immédiatement interpellé",
    "opportunité idéale",
    "démontrant une maîtrise",
    "témoignant d'une maîtrise",
    "applicable à la",
];

/// Formules interdites, en anglais.
pub const FORBIDDEN_PHRASES_EN: &[&str] = &[
    "directly transferable",
    "directly applicable",
    "directly aligned",
    "aligns with",
    "which matches",
    "which aligns",
    "proven expertise",
    "ideal opportunity",
    "immediately caught my attention",
    "demonstrating strong",
    "showcasing my",
];
