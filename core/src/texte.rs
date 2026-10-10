// SPDX-License-Identifier: GPL-3.0-or-later
//! Traitements de texte du noyau, sans aucune dépendance à Qt.
//!
//! Tout ce qui est ici est pur : une entrée, une sortie, pas d'état. C'est la
//! partie que l'on peut éprouver par des tests unitaires ordinaires, et elle
//! reproduit le comportement de la version PySide6 (lecture avec repli
//! d'encodage, écriture UTF-8 en CRLF, format déduit de l'extension).

/// Aide-mémoire des balises, affiché par le menu Aide — repris tel quel de la
/// version PySide6, où il vivait dans `style.py`.
pub const AIDE_BALISES: &str = r#"# Rappel des balises Markdown

## Titres
`# Titre 1`  ·  `## Titre 2`  ·  `### Titre 3`

## Emphase
`**gras**`  ·  `*italique*`  ·  `` `code` ``  ·  `~~barré~~`

## Listes
- `- élément` (liste à puces)
- `1. élément` (liste numérotée)
- Indenter de deux espaces pour un sous-niveau.

## Liens et images
`[texte](https://exemple.fr)`
`![texte alternatif](chemin/image.png)`

## Citation
`> ligne citée`

## Bloc de code
Entourer par trois accents graves ``` sur leur propre ligne.

## Séparateur horizontal
`---`

## Tableau
`| Colonne A | Colonne B |`
`| --- | --- |`
`| valeur 1 | valeur 2 |`
"#;

/// Extensions traitées comme du Markdown : aperçu rendu et coloration Markdown.
const EXT_MARKDOWN: &[&str] = &["md", "markdown"];

/// Formats balisés : coloration XML, aperçu Markdown sans objet.
const EXT_XML: &[&str] = &[
    "xml", "xsd", "xsl", "xslt", "svg", "rss", "atom", "html", "htm", "xhtml", "plist", "config",
    "csproj", "props", "targets", "vcxproj", "resx", "nuspec", "ui", "qrc", "xaml",
];

/// Données et configuration : clés, chaînes, nombres, commentaires, sections.
const EXT_DONNEES: &[&str] = &[
    "json", "jsonc", "yaml", "yml", "toml", "ini", "conf", "cfg", "env", "properties", "reg",
    "desktop", "service",
];

/// Scripts et code : commentaires, chaînes, nombres, mots-clés, variables.
const EXT_CODE: &[&str] = &[
    "sh", "bash", "zsh", "ps1", "psm1", "psd1", "bat", "cmd", "py", "rs", "js", "mjs", "cjs",
    "ts", "c", "h", "cpp", "hpp", "cc", "cs", "java", "go", "php", "rb", "lua", "sql", "vbs",
];

/// Journaux : niveaux de gravité et horodatages.
const EXT_JOURNAL: &[&str] = &["log", "out", "err", "trace"];

/// Fichiers reconnus à leur nom, faute d'extension — « .env » n'en a pas, au
/// sens de la fonction `extension()` ci-dessous, et un Makefile n'en porte pas.
const NOMS_CONNUS: &[(&str, Format)] = &[
    ("makefile", Format::Code),
    ("dockerfile", Format::Code),
    ("containerfile", Format::Code),
    (".env", Format::Donnees),
    (".editorconfig", Format::Donnees),
    (".gitconfig", Format::Donnees),
];

/// Format d'un document, déduit de l'extension du fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Xml,
    Donnees,
    Code,
    Journal,
    Texte,
}

impl Format {
    /// Clé stable, telle qu'elle est exposée à QML.
    pub fn cle(self) -> &'static str {
        match self {
            Format::Markdown => "markdown",
            Format::Xml => "xml",
            Format::Donnees => "donnees",
            Format::Code => "code",
            Format::Journal => "journal",
            Format::Texte => "texte",
        }
    }

    /// Libellé affiché en barre d'état.
    pub fn libelle(self) -> &'static str {
        match self {
            Format::Markdown => "Markdown",
            Format::Xml => "XML",
            Format::Donnees => "Configuration",
            Format::Code => "Code",
            Format::Journal => "Journal",
            Format::Texte => "Texte",
        }
    }
}

/// Format déduit d'un chemin. Un document sans chemin est du Markdown.
///
/// Le nom complet est regardé avant l'extension : « .env » et « Makefile » n'en
/// ont pas, et tomberaient sinon dans le format texte.
pub fn format_du_chemin(chemin: &str) -> Format {
    if chemin.is_empty() {
        return Format::Markdown;
    }
    let nom = nom_fichier(chemin).to_ascii_lowercase();
    if let Some((_, format)) = NOMS_CONNUS.iter().find(|(connu, _)| *connu == nom) {
        return *format;
    }
    let ext = extension(chemin);
    if EXT_MARKDOWN.contains(&ext.as_str()) {
        Format::Markdown
    } else if EXT_XML.contains(&ext.as_str()) {
        Format::Xml
    } else if EXT_DONNEES.contains(&ext.as_str()) {
        Format::Donnees
    } else if EXT_CODE.contains(&ext.as_str()) {
        Format::Code
    } else if EXT_JOURNAL.contains(&ext.as_str()) {
        Format::Journal
    } else {
        Format::Texte
    }
}

fn extension(chemin: &str) -> String {
    let nom = nom_fichier(chemin);
    match nom.rfind('.') {
        // Un nom commençant par un point n'a pas d'extension (« .gitignore »).
        Some(i) if i > 0 => nom[i + 1..].to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// Dernier segment d'un chemin, quel que soit le séparateur.
pub fn nom_fichier(chemin: &str) -> String {
    let fin = chemin.trim_end_matches(['/', '\\']);
    match fin.rfind(['/', '\\']) {
        Some(i) => fin[i + 1..].to_string(),
        None => fin.to_string(),
    }
}

/// Chemin sans son extension, pour proposer un nom d'export.
pub fn sans_extension(chemin: &str) -> String {
    let nom = nom_fichier(chemin);
    match nom.rfind('.') {
        Some(i) if i > 0 => chemin[..chemin.len() - (nom.len() - i)].to_string(),
        _ => chemin.to_string(),
    }
}

/// Convertit une URL de fichier (ce que rend `FileDialog`) en chemin local.
///
/// QML rend `file:///home/manu/notes.md` sous Linux et `file:///C:/notes.md`
/// sous Windows : la barre de tête doit sauter dans le second cas, pas dans le
/// premier. Les caractères spéciaux arrivent encodés (`%20`).
pub fn chemin_depuis_url(url: &str) -> String {
    let reste = match url.strip_prefix("file://") {
        // Une URL peut porter un hôte vide (« file:///… ») ou, sous Windows, un
        // partage réseau (« file://serveur/partage/… ») qu'il faut rendre en UNC.
        Some(r) => match r.strip_prefix('/') {
            Some(chemin) => chemin.to_string(),
            None => format!("//{r}"),
        },
        None => return decoder_pourcent(url),
    };
    let chemin = decoder_pourcent(&reste);
    // « C:/x » sous Windows : l'URL ne garde pas de barre de tête devant la lettre
    // de lecteur, alors qu'un chemin POSIX, lui, commence bien par « / ».
    if est_lecteur_windows(&chemin) || chemin.starts_with("//") {
        chemin
    } else {
        format!("/{chemin}")
    }
}

fn est_lecteur_windows(chemin: &str) -> bool {
    let o = chemin.as_bytes();
    o.len() >= 2 && o[0].is_ascii_alphabetic() && o[1] == b':'
}

fn decoder_pourcent(s: &str) -> String {
    let o = s.as_bytes();
    let mut sortie: Vec<u8> = Vec::with_capacity(o.len());
    let mut i = 0;
    while i < o.len() {
        if o[i] == b'%' && i + 2 < o.len() {
            if let (Some(a), Some(b)) = (hex(o[i + 1]), hex(o[i + 2])) {
                sortie.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        sortie.push(o[i]);
        i += 1;
    }
    String::from_utf8_lossy(&sortie).into_owned()
}

fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Décode un fichier texte : UTF-8, puis repli CP1252, puis Latin-1.
///
/// Rend le texte et le nom de l'encodage retenu. Latin-1 ne peut pas échouer :
/// il accepte n'importe quelle suite d'octets, et clôt donc la liste.
pub fn decoder(octets: &[u8]) -> (String, &'static str) {
    if let Ok(texte) = std::str::from_utf8(octets) {
        // Un BOM UTF-8 n'a pas à se retrouver dans le document.
        return (texte.trim_start_matches('\u{feff}').to_string(), "UTF-8");
    }
    if let Some(texte) = decoder_cp1252(octets) {
        return (texte, "CP1252");
    }
    (octets.iter().map(|o| *o as char).collect(), "LATIN-1")
}

/// CP1252 strict : rend `None` si un octet n'a pas de correspondance.
///
/// Cinq positions de la plage 0x80-0x9F sont vides dans CP1252 ; les rendre par
/// un caractère de remplacement masquerait un fichier d'un autre encodage.
fn decoder_cp1252(octets: &[u8]) -> Option<String> {
    const HAUT: [char; 32] = [
        '\u{20ac}', '\0', '\u{201a}', '\u{0192}', '\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}',
        '\u{02c6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\0', '\u{017d}', '\0', '\0',
        '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}',
        '\u{02dc}', '\u{2122}', '\u{0161}', '\u{203a}', '\u{0153}', '\0', '\u{017e}', '\u{0178}',
    ];
    let mut sortie = String::with_capacity(octets.len());
    for o in octets {
        match o {
            0x80..=0x9f => {
                let c = HAUT[(o - 0x80) as usize];
                if c == '\0' {
                    return None;
                }
                sortie.push(c);
            }
            _ => sortie.push(*o as char),
        }
    }
    Some(sortie)
}

/// Encode en UTF-8 sans BOM, fins de ligne normalisées en CRLF.
pub fn encoder_utf8_crlf(texte: &str) -> Vec<u8> {
    let normalise = texte.replace("\r\n", "\n").replace('\r', "\n");
    normalise.replace('\n', "\r\n").into_bytes()
}

/// Mots, caractères et lignes, comptés comme dans la version PySide6.
pub fn statistiques(texte: &str) -> (usize, usize, usize) {
    let caracteres = texte.chars().count();
    let mots = texte.split_whitespace().count();
    let lignes = if texte.is_empty() {
        0
    } else {
        texte.matches('\n').count() + 1
    };
    (mots, caracteres, lignes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_par_extension() {
        assert_eq!(format_du_chemin(""), Format::Markdown);
        assert_eq!(format_du_chemin("/tmp/notes.md"), Format::Markdown);
        assert_eq!(format_du_chemin("/tmp/NOTES.MARKDOWN"), Format::Markdown);
        assert_eq!(format_du_chemin(r"C:\x\page.HTML"), Format::Xml);
        assert_eq!(format_du_chemin("/tmp/vue.ui"), Format::Xml);
        assert_eq!(format_du_chemin("/tmp/releve.csv"), Format::Texte);
        assert_eq!(format_du_chemin("/tmp/.gitignore"), Format::Texte);
    }

    #[test]
    fn formats_configuration_code_journal() {
        assert_eq!(format_du_chemin("/etc/app/reglages.JSON"), Format::Donnees);
        assert_eq!(format_du_chemin("/etc/app.conf"), Format::Donnees);
        assert_eq!(format_du_chemin("/tmp/compose.yml"), Format::Donnees);
        assert_eq!(format_du_chemin("/tmp/deploy.sh"), Format::Code);
        assert_eq!(format_du_chemin(r"C:\scripts\Inventaire.PS1"), Format::Code);
        assert_eq!(format_du_chemin("/tmp/requete.sql"), Format::Code);
        assert_eq!(format_du_chemin("/var/log/syslog.log"), Format::Journal);
        // Reconnus à leur nom : ils n'ont pas d'extension.
        assert_eq!(format_du_chemin("/src/Makefile"), Format::Code);
        assert_eq!(format_du_chemin("/src/Dockerfile"), Format::Code);
        assert_eq!(format_du_chemin("/home/dev/.env"), Format::Donnees);
        // Un nom connu l'emporte sur l'absence d'extension, pas sur une vraie.
        assert_eq!(format_du_chemin("/src/makefile.md"), Format::Markdown);
    }

    #[test]
    fn cles_et_libelles_distincts() {
        let formats = [
            Format::Markdown,
            Format::Xml,
            Format::Donnees,
            Format::Code,
            Format::Journal,
            Format::Texte,
        ];
        // Les clés partent à QML, qui les compare telles quelles pour choisir la
        // coloration : un doublon y attacherait silencieusement la mauvaise.
        for (i, a) in formats.iter().enumerate() {
            for b in &formats[i + 1..] {
                assert_ne!(a.cle(), b.cle());
                assert_ne!(a.libelle(), b.libelle());
            }
        }
    }

    #[test]
    fn nom_et_extension() {
        assert_eq!(nom_fichier("/home/manu/notes.md"), "notes.md");
        assert_eq!(nom_fichier(r"C:\Users\manu\notes.md"), "notes.md");
        assert_eq!(nom_fichier("notes.md"), "notes.md");
        assert_eq!(sans_extension("/home/manu/notes.md"), "/home/manu/notes");
        assert_eq!(sans_extension("/home/manu/notes"), "/home/manu/notes");
    }

    #[test]
    fn url_vers_chemin() {
        assert_eq!(chemin_depuis_url("file:///home/manu/a b.md"), "/home/manu/a b.md");
        assert_eq!(chemin_depuis_url("file:///home/manu/a%20b.md"), "/home/manu/a b.md");
        assert_eq!(chemin_depuis_url("file:///C:/Users/manu/n.md"), "C:/Users/manu/n.md");
        assert_eq!(chemin_depuis_url("file://serveur/part/n.md"), "//serveur/part/n.md");
        assert_eq!(chemin_depuis_url("/home/manu/n.md"), "/home/manu/n.md");
    }

    #[test]
    fn encodages() {
        assert_eq!(decoder("ete".as_bytes()), ("ete".to_string(), "UTF-8"));
        assert_eq!(decoder(b"\xef\xbb\xbfok"), ("ok".to_string(), "UTF-8"));
        // 0xE9 seul n'est pas de l'UTF-8 : repli CP1252.
        assert_eq!(decoder(b"caf\xe9"), ("caf\u{e9}".to_string(), "CP1252"));
        // 0x81 est un trou de CP1252 : dernier repli Latin-1.
        let (texte, enc) = decoder(b"\x81\xe9");
        assert_eq!(enc, "LATIN-1");
        assert_eq!(texte, "\u{81}\u{e9}");
    }

    #[test]
    fn ecriture_crlf() {
        assert_eq!(encoder_utf8_crlf("a\nb"), b"a\r\nb".to_vec());
        // Déjà en CRLF : pas de doublement.
        assert_eq!(encoder_utf8_crlf("a\r\nb"), b"a\r\nb".to_vec());
        // Anciennes fins de ligne Mac.
        assert_eq!(encoder_utf8_crlf("a\rb"), b"a\r\nb".to_vec());
    }

    #[test]
    fn comptes() {
        assert_eq!(statistiques(""), (0, 0, 0));
        assert_eq!(statistiques("un deux"), (2, 7, 1));
        assert_eq!(statistiques("un\ndeux\n"), (2, 8, 3));
        assert_eq!(statistiques("\u{e9}t\u{e9}"), (1, 3, 1));
    }
}
