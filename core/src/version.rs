// SPDX-License-Identifier: GPL-3.0-or-later
//! Avis de nouvelle version : la dernière publication du dépôt, demandée à
//! l'API de GitHub. Repris de MMail (0.4.6), où il est en service depuis le
//! 2026-10-05 : mêmes règles, même comparaison.
//!
//! Rien d'autre n'est envoyé qu'une requête anonyme, et rien n'est installé :
//! l'interface montre un bandeau, et « Télécharger » ouvre la page de la
//! publication dans le navigateur.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type AvisVersion = super::AvisVersionRust;

        /// Demande en arrière-plan la dernière version publiée de MMdedit.
        /// Issue : `versionDisponible`, seulement si elle est plus récente que
        /// celle-ci ; rien sinon, ni en cas d'échec — l'avis ne dérange jamais.
        #[qinvokable]
        #[cxx_name = "verifierVersion"]
        fn verifier_version(self: Pin<&mut AvisVersion>);
    }

    impl cxx_qt::Threading for AvisVersion {}

    unsafe extern "RustQt" {
        /// Une version plus récente est publiée : son numéro et sa page.
        #[qsignal]
        #[cxx_name = "versionDisponible"]
        fn version_disponible(self: Pin<&mut AvisVersion>, version: &QString, adresse: &QString);
    }
}

use core::pin::Pin;
use std::thread;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use crate::http;

/// Dernière publication du dépôt : ni préversion ni brouillon.
const PUBLICATIONS: &str = "https://api.github.com/repos/mmedia-fr/mmdedit/releases/latest";

/// Seules pages que MMdedit accepte d'ouvrir : celles de son dépôt.
const DEPOT: &str = "https://github.com/mmedia-fr/mmdedit/";

/// Numéro et page de la dernière version publiée ; `None` si GitHub ne répond
/// pas, ou pas comme attendu.
///
/// Commodité de développement : `MMDEDIT_AVIS_ESSAI` donne le numéro à rendre
/// sans rien demander à GitHub, pour voir le bandeau sur une machine sans
/// écran (`--capture`). Rien n'est lu si la variable est absente.
fn derniere_version() -> Option<(String, String)> {
    if let Ok(essai) = std::env::var("MMDEDIT_AVIS_ESSAI") {
        return Some((essai.trim().to_string(), format!("{DEPOT}releases")));
    }
    let reponse = http::lire(PUBLICATIONS, "application/vnd.github+json").ok()?;
    if reponse.statut != 200 {
        return None;
    }
    lire_publication(&reponse.corps)
}

/// Numéro et page tirés de la description d'une publication par GitHub. Une
/// page ailleurs que sur le dépôt n'est pas retenue.
fn lire_publication(corps: &[u8]) -> Option<(String, String)> {
    let v: serde_json::Value = serde_json::from_slice(corps).ok()?;
    let version = v["tag_name"].as_str()?.trim_start_matches('v').to_string();
    let page = v["html_url"].as_str()?.to_string();
    page.starts_with(DEPOT).then_some((version, page))
}

/// Vrai si `publiee` (« 0.4.7 ») est plus récente que `actuelle`. Un numéro
/// qui ne se lit pas — préversion « 0.4.7-essai » comprise — n'est jamais
/// annoncé.
fn plus_recente(publiee: &str, actuelle: &str) -> bool {
    let lire = |v: &str| -> Option<Vec<u32>> {
        v.trim_start_matches('v').split('.').map(|n| n.parse().ok()).collect()
    };
    matches!((lire(publiee), lire(actuelle)), (Some(a), Some(b)) if a > b)
}

#[derive(Default)]
pub struct AvisVersionRust;

impl qobject::AvisVersion {
    pub fn verifier_version(self: Pin<&mut Self>) {
        let fil = self.qt_thread();
        let _ = thread::Builder::new().name("mmdedit-version".into()).spawn(move || {
            let Some((version, page)) = derniere_version() else { return };
            if plus_recente(&version, env!("CARGO_PKG_VERSION")) {
                let _ = fil.queue(move |objet: Pin<&mut qobject::AvisVersion>| {
                    objet.version_disponible(&QString::from(&version), &QString::from(&page));
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "interroge l'API de GitHub"]
    fn derniere_version_publiee() {
        std::env::remove_var("MMDEDIT_AVIS_ESSAI");
        let (version, page) = derniere_version().expect("réponse de GitHub");
        println!("dernière version publiée : {version} ({page})");
        assert!(version.split('.').all(|n| n.parse::<u32>().is_ok()), "{version}");
        assert!(page.starts_with("https://github.com/mmedia-fr/mmdedit/releases/tag/"));
    }

    #[test]
    fn comparaison_des_versions() {
        assert!(plus_recente("0.4.7", "0.4.6"));
        assert!(plus_recente("v0.4.10", "0.4.9"), "nombre, pas texte");
        assert!(plus_recente("0.5.0", "0.4.12"));
        assert!(!plus_recente("0.4.6", "0.4.6"));
        assert!(!plus_recente("0.4.5", "0.4.6"));
        assert!(!plus_recente("0.4.7-essai", "0.4.6"), "préversion jamais annoncée");
        assert!(!plus_recente("", "0.4.6"));
    }

    #[test]
    fn description_de_publication() {
        let corps = br#"{"tag_name":"v0.4.7","html_url":"https://github.com/mmedia-fr/mmdedit/releases/tag/v0.4.7","assets":[]}"#;
        assert_eq!(
            lire_publication(corps),
            Some(("0.4.7".into(), "https://github.com/mmedia-fr/mmdedit/releases/tag/v0.4.7".into()))
        );
        // Une page hors du dépôt n'est pas ouverte.
        let ailleurs = br#"{"tag_name":"v9.9.9","html_url":"https://exemple.fr/piege"}"#;
        assert_eq!(lire_publication(ailleurs), None);
        // Réponse d'erreur de l'API, ou document illisible.
        assert_eq!(lire_publication(br#"{"message":"Not Found"}"#), None);
        assert_eq!(lire_publication(b"<html>"), None);
    }
}
