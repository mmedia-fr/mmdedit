// SPDX-License-Identifier: GPL-3.0-or-later
//! Lecture HTTPS minimale : la seule sortie réseau de MMdedit, pour l'avis de
//! nouvelle version et la mise à jour assistée (cf. `mise_a_jour`).
//!
//! Repris du client de MMail (`core/src/http.rs`), réduit au GET : même pile
//! TLS en pur Rust, mêmes autorités de certification, mêmes bornes.
//! Une bibliothèque HTTP apporterait sa propre pile TLS, qu'il faudrait régler
//! à nouveau pour Android ; Qt Network demanderait d'y embarquer OpenSSL.
//!
//! Seul HTTPS est accepté, redirections comprises.

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Attente d'une lecture ou d'une écriture.
const DELAI: Duration = Duration::from_secs(30);

/// Attente de l'établissement de la connexion, par adresse.
const DELAI_CONNEXION: Duration = Duration::from_secs(15);

/// Durée totale d'une requête : `DELAI` ne borne que chaque lecture.
const DUREE_MAX: Duration = Duration::from_secs(60);

/// Taille au-delà de laquelle une réponse est refusée : la description d'une
/// publication tient en quelques kilo-octets.
const TAILLE_MAX: usize = 256 * 1024;

/// Redirections suivies au plus.
const REDIRECTIONS_MAX: usize = 3;

pub type Resultat<T> = Result<T, Erreur>;

#[derive(Debug)]
pub enum Erreur {
    Reseau(String),
    Protocole(String),
    Refuse(String),
}

impl std::fmt::Display for Erreur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erreur::Reseau(m) => write!(f, "réseau : {m}"),
            Erreur::Protocole(m) => write!(f, "protocole : {m}"),
            Erreur::Refuse(m) => write!(f, "refusé : {m}"),
        }
    }
}

impl From<std::io::Error> for Erreur {
    fn from(e: std::io::Error) -> Self {
        Erreur::Reseau(e.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct Reponse {
    pub statut: u16,
    /// En-tête `Location`, s'il y en a un.
    pub redirection: Option<String>,
    pub corps: Vec<u8>,
}

/// Une adresse HTTPS découpée : ce qu'il faut pour se connecter et demander.
#[derive(Debug, PartialEq)]
pub struct Cible {
    pub hote: String,
    pub port: u16,
    /// Chemin et requête, tels qu'ils partent sur la ligne de demande.
    pub chemin: String,
}

/// Découpe une adresse `https://hote[:port]/chemin?requete`. Tout le reste —
/// `http://`, identifiants dans l'adresse, espaces — est refusé.
pub fn analyser_url(url: &str) -> Resultat<Cible> {
    let invalide = || Erreur::Protocole(format!("adresse invalide, « https:// » attendu : {url}"));
    let reste = url.trim().strip_prefix("https://").ok_or_else(invalide)?;
    let reste = reste.split('#').next().unwrap_or("");
    let (autorite, chemin) = match reste.find(['/', '?']) {
        Some(i) => (&reste[..i], &reste[i..]),
        None => (reste, "/"),
    };
    let chemin = if chemin.starts_with('?') { format!("/{chemin}") } else { chemin.to_string() };
    if autorite.contains('@') || chemin.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalide());
    }
    let (hote, port) = match autorite.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().map_err(|_| invalide())?),
        None => (autorite, 443),
    };
    let hote_valide = !hote.is_empty()
        && hote.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !hote_valide {
        return Err(invalide());
    }
    Ok(Cible { hote: hote.to_ascii_lowercase(), port, chemin })
}

/// Adresse désignée par un en-tête `Location`, relative ou absolue.
pub fn resoudre(depart: &str, location: &str) -> Resultat<String> {
    if location.starts_with("https://") {
        return Ok(location.to_string());
    }
    if location.starts_with('/') && !location.starts_with("//") {
        let cible = analyser_url(depart)?;
        let autorite = if cible.port == 443 {
            cible.hote
        } else {
            format!("{}:{}", cible.hote, cible.port)
        };
        return Ok(format!("https://{autorite}{location}"));
    }
    Err(Erreur::Protocole(format!("redirection refusée : {location}")))
}

/// Lit une adresse par GET, redirections HTTPS suivies, et rend la réponse.
pub fn lire(url: &str, accepte: &str) -> Resultat<Reponse> {
    let mut url = url.trim().to_string();
    for _ in 0..=REDIRECTIONS_MAX {
        let reponse = une_requete(&url, accepte)?;
        match (&reponse.redirection, reponse.statut) {
            (Some(location), 301 | 302 | 303 | 307 | 308) => url = resoudre(&url, location)?,
            _ => return Ok(reponse),
        }
    }
    Err(Erreur::Protocole("trop de redirections".into()))
}

/// Connexion TLS ouverte sur le serveur d'une adresse.
type Flux = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

/// Se connecte au serveur de `url` et lui envoie la demande ; la réponse reste
/// à lire sur le flux rendu.
fn demander(url: &str, accepte: &str) -> Resultat<Flux> {
    let cible = analyser_url(url)?;
    let nom = rustls::pki_types::ServerName::try_from(cible.hote.clone())
        .map_err(|_| Erreur::Reseau(format!("nom de serveur invalide : {}", cible.hote)))?;
    let connexion = rustls::ClientConnection::new(Arc::new(configuration_tls()?), nom)
        .map_err(|e| Erreur::Reseau(e.to_string()))?;
    let tcp = joindre(&cible.hote, cible.port)?;
    tcp.set_read_timeout(Some(DELAI))?;
    tcp.set_write_timeout(Some(DELAI))?;
    let mut flux = rustls::StreamOwned::new(connexion, tcp);

    let hote = if cible.port == 443 {
        cible.hote.clone()
    } else {
        format!("{}:{}", cible.hote, cible.port)
    };
    // L'API de GitHub refuse une requête sans User-Agent.
    let tete = format!(
        "GET {} HTTP/1.1\r\nHost: {hote}\r\nUser-Agent: MMdedit/{}\r\nAccept: {accepte}\r\n\
         Connection: close\r\n\r\n",
        cible.chemin,
        env!("CARGO_PKG_VERSION"),
    );
    flux.write_all(tete.as_bytes())?;
    flux.flush()?;
    Ok(flux)
}

fn une_requete(url: &str, accepte: &str) -> Resultat<Reponse> {
    let mut flux = demander(url, accepte)?;
    let mut brut = Vec::new();
    let mut tampon = [0u8; 8192];
    let echeance = Instant::now() + DUREE_MAX;
    loop {
        let reste = echeance.saturating_duration_since(Instant::now());
        if reste.is_zero() {
            return Err(Erreur::Reseau(format!("pas de réponse complète en {} s", DUREE_MAX.as_secs())));
        }
        flux.sock.set_read_timeout(Some(reste.min(DELAI)))?;
        match flux.read(&mut tampon) {
            Ok(0) => break,
            Ok(n) => {
                brut.extend_from_slice(&tampon[..n]);
                // Marge pour les en-têtes : c'est le corps que borne la limite.
                if brut.len() > TAILLE_MAX + 64 * 1024 {
                    return Err(Erreur::Refuse("réponse trop volumineuse".into()));
                }
                // Un serveur peut garder la connexion ouverte malgré
                // « Connection: close » : on n'attend pas sa fermeture quand la
                // réponse est complète.
                if reponse_complete(&brut) {
                    break;
                }
            }
            // Bien des serveurs ferment sans la notification TLS de fin : ce qui
            // a été lu reste valable, et le découpage HTTP dira s'il est complet.
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
    }
    analyser_reponse(&brut)
}

/// Taille de la tête d'une réponse au-delà de laquelle on renonce.
const TETE_MAX: usize = 64 * 1024;

/// Télécharge un fichier (GET, redirections vers HTTPS suivies) en le passant
/// morceau par morceau à `ecrire`, sans le garder en mémoire : une mise à jour
/// de MMdedit pèse des dizaines de mégaoctets. La longueur doit être annoncée —
/// c'est elle qui dit le fichier complet, et qui mesure la progression,
/// `progression(reçus, total)`. Chaque lecture est bornée par `DELAI`, non le
/// téléchargement entier : une connexion lente mais vivante va au bout. Rend
/// la taille reçue. Repris de MMail (0.5.8).
pub fn telecharger(
    url: &str,
    taille_max: u64,
    ecrire: &mut dyn FnMut(&[u8]) -> Resultat<()>,
    progression: &mut dyn FnMut(u64, u64),
) -> Resultat<u64> {
    let mut url = url.trim().to_string();
    for _ in 0..=REDIRECTIONS_MAX {
        let mut flux = demander(&url, "application/octet-stream")?;
        let mut brut = Vec::new();
        let mut tampon = [0u8; 64 * 1024];
        while !brut.windows(4).any(|w| w == b"\r\n\r\n") {
            if brut.len() > TETE_MAX {
                return Err(Erreur::Protocole("en-têtes HTTP démesurés".into()));
            }
            match flux.read(&mut tampon) {
                Ok(0) => return Err(Erreur::Reseau("réponse HTTP incomplète".into())),
                Ok(n) => brut.extend_from_slice(&tampon[..n]),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    return Err(Erreur::Reseau("réponse HTTP incomplète".into()))
                }
                Err(e) => return Err(e.into()),
            }
        }
        let (statut, entetes, debut) = analyser_tete(&brut)?;
        let entete = |nom: &str| entetes.iter().find(|(n, _)| n == nom).map(|(_, v)| v.as_str());
        if matches!(statut, 301 | 302 | 303 | 307 | 308) {
            let location = entete("location").ok_or_else(|| Erreur::Protocole("redirection sans adresse".into()))?;
            url = resoudre(&url, location)?;
            continue;
        }
        if statut != 200 {
            return Err(Erreur::Refuse(format!("le serveur a répondu {statut}")));
        }
        if entete("transfer-encoding").is_some_and(|v| !v.eq_ignore_ascii_case("identity")) {
            return Err(Erreur::Protocole("longueur du fichier non annoncée".into()));
        }
        let total: u64 = entete("content-length")
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Erreur::Protocole("longueur du fichier non annoncée".into()))?;
        if total > taille_max {
            return Err(Erreur::Refuse("fichier trop volumineux".into()));
        }
        let mut recus = 0u64;
        let mut morceau = &brut[debut..];
        loop {
            let utile = morceau.len().min((total - recus) as usize);
            if utile > 0 {
                ecrire(&morceau[..utile])?;
                recus += utile as u64;
                progression(recus, total);
            }
            if recus == total {
                return Ok(total);
            }
            let n = match flux.read(&mut tampon) {
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => 0,
                Err(e) => return Err(e.into()),
            };
            if n == 0 {
                return Err(Erreur::Reseau(format!("téléchargement interrompu : {recus} octets sur {total}")));
            }
            morceau = &tampon[..n];
        }
    }
    Err(Erreur::Protocole("trop de redirections".into()))
}

/// Configuration TLS du bureau : le vérificateur de la plateforme, qui lit le
/// magasin de certificats du système (Windows, Linux, macOS).
#[cfg(not(target_os = "android"))]
fn configuration_tls() -> Resultat<rustls::ClientConfig> {
    use rustls_platform_verifier::ConfigVerifierExt;
    rustls::ClientConfig::with_platform_verifier().map_err(|e| Erreur::Reseau(e.to_string()))
}

/// Configuration TLS d'Android : les autorités de Mozilla, compilées avec le
/// programme. Le vérificateur de plateforme y exigerait un composant Java et une
/// initialisation JNI que l'APK n'embarque pas — toute connexion échouerait.
#[cfg(target_os = "android")]
fn configuration_tls() -> Resultat<rustls::ClientConfig> {
    let racines = rustls::RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
    Ok(rustls::ClientConfig::builder().with_root_certificates(racines).with_no_client_auth())
}

/// Établit la connexion TCP, adresse par adresse, avec un délai borné.
fn joindre(hote: &str, port: u16) -> Resultat<TcpStream> {
    let adresses = (hote, port)
        .to_socket_addrs()
        .map_err(|e| Erreur::Reseau(format!("{hote} : {e}")))?;
    let mut derniere = Erreur::Reseau(format!("{hote} : aucune adresse"));
    for adresse in adresses {
        match TcpStream::connect_timeout(&adresse, DELAI_CONNEXION) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => derniere = Erreur::Reseau(format!("{adresse} : {e}")),
        }
    }
    Err(derniere)
}

/// Vrai si la réponse reçue est entière : corps de la longueur annoncée, ou
/// dernier bloc reçu. Sans l'une ni l'autre, seule la fermeture le dira.
fn reponse_complete(brut: &[u8]) -> bool {
    let Some(fin) = brut.windows(4).position(|w| w == b"\r\n\r\n") else {
        return false;
    };
    let tete = String::from_utf8_lossy(&brut[..fin]).to_ascii_lowercase();
    let entete = |nom: &str| {
        tete.split("\r\n")
            .find_map(|l| l.split_once(':').filter(|(n, _)| n.trim() == nom).map(|(_, v)| v.trim().to_string()))
    };
    let corps = &brut[fin + 4..];
    if entete("transfer-encoding").map(|v| v.contains("chunked")).unwrap_or(false) {
        return corps.ends_with(b"\r\n0\r\n\r\n") || corps == b"0\r\n\r\n";
    }
    match entete("content-length").and_then(|v| v.parse::<usize>().ok()) {
        Some(n) => corps.len() >= n,
        None => false,
    }
}

/// Statut et en-têtes (noms en minuscules) d'une réponse, dont `brut` contient
/// au moins la tête ; et la position où commence le corps.
fn analyser_tete(brut: &[u8]) -> Resultat<(u16, Vec<(String, String)>, usize)> {
    let fin = brut
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| Erreur::Protocole("réponse HTTP incomplète".into()))?;
    let tete = std::str::from_utf8(&brut[..fin])
        .map_err(|_| Erreur::Protocole("en-têtes HTTP illisibles".into()))?;
    let mut lignes = tete.split("\r\n");
    let premiere = lignes.next().unwrap_or("");
    let mut parties = premiere.splitn(3, ' ');
    if !parties.next().unwrap_or("").starts_with("HTTP/1.") {
        return Err(Erreur::Protocole(format!("réponse inattendue : {premiere}")));
    }
    let statut: u16 = parties
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Erreur::Protocole(format!("statut illisible : {premiere}")))?;
    let entetes: Vec<(String, String)> = lignes
        .filter_map(|l| l.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    Ok((statut, entetes, fin + 4))
}

/// Découpe une réponse HTTP/1.x complète : statut, redirection, corps.
pub fn analyser_reponse(brut: &[u8]) -> Resultat<Reponse> {
    let (statut, entetes, debut) = analyser_tete(brut)?;
    let entete = |nom: &str| entetes.iter().find(|(n, _)| n == nom).map(|(_, v)| v.as_str());

    let reste = &brut[debut..];
    let morcele = entete("transfer-encoding")
        .map(|v| v.to_ascii_lowercase().contains("chunked"))
        .unwrap_or(false);
    let corps = if morcele {
        rassembler(reste)?
    } else if let Some(longueur) = entete("content-length") {
        let n: usize = longueur
            .parse()
            .map_err(|_| Erreur::Protocole(format!("longueur illisible : {longueur}")))?;
        if n > TAILLE_MAX {
            return Err(Erreur::Refuse("réponse trop volumineuse".into()));
        }
        if reste.len() < n {
            return Err(Erreur::Reseau("réponse tronquée".into()));
        }
        reste[..n].to_vec()
    } else {
        reste.to_vec()
    };
    if corps.len() > TAILLE_MAX {
        return Err(Erreur::Refuse("réponse trop volumineuse".into()));
    }
    Ok(Reponse { statut, redirection: entete("location").map(str::to_string), corps })
}

/// Recompose un corps envoyé par blocs (`Transfer-Encoding: chunked`). Un
/// corps sans son bloc final de taille nulle est tenu pour tronqué.
fn rassembler(mut reste: &[u8]) -> Resultat<Vec<u8>> {
    let tronque = || Erreur::Reseau("réponse tronquée".into());
    let mut corps = Vec::new();
    loop {
        let fin_ligne = reste.windows(2).position(|w| w == b"\r\n").ok_or_else(tronque)?;
        let ligne = std::str::from_utf8(&reste[..fin_ligne])
            .map_err(|_| Erreur::Protocole("taille de bloc illisible".into()))?;
        let hexa = ligne.split(';').next().unwrap_or("").trim();
        let taille = usize::from_str_radix(hexa, 16)
            .map_err(|_| Erreur::Protocole(format!("taille de bloc illisible : {hexa}")))?;
        reste = &reste[fin_ligne + 2..];
        if taille == 0 {
            return Ok(corps);
        }
        if taille > TAILLE_MAX || corps.len() + taille > TAILLE_MAX {
            return Err(Erreur::Refuse("réponse trop volumineuse".into()));
        }
        if reste.len() < taille + 2 {
            return Err(tronque());
        }
        if &reste[taille..taille + 2] != b"\r\n" {
            return Err(Erreur::Protocole("bloc mal terminé".into()));
        }
        corps.extend_from_slice(&reste[..taille]);
        reste = &reste[taille + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fin_de_reponse_reconnue() {
        assert!(!reponse_complete(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n"));
        assert!(!reponse_complete(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nab"));
        assert!(reponse_complete(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nabcd"));
        assert!(!reponse_complete(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nabcd\r\n"));
        assert!(reponse_complete(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nabcd\r\n0\r\n\r\n"));
        assert!(!reponse_complete(b"HTTP/1.1 200 OK\r\n\r\nsans longueur"));
    }

    #[test]
    fn adresses_acceptees() {
        assert_eq!(
            analyser_url("https://Exemple.fr/chemin?a=1#ancre").unwrap(),
            Cible { hote: "exemple.fr".into(), port: 443, chemin: "/chemin?a=1".into() }
        );
        assert_eq!(
            analyser_url("https://exemple.fr:8443").unwrap(),
            Cible { hote: "exemple.fr".into(), port: 8443, chemin: "/".into() }
        );
        assert_eq!(analyser_url("https://exemple.fr?q=1").unwrap().chemin, "/?q=1");
    }

    #[test]
    fn adresses_refusees() {
        for url in [
            "http://exemple.fr/",
            "ftp://exemple.fr/",
            "https://",
            "https://nom:secret@exemple.fr/",
            "https://exemple.fr/un chemin",
            "https://exemple.fr:port/",
            "https://[::1]/",
            "exemple.fr",
        ] {
            assert!(analyser_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn redirections() {
        assert_eq!(resoudre("https://a.fr/x", "/y?z").unwrap(), "https://a.fr/y?z");
        assert_eq!(resoudre("https://a.fr:8443/x", "/y").unwrap(), "https://a.fr:8443/y");
        assert_eq!(resoudre("https://a.fr/x", "https://b.fr/").unwrap(), "https://b.fr/");
        assert!(resoudre("https://a.fr/x", "http://b.fr/").is_err());
        assert!(resoudre("https://a.fr/x", "//b.fr/").is_err());
    }

    #[test]
    fn reponse_a_longueur_annoncee() {
        let r = analyser_reponse(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nbonjour").unwrap();
        assert_eq!(r.statut, 200);
        assert_eq!(r.corps, b"bonjo");
        assert!(analyser_reponse(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\nbonjour").is_err());
    }

    #[test]
    fn reponse_par_blocs() {
        let r = analyser_reponse(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nbonj\r\n3;x=y\r\nour\r\n0\r\n\r\n",
        )
        .unwrap();
        assert_eq!(r.corps, b"bonjour");
        // Sans le bloc final : tronquée.
        assert!(analyser_reponse(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nbonj\r\n")
            .is_err());
    }

    #[test]
    fn tete_d_une_reponse() {
        let brut = b"HTTP/1.1 302 Found\r\nLocation: https://b.fr/x\r\nContent-Length: 12\r\n\r\ndebut";
        let (statut, entetes, debut) = analyser_tete(brut).unwrap();
        assert_eq!(statut, 302);
        assert!(entetes.contains(&("location".into(), "https://b.fr/x".into())));
        assert!(entetes.contains(&("content-length".into(), "12".into())));
        assert_eq!(&brut[debut..], b"debut");
        assert!(analyser_tete(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n").is_err());
    }

    #[test]
    fn reponse_redirigee_et_refus() {
        let r = analyser_reponse(b"HTTP/1.1 301 Moved\r\nLocation: /ailleurs\r\n\r\n").unwrap();
        assert_eq!(r.statut, 301);
        assert_eq!(r.redirection.as_deref(), Some("/ailleurs"));
        assert!(analyser_reponse(b"SSH-2.0-OpenSSH\r\n\r\n").is_err());
        let enorme = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", TAILLE_MAX + 1);
        assert!(analyser_reponse(enorme.as_bytes()).is_err());
    }
}
