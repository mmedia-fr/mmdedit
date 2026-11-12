// SPDX-License-Identifier: GPL-3.0-or-later
//! Import bureautique : des documents zippés (.docx, .odt, .epub) vers le
//! Markdown, à l'ouverture — étape 6 de la feuille de route du 12/11.
//!
//! Fidélité limitée, annoncée comme telle : titres, gras, italique, listes
//! (les numérotées de Word et LibreOffice deviennent des puces ; celles des
//! livres reprennent leurs numéros), tableaux simples et liens ; le reste —
//! images, en-têtes et pieds de page, styles fins — est ignoré sans faire
//! échouer l'import. L'emphase se pose au plus près du texte : jamais sur un
//! paragraphe entier. Tout est pur Rust : la conversion tourne à l'identique
//! sur les trois cibles, Android compris.
//!
//! Ces fichiers sont des ZIP lus en mémoire, jamais réécrits — le noyau
//! refuse d'ailleurs toute cible bureautique à l'enregistrement.

use crate::texte;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::str::from_utf8;

/// Erreurs en français, affichées telles quelles à l'utilisateur.
type Resultat<T> = Result<T, String>;

/// Convertit un fichier bureautique déjà lu en octets, aiguillé par son
/// extension. Le chemin ne sert qu'à l'aiguillage, jamais à l'écriture.
pub fn vers_markdown(chemin: &str, octets: &[u8]) -> Resultat<String> {
    match texte::extension(chemin).as_str() {
        "docx" => docx(octets),
        "odt" => odt(octets),
        "epub" => epub(octets),
        _ => Err("Ce fichier bureautique n'est pas pris en charge.".to_string()),
    }
}

/// Nom local d'une balise ou d'un attribut : sans son préfixe d'espace de
/// noms — `w:p` comme `p`.
fn local(nom: &[u8]) -> &str {
    match nom.split(|&o| o == b':').last() {
        Some(partie) => from_utf8(partie).unwrap_or(""),
        None => "",
    }
}

/// Extrait le contenu d'un membre du ZIP, décompressé.
fn membre(octets: &[u8], nom: &str) -> Resultat<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(octets))
        .map_err(|_| "Lecture impossible : ce n'est pas une archive ZIP.".to_string())?;
    let mut fichier = archive
        .by_name(nom)
        .map_err(|_| format!("Le fichier attendu ({nom}) est introuvable dans l'archive."))?;
    let mut contenu = Vec::new();
    fichier
        .read_to_end(&mut contenu)
        .map_err(|e| format!("Lecture de {nom} impossible : {e}"))?;
    Ok(contenu)
}

/// Attribut d'une balise, décodé ; rien s'il est absent ou illisible.
fn attribut(balise: &BytesStart, nom: &str) -> Option<String> {
    balise
        .attributes()
        .flatten()
        .find(|a| local(a.key.as_ref()) == nom)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// Texte d'un événement, décodé de ses entités (`&amp;`…).
fn event_texte(contenu: &quick_xml::events::BytesText) -> String {
    contenu
        .unescape()
        .map(|t| t.replace('\r', ""))
        .unwrap_or_default()
}

/// Un groupe de lignes jointes par des lignes vides — un document tout court.
fn joindre(lignes: &[String]) -> String {
    lignes
        .iter()
        .filter(|l| !l.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n")
        .trim()
        .to_string()
}

/// Enferme un texte dans ses marqueurs d'emphase. Sans emphase, le texte
/// passe tel quel : ses espaces de tête et de queue SONT du document. Avec
/// emphase, les bords se taillent — « ** gras ** » n'est pas du Markdown.
fn enrober(texte: &str, gras: bool, italique: bool) -> String {
    if texte.is_empty() {
        return String::new();
    }
    if !gras && !italique {
        return texte.to_string();
    }
    let propre = texte.trim();
    if propre.is_empty() {
        return String::new();
    }
    let mut s = propre.to_string();
    if italique {
        s = format!("*{s}*");
    }
    if gras {
        s = format!("**{s}**");
    }
    s
}

/// Titre de niveau 1 à 6, raborné.
fn titre(niveau: u64, texte: &str) -> String {
    format!("{} {}", "#".repeat(niveau.clamp(1, 6) as usize), texte)
}


// -------------------------------------------------------------------- /docx/

/// Texte d'un run de Word, en retard : ses marqueurs ne se posent qu'après
/// lecture du `w:rPr` qui suit son ouverture.
#[derive(Default)]
struct Run {
    gras: bool,
    italique: bool,
    texte: String,
}

fn docx(octets: &[u8]) -> Resultat<String> {
    let xml = String::from_utf8_lossy(&membre(octets, "word/document.xml")?).to_string();
    let mut lecteur = Reader::from_str(&xml);
    let mut lignes: Vec<String> = Vec::new();
    let mut tampon = String::new(); // corps du paragraphe en cours
    let mut style: Option<String> = None;
    let mut item: Option<usize> = None; // niveau de liste, si paragraphes numérotés
    let mut run: Option<Run> = None;
    let mut cellule = String::new();
    let mut rang: Vec<String> = Vec::new();
    let mut bloc_table: Vec<String> = Vec::new();
    let mut rangs_poses = 0usize;
    let mut dans_table = 0usize;
    let mut buf = Vec::new();
    loop {
        let evenement = lecteur
            .read_event_into(&mut buf)
            .map_err(|e| format!("Document Word illisible : {e}"))?;
        match evenement {
            Event::Start(balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                match nom {
                    "tbl" => dans_table += 1,
                    "tr" if dans_table > 0 => rang.clear(),
                    "tc" => cellule.clear(),
                    "p" => {
                        tampon.clear();
                        style = None;
                        item = None;
                    }
                    "r" => run = Some(Run::default()),
                    "pStyle" => style = attribut(&balise, "val"),
                    "numPr" => item = Some(0),
                    "ilvl" => item = attribut(&balise, "val").and_then(|v| v.parse().ok()),
                    "b" => {
                        if let Some(r) = run.as_mut() {
                            r.gras = !coupe(&balise);
                        }
                    }
                    "i" => {
                        if let Some(r) = run.as_mut() {
                            r.italique = !coupe(&balise);
                        }
                    }
                    _ => {}
                }
                if nom == "br" {
                    tampon.push('\n');
                }
                if nom == "tab" {
                    tampon.push('\t');
                }
            }
            Event::Empty(balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                match nom {
                    "pStyle" => style = attribut(&balise, "val"),
                    "numPr" => item = Some(0),
                    "ilvl" => item = attribut(&balise, "val").and_then(|v| v.parse().ok()),
                    "b" => {
                        if let Some(r) = run.as_mut() {
                            r.gras = !coupe(&balise);
                        }
                    }
                    "i" => {
                        if let Some(r) = run.as_mut() {
                            r.italique = !coupe(&balise);
                        }
                    }
                    "br" => tampon.push('\n'),
                    "tab" => tampon.push('\t'),
                    _ => {}
                }
            }
            Event::Text(contenu) => {
                if let Some(r) = run.as_mut() {
                    r.texte.push_str(&event_texte(&contenu));
                }
            }
            Event::CData(contenu) => {
                let t = String::from_utf8_lossy(contenu.as_ref()).to_string();
                if let Some(r) = run.as_mut() {
                    r.texte.push_str(&t);
                }
            }
            Event::End(balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                match nom {
                    "r" => {
                        if let Some(r) = run.take() {
                            tampon.push_str(&enrober(&r.texte, r.gras, r.italique));
                        }
                    }
                    "p" => {
                        let corps = tampon.trim().to_string();
                        tampon.clear();
                        if dans_table > 0 {
                            if !cellule.is_empty() && !corps.is_empty() {
                                cellule.push(' ');
                            }
                            cellule.push_str(&corps);
                        } else {
                            let pose = match (&style, item) {
                                (Some(style), _) => {
                                    let s = style.to_lowercase();
                                    if let Some(n) = s
                                        .strip_prefix("heading")
                                        .and_then(|r| r.trim().parse::<u64>().ok())
                                    {
                                        Some(titre(n, &corps))
                                    } else if s == "title" {
                                        Some(titre(1, &corps))
                                    } else {
                                        None
                                    }
                                }
                                (None, Some(niveau)) if !corps.is_empty() => {
                                    Some(format!("{}- {}", "  ".repeat(niveau), corps))
                                }
                                _ => None,
                            };
                            match pose {
                                Some(texte) if !texte.trim_matches([' ', '#']).is_empty() => {
                                    lignes.push(texte)
                                }
                                _ => {
                                    if !corps.is_empty() {
                                        lignes.push(corps)
                                    }
                                }
                            }
                            item = None;
                        }
                    }
                    "tc" if dans_table > 0 => {
                        rang.push(cellule.trim().to_string());
                        cellule.clear();
                    }
                    "tr" if dans_table > 0 => {
                        if rangs_poses == 0 {
                            bloc_table.extend(poser_ligne_tableau(&rang));
                        } else if !rang.is_empty() {
                            bloc_table.push(format!("| {} |", rang.join(" | ")));
                        }
                        rangs_poses += 1;
                        rang.clear();
                    }
                    "tbl" => {
                        dans_table = dans_table.saturating_sub(1);
                        if dans_table == 0 {
                            lignes.push(bloc_table.join("\n"));
                            bloc_table.clear();
                            rangs_poses = 0;
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(joindre(&lignes))
}

/// Attribut `w:val` qui coupe un attribut binaire de Word (`b`, `i`).
fn coupe(balise: &BytesStart) -> bool {
    matches!(
        attribut(balise, "val").as_deref(),
        Some("false") | Some("none") | Some("0")
    )
}

/// Rangée de tableau en Markdown, ligne et séparateur — sans lui, une rangée
/// de pips ne ferait pas un tableau.
fn poser_ligne_tableau(rang: &[String]) -> Vec<String> {
    if rang.is_empty() {
        return Vec::new();
    }
    let sep: Vec<&str> = rang.iter().map(|_| "---").collect();
    vec![
        format!("| {} |", rang.join(" | ")),
        format!("| {} |", sep.join(" | ")),
    ]
}

// ------------------------------------------------------------- /machine odt/

/// Un étage ouvert du document : un paragraphe, un titre, une portion
/// emphase… Le texte lu va toujours à l'étage du dessus ; sa fermeture le
/// rend, enrobé, à celui de dessous.
enum Genre {
    Racine,
    Paragraphe,
    Titre(u64),
    Item(usize), // niveau d'imbrication des listes
    Liste(bool), // ordonnée — le numéro des items s'y tient
    Cellule,
    Rangée,
    Table,
    Portion, // emphase ou lien : son texte se rend à son parent
    Lien,
}

struct Etage {
    genre: Genre,
    gras: bool,
    italique: bool,
    texte: String,
    poses: Vec<String>, // lignes ou cellules déjà closes dessous
    href: Option<String>,
    compteur: usize, // numéro courant d'une liste ordonnée
}

impl Etage {
    fn nouveau(genre: Genre) -> Self {
        Self {
            genre,
            gras: false,
            italique: false,
            texte: String::new(),
            poses: Vec::new(),
            href: None,
            compteur: 0,
        }
    }
}

/// La machine commune : le lecteur des trois formats y pousse et y referme
/// ses étages ; tout y est pur texte, aucun état de Qt n'y paraît.
struct Machine {
    pile: Vec<Etage>,
}

impl Machine {
    fn nouvelle() -> Self {
        Self {
            pile: vec![Etage::nouveau(Genre::Racine)],
        }
    }

    fn haut(&mut self) -> &mut Etage {
        self.pile.last_mut().unwrap()
    }

    fn ouvrir(&mut self, genre: Genre) {
        self.pile.push(Etage::nouveau(genre));
    }

    fn texte(&mut self, t: &str) {
        self.haut().texte.push_str(t);
    }

    /// Referme l'étage du dessus et rend son texte dégraissé.
    fn fermer(&mut self) -> Etage {
        self.pile.pop().unwrap()
    }

    /// Fermeture d'une portion : le texte enrobé remonte au parent.
    fn fermer_portion(&mut self) {
        let portion = self.fermer();
        let corps = enrober(&portion.texte, portion.gras, portion.italique);
        self.haut().texte.push_str(&corps);
    }

    /// Fermeture d'un lien : le lien remonte au parent.
    fn fermer_lien(&mut self) {
        let lien = self.fermer();
        let href = lien.href.clone().unwrap_or_default();
        let corps = lien.texte.trim();
        let ancre = if corps.is_empty() {
            href.clone()
        } else {
            corps.to_string()
        };
        if href.is_empty() {
            self.haut().texte.push_str(&ancre);
        } else {
            self.haut().texte.push_str(&format!("[{ancre}]({href})"));
        }
    }

    /// Fermeture d'un paragraphe : posé en ligne, ou absorbé par l'item qui
    /// l'entoure s'il y en a un.
    fn fermer_paragraphe(&mut self) {
        let paragraphe = self.fermer();
        let corps = enrober(&paragraphe.texte, paragraphe.gras, paragraphe.italique)
            .trim()
            .to_string();
        if corps.is_empty() {
            return;
        }
        match self.haut().genre {
            Genre::Item(_) => {
                let haut = self.haut();
                if !haut.texte.is_empty() {
                    haut.texte.push(' ');
                }
                haut.texte.push_str(&corps);
            }
            _ => self.haut().poses.push(corps),
        }
    }

    /// Fermeture d'un titre.
    fn fermer_titre(&mut self) {
        let paragraphe = self.fermer();
        let corps = paragraphe.texte.trim();
        if !corps.is_empty() {
            if let Genre::Titre(niveau) = paragraphe.genre {
                self.haut().poses.push(titre(niveau, corps));
            }
        }
    }

    /// Fermeture d'un item : la ligne se pose sur la liste qui l'entoure —
    /// numérotée si cette liste l'est, à puce sinon.
    fn fermer_item(&mut self) {
        let item = self.fermer();
        let corps = enrober(&item.texte, item.gras, item.italique)
            .trim()
            .to_string();
        if corps.is_empty() {
            return;
        }
        let niveau = match item.genre {
            Genre::Item(niveau) => niveau,
            _ => 0,
        };
        let retrait = "  ".repeat(niveau);
        let haut = self.haut();
        let marque = match haut.genre {
            Genre::Liste(true) => {
                haut.compteur += 1;
                format!("{retrait}{}. ", haut.compteur)
            }
            _ => format!("{retrait}- "),
        };
        haut.poses.push(format!("{marque}{corps}"));
    }

    /// Fermeture d'une cellule : elle rejoint la rangée.
    fn fermer_cellule(&mut self) {
        let cellule = self.fermer();
        self.haut().poses.push(cellule.texte.trim().to_string());
    }

    /// Fermeture d'une rangée : ligne Markdown, séparateur après la premiere.
    fn fermer_rangée(&mut self) {
        let rangée = self.fermer();
        let haut = self.haut();
        if haut.compteur == 0 {
            haut.poses.extend(poser_ligne_tableau(&rangée.poses));
        } else if !rangée.poses.is_empty() {
            haut.poses.push(format!("| {} |", rangée.poses.join(" | ")));
        }
        haut.compteur += 1;
    }

    /// Fermeture d'une table : ses rangées jointes forment UN bloc — des
    /// lignes vides entre elles casseraient le tableau.
    fn fermer_table(&mut self) {
        let table = self.fermer();
        if !table.poses.is_empty() {
            self.haut().poses.push(table.poses.join("\n"));
        }
    }

    /// Fermeture d'une liste : ses items joints forment UN bloc — des lignes
    /// vides entre eux sépareraient la liste en plusieurs.
    fn fermer_liste(&mut self) {
        let liste = self.fermer();
        if !liste.poses.is_empty() {
            self.haut().poses.push(liste.poses.join("\n"));
        }
    }

    fn fin(&mut self) -> String {
        let racine = self.pile.pop().unwrap();
        joindre(&racine.poses)
    }
}

fn odt(octets: &[u8]) -> Resultat<String> {
    let xml = String::from_utf8_lossy(&membre(octets, "content.xml")?).to_string();
    let mut lecteur = Reader::from_str(&xml);
    let mut machine = Machine::nouvelle();
    // Les styles automatiques d'abord : ils disent si un `text:style-name`
    // porte du gras ou de l'italique.
    let mut styles: HashMap<String, (bool, bool)> = HashMap::new();
    let mut style_courant: Option<String> = None;
    let mut bits = (false, false);
    let mut dans_styles = false;
    let mut buf = Vec::new();
    loop {
        let evenement = lecteur
            .read_event_into(&mut buf)
            .map_err(|e| format!("Document LibreOffice illisible : {e}"))?;
        match evenement {
            Event::Start(balise) | Event::Empty(balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                match nom {
                    "automatic-styles" => dans_styles = true,
                    "style" if dans_styles => {
                        style_courant = attribut(&balise, "name");
                        bits = (false, false);
                    }
                    "text-properties" if dans_styles && style_courant.is_some() => {
                        if matches!(
                            attribut(&balise, "font-weight").as_deref(),
                            Some("bold") | Some("600") | Some("700")
                        ) {
                            bits.0 = true;
                        }
                        if attribut(&balise, "font-style").as_deref() == Some("italic") {
                            bits.1 = true;
                        }
                    }
                    _ => {}
                }
                if !dans_styles {
                    match nom {
                        "h" => {
                            let niveau = attribut(&balise, "outline-level")
                                .and_then(|v| v.parse::<u64>().ok())
                                .filter(|n| *n >= 1)
                                .unwrap_or(1);
                            machine.ouvrir(Genre::Titre(niveau));
                        }
                        "p" => machine.ouvrir(Genre::Paragraphe),
                        "span" => {
                            let mut étage = Etage::nouveau(Genre::Portion);
                            if let Some(nom_style) = attribut(&balise, "style-name") {
                                if let Some((gras, italique)) = styles.get(&nom_style).copied() {
                                    étage.gras = gras;
                                    étage.italique = italique;
                                }
                            }
                            machine.pile.push(étage);
                        }
                        "line-break" => machine.texte("\n"),
                        "tab" => machine.texte("\t"),
                        "s" => {
                            let n = attribut(&balise, "c")
                                .and_then(|v| v.parse::<usize>().ok())
                                .filter(|n| *n > 0)
                                .unwrap_or(1);
                            machine.texte(&" ".repeat(n));
                        }
                        "table" => machine.ouvrir(Genre::Table),
                        "table-row" => machine.ouvrir(Genre::Rangée),
                        "table-cell" | "covered-table-cell" => machine.ouvrir(Genre::Cellule),
                        "list" => machine.ouvrir(Genre::Liste(false)),
                        "list-item" => machine.ouvrir(Genre::Item(
                            machine
                                .pile
                                .iter()
                                .filter(|e| matches!(e.genre, Genre::Liste(_)))
                                .count()
                                .saturating_sub(1),
                        )),
                        _ => {}
                    }
                }
            }
            Event::Text(contenu) => machine.texte(&event_texte(&contenu)),
            Event::CData(contenu) => {
                machine.texte(&String::from_utf8_lossy(contenu.as_ref()));
            }
            Event::End(balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                match nom {
                    "automatic-styles" => dans_styles = false,
                    "style" => {
                        if let Some(nom_style) = style_courant.take() {
                            styles.insert(nom_style, bits);
                        }
                    }
                    "span" => machine.fermer_portion(),
                    "h" => machine.fermer_titre(),
                    "p" => machine.fermer_paragraphe(),
                    "list-item" => machine.fermer_item(),
                    "list" => machine.fermer_liste(),
                    "table-cell" | "covered-table-cell" => machine.fermer_cellule(),
                    "table-row" => machine.fermer_rangée(),
                    "table" => machine.fermer_table(),
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(machine.fin())
}

// ------------------------------------------------------------------- /epub/

fn html(octets: &[u8]) -> Resultat<String> {
    let xml = String::from_utf8_lossy(octets).to_string();
    let mut lecteur = Reader::from_str(&xml.trim_start_matches('\u{feff}'));
    // Les documents XHTML viennent sans déclaration d'espaces de noms ou
    // av documentés pieuvre : quick-xml lit le document tel quel, les
    // balises inconnues traversent sans effet.
    let mut machine = Machine::nouvelle();
    let mut saute = 0usize; // style, script, svg : le contenu s'ignore
    let mut buf = Vec::new();
    loop {
        let evenement = lecteur
            .read_event_into(&mut buf)
            .map_err(|e| format!("Page du livre illisible : {e}"))?;
        match evenement {
            Event::Start(ref balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                if saute > 0 || matches!(nom, "style" | "script" | "svg" | "head") {
                    saute += 1;
                    continue;
                }
                match nom {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let niveau = nom[1..].parse::<u64>().unwrap_or(1);
                        machine.ouvrir(Genre::Titre(niveau));
                    }
                    "p" | "blockquote" | "figcaption" | "dd" => machine.ouvrir(Genre::Paragraphe),
                    "ul" | "menu" => machine.ouvrir(Genre::Liste(false)),
                    "ol" => machine.ouvrir(Genre::Liste(true)),
                    "li" => {
                        let niveau = machine
                            .pile
                            .iter()
                            .filter(|e| matches!(e.genre, Genre::Liste(_)))
                            .count()
                            .saturating_sub(1);
                        machine.ouvrir(Genre::Item(niveau));
                    }
                    "strong" | "b" => {
                        let mut étage = Etage::nouveau(Genre::Portion);
                        étage.gras = true;
                        machine.pile.push(étage);
                    }
                    "em" | "i" | "cite" => {
                        let mut étage = Etage::nouveau(Genre::Portion);
                        étage.italique = true;
                        machine.pile.push(étage);
                    }
                    "a" => {
                        let mut étage = Etage::nouveau(Genre::Lien);
                        étage.href = attribut(balise, "href");
                        machine.pile.push(étage);
                    }
                    "br" => machine.texte("\n"),
                    "table" => machine.ouvrir(Genre::Table),
                    "tr" => machine.ouvrir(Genre::Rangée),
                    "td" | "th" => machine.ouvrir(Genre::Cellule),
                    _ => {}
                }
            }
            Event::Empty(ref balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                if saute == 0 && nom == "br" {
                    machine.texte("\n");
                }
            }
            Event::Text(contenu) => {
                if saute == 0 {
                    machine.texte(&event_texte(&contenu));
                }
            }
            Event::CData(contenu) => {
                if saute == 0 {
                    machine.texte(&String::from_utf8_lossy(contenu.as_ref()));
                }
            }
            Event::End(ref balise) => {
                let nom_complet = balise.name();
                let nom = local(nom_complet.as_ref());
                if saute > 0 {
                    saute = saute.saturating_sub(1);
                    continue;
                }
                match nom {
                    "strong" | "b" | "em" | "i" | "cite" => machine.fermer_portion(),
                    "a" => machine.fermer_lien(),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => machine.fermer_titre(),
                    "p" | "blockquote" | "figcaption" | "dd" => machine.fermer_paragraphe(),
                    "li" => machine.fermer_item(),
                    "ul" | "menu" | "ol" => machine.fermer_liste(),
                    "td" | "th" => machine.fermer_cellule(),
                    "tr" => machine.fermer_rangée(),
                    "table" => machine.fermer_table(),
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(machine.fin())
}

fn epub(octets: &[u8]) -> Resultat<String> {
    // L'ordre des pages est celui de la colonne vertébrale du catalogue ;
    // chaque page s'y rejoint par son identifiant du manifeste.
    let conteneur = membre(octets, "META-INF/container.xml")?;
    let xml = String::from_utf8_lossy(&conteneur).to_string();
    let opf = chemin_opf(&xml).ok_or("Le livre ne dit pas où est son catalogue.")?;
    let opf_octets = membre(octets, &opf)?;
    let opf_xml = String::from_utf8_lossy(&opf_octets).to_string();
    let base = match opf.rfind('/') {
        Some(i) => format!("{}/", &opf[..i]),
        None => String::new(),
    };
    let manifeste =
        manifest_opf(&opf_xml).ok_or("Le catalogue du livre est illisible.")?;
    let ordre = spine_opf(&opf_xml).ok_or("L'ordre des pages du livre est illisible.")?;
    if ordre.is_empty() {
        return Err("Le livre n'a aucune page.".to_string());
    }
    let mut pages: Vec<String> = Vec::new();
    for idref in &ordre {
        if let Some(href) = manifeste.get(idref) {
            if let Ok(page) = membre(octets, &format!("{base}{href}")) {
                pages.push(html(&page)?);
            }
        }
    }
    Ok(joindre(&pages))
}

/// Chemin du catalogue dans le conteneur EPUB.
fn chemin_opf(xml: &str) -> Option<String> {
    balise_attribut(xml, "rootfile", "full-path")
}

/// Manifeste de l'OPF : identifiant vers adresse relative.
fn manifest_opf(xml: &str) -> Option<HashMap<String, String>> {
    let mut lecteur = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut manifeste = HashMap::new();
    let mut dans = false;
    loop {
        let evenement = lecteur.read_event_into(&mut buf).ok()?;
        match evenement {
            Event::Start(b) if local(b.name().as_ref()) == "manifest" => dans = true,
            Event::End(b) if local(b.name().as_ref()) == "manifest" => break,
            Event::Start(b) | Event::Empty(b) if dans && local(b.name().as_ref()) == "item" => {
                if let (Some(id), Some(href)) = (attribut(&b, "id"), attribut(&b, "href")) {
                    manifeste.insert(id, href.split('#').next().unwrap_or("").to_string());
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Some(manifeste)
}

/// Colonne vertébrale de l'OPF : l'ordre des pages du livre.
fn spine_opf(xml: &str) -> Option<Vec<String>> {
    let mut lecteur = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut ordre = Vec::new();
    let mut dans = false;
    loop {
        let evenement = lecteur.read_event_into(&mut buf).ok()?;
        match evenement {
            Event::Start(b) if local(b.name().as_ref()) == "spine" => dans = true,
            Event::End(b) if local(b.name().as_ref()) == "spine" => break,
            Event::Start(b) | Event::Empty(b) if dans && local(b.name().as_ref()) == "itemref" => {
                if let Some(idref) = attribut(&b, "idref") {
                    ordre.push(idref);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Some(ordre)
}

/// Premier attribut d'une balise donnée, dans un petit document — l'usage
/// du conteneur EPUB, qui n'a qu'une balise utile.
fn balise_attribut(xml: &str, balise: &str, nom: &str) -> Option<String> {
    let mut lecteur = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        let evenement = lecteur.read_event_into(&mut buf).ok()?;
        match evenement {
            Event::Start(b) | Event::Empty(b) if local(b.name().as_ref()) == balise => {
                if let Some(valeur) = attribut(&b, nom) {
                    return Some(valeur);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Archive ZIP en mémoire : les tests bâtissent leurs échantillons
    /// complets, rien ne se lit du disque.
    fn zip(membres: &[(&str, &str)]) -> Vec<u8> {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (nom, contenu) in membres {
            archive.start_file::<_, ()>(nom.to_string(), options).unwrap();
            archive.write_all(contenu.as_bytes()).unwrap();
        }
        archive.finish().unwrap().into_inner()
    }

    fn docx_xml(corps: &str) -> Vec<u8> {
        zip(&[(
            "word/document.xml",
            &format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="ex"><w:body>{corps}</w:body></w:document>"#
            ),
        )])
    }

    #[test]
    fn docx_titres_gras_listes() {
        let octets = docx_xml(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Modèle</w:t></w:r></w:p>
               <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Vital :</w:t></w:r><w:r><w:t> voir les listes.</w:t></w:r></w:p>
               <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/></w:numPr></w:pPr><w:r><w:t>chou</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>vert</w:t></w:r><w:pPr><w:numPr/></w:pPr></w:p>"#,
        );
        assert_eq!(
            docx(&octets).unwrap(),
            "# Modèle\n\n**Vital :** voir les listes.\n\n- chou*vert*"
        );
    }

    #[test]
    fn docx_tableau() {
        let octets = docx_xml(
            r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Nom</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Prix</w:t></w:r></w:p></w:tc></w:tr>
               <w:tr><w:tc><w:p><w:r><w:t>Chou</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>1 €</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        );
        assert_eq!(
            docx(&octets).unwrap(),
            "| Nom | Prix |\n| --- | --- |\n| Chou | 1 € |"
        );
    }

    #[test]
    fn odt_titres_et_emphases() {
        let octets = zip(&[(
            "content.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="o" xmlns:text="t" xmlns:style="s" xmlns:fo="f">
<office:automatic-styles>
  <style:style style:name="G1" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
  <style:style style:name="I1" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
</office:automatic-styles>
<office:body>
<text:p>Avant tout, <text:span text:style-name="G1">le gras</text:span> et <text:span text:style-name="I1">l'italique</text:span>.</text:p>
<text:h text:outline-level="2">Second titre</text:h>
<text:list><text:list-item><text:p>un point</text:p></text:list-item></text:list>
</office:body></office:document-content>"#,
        )]);
        assert_eq!(
            odt(&octets).unwrap(),
            "Avant tout, **le gras** et *l'italique*.\n\n## Second titre\n\n- un point"
        );
    }

    #[test]
    fn epub_deux_pages_dans_l_ordre() {
        let octets = zip(&[
            (
                "META-INF/container.xml",
                r#"<container><rootfiles><rootfile full-path="OEBPS/livre.opf"/></rootfiles></container>"#,
            ),
            (
                "OEBPS/livre.opf",
                r#"<package><manifest><item id="p2" href="p2.xhtml"/><item id="p1" href="p1.xhtml"/></manifest>
                    <spine><itemref idref="p1"/><itemref idref="p2"/></spine></package>"#,
            ),
            (
                "OEBPS/p1.xhtml",
                r#"<html><body><h1>Chapitre</h1><p>Du <strong>gras</strong> et un <a href="p2.xhtml">lien</a>.</p></body></html>"#,
            ),
            (
                "OEBPS/p2.xhtml",
                r#"<html><body><h2>Suite</h2><ol><li>premier</li><li><p>second, en paragraphe</p></li></ol></body></html>"#,
            ),
        ]);
        assert_eq!(
            epub(&octets).unwrap(),
            "# Chapitre\n\nDu **gras** et un [lien](p2.xhtml).\n\n## Suite\n\n1. premier\n2. second, en paragraphe"
        );
    }

    #[test]
    fn pas_une_archive() {
        let erreur = vers_markdown("/tmp/essai.docx", b"n'est pas un zip").unwrap_err();
        assert!(erreur.contains("ZIP"), "{erreur}");
        let octets = zip(&[("autre.txt", "rien")]);
        let erreur = vers_markdown("/tmp/essai.odt", &octets).unwrap_err();
        assert!(erreur.contains("introuvable"), "{erreur}");
    }

    #[test]
    fn aiguillage_par_extension() {
        let octets = docx_xml("<w:p><w:r><w:t>x</w:t></w:r></w:p>");
        assert!(vers_markdown("/tmp/a.docx", &octets).is_ok());
        assert!(vers_markdown("/tmp/a.DOCX", &octets).is_ok());
        assert!(vers_markdown("/tmp/a.doc", &octets).is_err());
    }
}
