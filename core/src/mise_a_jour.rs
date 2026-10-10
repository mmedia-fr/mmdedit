// SPDX-License-Identifier: GPL-3.0-or-later
//! Avis de nouvelle version (0.4.7) et mise à jour assistée (0.4.8), repris de
//! MMail (0.4.6 et 0.5.8) : mêmes règles, même comparaison, même contrôle.
//!
//! MMdedit demande à l'API de GitHub la dernière publication de son dépôt et,
//! si elle est plus récente, la signale. Si on le lui demande, il télécharge le
//! paquet de sa plateforme, en contrôle l'empreinte et le tient prêt.
//! L'installation — un installeur silencieux lancé à la fermeture, ou
//! l'AppImage remplacée — revient au programme (`cpp/installeur.h`) : elle
//! suit la fin du processus.
//!
//! Tout vient de la publication : son numéro par l'API, le paquet et le fichier
//! des empreintes (`SHA256SUMS-<version>.txt`) à leur adresse de publication,
//! que MMdedit compose lui-même. Un paquet dont l'empreinte ne correspond pas à
//! celle publiée n'est jamais gardé. Rien d'autre n'est envoyé que des
//! requêtes anonymes.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type MiseAJour = super::MiseAJourRust;

        /// Demande en arrière-plan la dernière version publiée. Issue :
        /// `disponible`, seulement si elle est plus récente que celle-ci ; rien
        /// sinon, ni en cas d'échec — l'avis ne dérange jamais.
        #[qinvokable]
        #[cxx_name = "verifier"]
        fn verifier(self: Pin<&mut MiseAJour>);

        /// Télécharge en arrière-plan le paquet `genre` (« setup », « appimage »)
        /// de `version` vers `destination`, empreinte contrôlée. Rien n'est
        /// téléchargé si le fichier y est déjà, intact. Issues : `progression`,
        /// puis `prete` ou `echec`. Sans effet pendant un autre téléchargement.
        #[qinvokable]
        #[cxx_name = "telecharger"]
        fn telecharger(self: Pin<&mut MiseAJour>, version: &QString, genre: &QString, destination: &QString);

        /// Efface de `dossier` les paquets et les téléchargements interrompus
        /// de MMdedit, sauf le fichier `garder` (chemin complet, ou vide). Pour
        /// une AppImage, dont le dossier est celui de l'utilisateur, seuls les
        /// téléchargements interrompus — fichiers cachés écrits par MMdedit.
        #[qinvokable]
        #[cxx_name = "nettoyer"]
        fn nettoyer(self: &MiseAJour, genre: &QString, dossier: &QString, garder: &QString);

        /// Vrai si `fichier` est l'installeur, déjà téléchargé, d'une `version`
        /// plus récente que celle-ci : de quoi l'installer à la fermeture d'un
        /// autre MMdedit que celui qui l'a téléchargé.
        #[qinvokable]
        #[cxx_name = "aInstaller"]
        fn a_installer(self: &MiseAJour, version: &QString, fichier: &QString) -> bool;

        /// Version de ce programme, telle que la compare `verifier`.
        #[qinvokable]
        #[cxx_name = "versionActuelle"]
        fn version_actuelle(self: &MiseAJour) -> QString;
    }

    impl cxx_qt::Threading for MiseAJour {}

    unsafe extern "RustQt" {
        /// Une version plus récente est publiée : son numéro et sa page.
        #[qsignal]
        #[cxx_name = "disponible"]
        fn disponible(self: Pin<&mut MiseAJour>, version: &QString, page: &QString);

        /// Avancement du téléchargement, en pour cent.
        #[qsignal]
        #[cxx_name = "progression"]
        fn progression(self: Pin<&mut MiseAJour>, version: &QString, pourcent: i32);

        /// Le paquet est là, intact.
        #[qsignal]
        #[cxx_name = "prete"]
        fn prete(self: Pin<&mut MiseAJour>, version: &QString, fichier: &QString);

        #[qsignal]
        #[cxx_name = "echec"]
        fn echec(self: Pin<&mut MiseAJour>, version: &QString, message: &QString);
    }
}

use core::pin::Pin;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use aws_lc_rs::digest;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use crate::http;

/// Dernière publication du dépôt : ni préversion ni brouillon.
const PUBLICATIONS: &str = "https://api.github.com/repos/mmedia-fr/mmdedit/releases/latest";

/// Seules pages que MMdedit accepte d'ouvrir : celles de son dépôt.
const DEPOT: &str = "https://github.com/mmedia-fr/mmdedit/";

/// Adresse de publication des fichiers d'une version.
const TELECHARGEMENTS: &str = "https://github.com/mmedia-fr/mmdedit/releases/download";

/// Taille au-delà de laquelle un paquet est refusé : l'AppImage pèse 45 Mo.
const TAILLE_MAX_PAQUET: u64 = 400 * 1024 * 1024;

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

/// Numéro lu chiffre par chiffre : « 0.4.8 » → [0, 4, 8]. Une préversion
/// (« 0.4.8-essai ») ne se lit pas.
fn numeros(version: &str) -> Option<Vec<u32>> {
    let v = version.trim_start_matches('v');
    if v.is_empty() {
        return None;
    }
    v.split('.').map(|n| n.parse().ok()).collect()
}

/// Vrai si `publiee` (« 0.4.8 ») est plus récente que `actuelle`. Un numéro
/// qui ne se lit pas — préversion « 0.4.8-essai » comprise — n'est jamais
/// annoncé.
fn plus_recente(publiee: &str, actuelle: &str) -> bool {
    matches!((numeros(publiee), numeros(actuelle)), (Some(a), Some(b)) if a > b)
}

/// Version de ce programme. `MMDEDIT_VERSION_ESSAI` la remplace pour éprouver
/// la mise à jour sur une version déjà publiée : l'essai se croit plus ancien,
/// et installe la dernière publication — rien d'autre ne peut l'être.
fn version_actuelle() -> String {
    std::env::var("MMDEDIT_VERSION_ESSAI")
        .ok()
        .filter(|v| numeros(v).is_some())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

/// Nom du paquet `genre` d'une version, tel que le publie l'intégration
/// continue ; `None` pour un genre inconnu ou un numéro illisible.
fn nom_du_paquet(version: &str, genre: &str) -> Option<String> {
    numeros(version)?;
    match genre {
        "setup" => Some(format!("MMdedit-{version}-setup.exe")),
        "appimage" => Some(format!("MMdedit-{version}-x86_64.AppImage")),
        _ => None,
    }
}

/// Empreinte publiée pour `nom` dans un fichier au format de `sha256sum` :
/// « <64 chiffres hexadécimaux>  <nom> », en minuscules.
fn empreinte_publiee(sommes: &str, nom: &str) -> Option<String> {
    sommes.lines().find_map(|ligne| {
        let (empreinte, fichier) = ligne.trim().split_once(char::is_whitespace)?;
        let fichier = fichier.trim_start().trim_start_matches('*');
        let valide = empreinte.len() == 64 && empreinte.chars().all(|c| c.is_ascii_hexdigit());
        (valide && fichier == nom).then(|| empreinte.to_ascii_lowercase())
    })
}

fn hexadecimal(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// Empreinte SHA-256 d'un fichier ; `None` s'il ne se lit pas.
fn empreinte_du_fichier(chemin: &Path) -> Option<String> {
    let mut fichier = fs::File::open(chemin).ok()?;
    let mut contexte = digest::Context::new(&digest::SHA256);
    let mut tampon = vec![0u8; 256 * 1024];
    loop {
        match fichier.read(&mut tampon).ok()? {
            0 => break,
            n => contexte.update(&tampon[..n]),
        }
    }
    Some(hexadecimal(contexte.finish().as_ref()))
}

/// Téléchargement interrompu : un fichier caché à côté de la destination, pour
/// que le remplacement final soit un simple renommage, d'un seul coup.
fn chemin_partiel(destination: &Path, nom: &str) -> PathBuf {
    destination.with_file_name(format!(".{nom}.part"))
}

/// Télécharge le paquet `genre` de `version` vers `destination` et en contrôle
/// l'empreinte. Rend un message lisible en cas d'échec.
fn telecharger_paquet(
    version: &str,
    genre: &str,
    destination: &Path,
    progression: &mut dyn FnMut(i32),
) -> Result<(), String> {
    let nom = nom_du_paquet(version, genre).ok_or_else(|| format!("paquet « {genre} » inconnu"))?;
    let sommes = http::lire(&format!("{TELECHARGEMENTS}/v{version}/SHA256SUMS-{version}.txt"), "text/plain")
        .map_err(|e| format!("empreintes de la publication illisibles — {e}"))?;
    if sommes.statut != 200 {
        return Err(format!("empreintes de la publication introuvables (statut {})", sommes.statut));
    }
    let attendue = empreinte_publiee(&String::from_utf8_lossy(&sommes.corps), &nom)
        .ok_or_else(|| format!("la publication ne donne pas l'empreinte de {nom}"))?;
    if empreinte_du_fichier(destination).as_deref() == Some(attendue.as_str()) {
        progression(100);
        return Ok(());
    }
    if let Some(dossier) = destination.parent() {
        fs::create_dir_all(dossier).map_err(|e| format!("{} : {e}", dossier.display()))?;
    }
    let partiel = chemin_partiel(destination, &nom);
    let resultat = (|| {
        let mut fichier = fs::File::create(&partiel).map_err(|e| format!("{} : {e}", partiel.display()))?;
        let mut contexte = digest::Context::new(&digest::SHA256);
        let mut dernier = -1;
        http::telecharger(
            &format!("{TELECHARGEMENTS}/v{version}/{nom}"),
            TAILLE_MAX_PAQUET,
            &mut |morceau| {
                contexte.update(morceau);
                fichier.write_all(morceau)?;
                Ok(())
            },
            &mut |recus, total| {
                let pourcent = (recus * 100 / total.max(1)) as i32;
                if pourcent != dernier {
                    dernier = pourcent;
                    progression(pourcent);
                }
            },
        )
        .map_err(|e| e.to_string())?;
        fichier.sync_all().map_err(|e| e.to_string())?;
        if hexadecimal(contexte.finish().as_ref()) != attendue {
            return Err(format!("{nom} reçu ne correspond pas à l'empreinte publiée"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&partiel, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        }
        fs::rename(&partiel, destination).map_err(|e| format!("{} : {e}", destination.display()))
    })();
    if resultat.is_err() {
        let _ = fs::remove_file(&partiel);
    }
    resultat
}

/// Version d'un installeur d'après son nom : « …/MMdedit-0.4.8-setup.exe » →
/// « 0.4.8 ».
fn version_du_paquet(chemin: &Path) -> Option<String> {
    let nom = chemin.file_name()?.to_str()?;
    let version = nom.strip_prefix("MMdedit-")?.strip_suffix("-setup.exe")?;
    numeros(version).map(|_| version.to_string())
}

/// Fichiers de `dossier` que `nettoyer` efface.
fn a_effacer(genre: &str, nom: &str) -> bool {
    let partiel = nom.starts_with(".MMdedit-") && nom.ends_with(".part");
    let paquet = genre == "setup" && nom.starts_with("MMdedit-") && nom.ends_with("-setup.exe");
    partiel || paquet
}

#[derive(Default)]
pub struct MiseAJourRust {
    en_cours: Arc<AtomicBool>,
}

impl qobject::MiseAJour {
    pub fn verifier(self: Pin<&mut Self>) {
        let fil = self.qt_thread();
        let _ = thread::Builder::new().name("mmdedit-version".into()).spawn(move || {
            let Some((version, page)) = derniere_version() else { return };
            if plus_recente(&version, &version_actuelle()) {
                let _ = fil.queue(move |objet: Pin<&mut qobject::MiseAJour>| {
                    objet.disponible(&QString::from(&version), &QString::from(&page));
                });
            }
        });
    }

    pub fn telecharger(self: Pin<&mut Self>, version: &QString, genre: &QString, destination: &QString) {
        let en_cours = self.rust().en_cours.clone();
        if en_cours.swap(true, Ordering::SeqCst) {
            return;
        }
        let (version, genre) = (version.to_string(), genre.to_string());
        let destination = PathBuf::from(destination.to_string());
        let fil = self.qt_thread();
        let lance = thread::Builder::new().name("mmdedit-mise-a-jour".into()).spawn(move || {
            let progres = fil.clone();
            let v = version.clone();
            let resultat = telecharger_paquet(&version, &genre, &destination, &mut |pourcent| {
                let v = v.clone();
                let _ = progres.queue(move |objet: Pin<&mut qobject::MiseAJour>| {
                    objet.progression(&QString::from(&v), pourcent);
                });
            });
            en_cours.store(false, Ordering::SeqCst);
            let fichier = destination.to_string_lossy().into_owned();
            let _ = fil.queue(move |objet: Pin<&mut qobject::MiseAJour>| match resultat {
                Ok(()) => objet.prete(&QString::from(&version), &QString::from(&fichier)),
                Err(message) => objet.echec(&QString::from(&version), &QString::from(&message)),
            });
        });
        if lance.is_err() {
            self.rust().en_cours.store(false, Ordering::SeqCst);
        }
    }

    pub fn nettoyer(&self, genre: &QString, dossier: &QString, garder: &QString) {
        let genre = genre.to_string();
        // Un paquet n'est gardé que s'il reste à installer.
        let garder = PathBuf::from(garder.to_string());
        let garder = version_du_paquet(&garder)
            .filter(|v| plus_recente(v, &version_actuelle()))
            .map(|_| garder)
            .unwrap_or_default();
        let Ok(entrees) = fs::read_dir(dossier.to_string()) else { return };
        for entree in entrees.flatten() {
            let chemin = entree.path();
            let nom = entree.file_name().to_string_lossy().into_owned();
            if chemin != garder && a_effacer(&genre, &nom) && chemin.is_file() {
                let _ = fs::remove_file(&chemin);
            }
        }
    }

    pub fn a_installer(&self, version: &QString, fichier: &QString) -> bool {
        a_installer(&version.to_string(), Path::new(&fichier.to_string()))
    }

    pub fn version_actuelle(&self) -> QString {
        QString::from(&version_actuelle())
    }
}

fn a_installer(version: &str, fichier: &Path) -> bool {
    version_du_paquet(fichier).as_deref() == Some(version)
        && plus_recente(version, &version_actuelle())
        && fichier.is_file()
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
        assert!(numeros(&version).is_some(), "{version}");
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

    #[test]
    fn noms_des_paquets() {
        assert_eq!(nom_du_paquet("0.4.8", "setup").as_deref(), Some("MMdedit-0.4.8-setup.exe"));
        assert_eq!(nom_du_paquet("0.4.8", "appimage").as_deref(), Some("MMdedit-0.4.8-x86_64.AppImage"));
        assert_eq!(nom_du_paquet("0.4.8", "apk"), None);
        // Le numéro entre dans une adresse et un nom de fichier : rien d'autre
        // que des chiffres et des points.
        assert_eq!(nom_du_paquet("0.4.8/../x", "setup"), None);
        assert_eq!(nom_du_paquet("", "setup"), None);
    }

    /// Le fichier publié avec la 0.4.7.
    const SOMMES: &str = "3afa61bfac0c7b3f1e160a627e3499138b623940421c091f294af2db4f1696f2  MMdedit-0.4.7-setup.exe
ea640a11fa866dc09b031ae646460e8cecf3cae40696c7003ae9813052a5ddcc  MMdedit-0.4.7-x86_64.AppImage
e76dff2d4802d4315b86ca43defd358682b81f8cc6b01579852a2ea5685e734c  MMdedit-0.4.7-android-arm64.apk
d79c0c97922473275318a5d7252f803a77edaa44e848614d6e109307446b06d3  MMdedit-0.4.7-sources.tar.gz
";

    #[test]
    fn empreintes_publiees() {
        assert_eq!(
            empreinte_publiee(SOMMES, "MMdedit-0.4.7-setup.exe").as_deref(),
            Some("3afa61bfac0c7b3f1e160a627e3499138b623940421c091f294af2db4f1696f2")
        );
        assert_eq!(
            empreinte_publiee(SOMMES, "MMdedit-0.4.7-x86_64.AppImage").as_deref(),
            Some("ea640a11fa866dc09b031ae646460e8cecf3cae40696c7003ae9813052a5ddcc")
        );
        assert_eq!(empreinte_publiee(SOMMES, "MMdedit-0.4.8-setup.exe"), None);
        assert_eq!(empreinte_publiee(SOMMES, "setup.exe"), None, "nom entier");
        // Mode binaire de sha256sum (« *nom ») et majuscules.
        let binaire = "ABCDEF0123456789abcdef0123456789ABCDEF0123456789abcdef0123456789 *MMdedit-1.0.0-setup.exe";
        assert_eq!(
            empreinte_publiee(binaire, "MMdedit-1.0.0-setup.exe").as_deref(),
            Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
        );
        assert_eq!(empreinte_publiee("abc  MMdedit-1.0.0-setup.exe", "MMdedit-1.0.0-setup.exe"), None);
    }

    #[test]
    fn empreinte_d_un_fichier() {
        let dossier = std::env::temp_dir().join(format!("mmdedit-maj-{}", std::process::id()));
        fs::create_dir_all(&dossier).unwrap();
        let chemin = dossier.join("abc");
        fs::write(&chemin, b"abc").unwrap();
        assert_eq!(
            empreinte_du_fichier(&chemin).as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        assert_eq!(empreinte_du_fichier(&dossier.join("absent")), None);
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn fichiers_effaces_au_nettoyage() {
        assert!(a_effacer("setup", "MMdedit-0.4.8-setup.exe"));
        assert!(a_effacer("setup", ".MMdedit-0.4.8-setup.exe.part"));
        assert!(a_effacer("appimage", ".MMdedit-0.4.8-x86_64.AppImage.part"));
        // Le dossier d'une AppImage est celui de l'utilisateur : ses fichiers à
        // lui n'y sont jamais touchés, installeurs Windows compris.
        assert!(!a_effacer("appimage", "MMdedit-0.4.8-setup.exe"));
        assert!(!a_effacer("appimage", "MMdedit-0.4.7-x86_64.AppImage"));
        assert!(!a_effacer("setup", "notes.md"));
        assert_eq!(version_du_paquet(Path::new("/x/MMdedit-0.4.8-setup.exe")).as_deref(), Some("0.4.8"));
        assert_eq!(version_du_paquet(Path::new("/x/MMdedit-0.4.8-x86_64.AppImage")), None);
        assert_eq!(version_du_paquet(Path::new("")), None);
    }

    #[test]
    fn installeur_a_reprendre() {
        let dossier = std::env::temp_dir().join(format!("mmdedit-maj-reprise-{}", std::process::id()));
        fs::create_dir_all(&dossier).unwrap();
        let paquet = dossier.join("MMdedit-99.0.0-setup.exe");
        assert!(!a_installer("99.0.0", &paquet), "absent");
        fs::write(&paquet, b"x").unwrap();
        assert!(a_installer("99.0.0", &paquet));
        assert!(!a_installer("99.0.1", &paquet), "autre version que le fichier");
        let ancien = dossier.join("MMdedit-0.0.1-setup.exe");
        fs::write(&ancien, b"x").unwrap();
        assert!(!a_installer("0.0.1", &ancien), "plus ancien que celui-ci");
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    #[ignore = "télécharge le fichier des empreintes et le paquet Windows de la 0.4.7 sur GitHub"]
    fn paquet_publie_telecharge_et_controle() {
        let dossier = std::env::temp_dir().join(format!("mmdedit-maj-reel-{}", std::process::id()));
        let destination = dossier.join("MMdedit-0.4.7-setup.exe");
        let attendue = "3afa61bfac0c7b3f1e160a627e3499138b623940421c091f294af2db4f1696f2";
        let mut etapes = Vec::new();
        telecharger_paquet("0.4.7", "setup", &destination, &mut |p| etapes.push(p)).unwrap();
        assert_eq!(etapes.last(), Some(&100));
        assert_eq!(empreinte_du_fichier(&destination).as_deref(), Some(attendue));
        assert!(!chemin_partiel(&destination, "MMdedit-0.4.7-setup.exe").exists());
        // Déjà là, intact : rien n'est téléchargé de nouveau.
        let mut encore = Vec::new();
        telecharger_paquet("0.4.7", "setup", &destination, &mut |p| encore.push(p)).unwrap();
        assert_eq!(encore, vec![100]);
        // Altéré : remplacé par le paquet publié.
        fs::write(&destination, b"altere").unwrap();
        telecharger_paquet("0.4.7", "setup", &destination, &mut |_| {}).unwrap();
        assert_eq!(empreinte_du_fichier(&destination).as_deref(), Some(attendue));
        // Version sans publication : échec lisible, rien d'écrit.
        let absente = dossier.join("MMdedit-9.9.9-setup.exe");
        let erreur = telecharger_paquet("9.9.9", "setup", &absente, &mut |_| {}).unwrap_err();
        println!("version absente : {erreur}");
        assert!(!absente.exists());
        fs::remove_dir_all(&dossier).unwrap();
    }
}
