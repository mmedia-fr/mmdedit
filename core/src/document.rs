// SPDX-License-Identifier: GPL-3.0-or-later
//! Document ouvert : état du fichier et accès disque, exposés à QML.
//!
//! L'interface QML détient le texte (c'est le `TextArea` qui l'édite) ; cet
//! objet ne porte que ce qui l'entoure — chemin, nom affiché, encodage lu,
//! format déduit de l'extension — et rend les services qui touchent au disque.

use crate::texte;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// Chemin local du fichier ouvert ; vide pour un document jamais enregistré.
        #[qproperty(QString, chemin)]
        /// Nom affiché dans le titre : nom du fichier, ou « Sans titre ».
        #[qproperty(QString, nom)]
        /// Encodage retenu à la lecture, affiché en barre d'état.
        #[qproperty(QString, encodage)]
        /// Format du document : clé stable déduite du fichier ouvert (markdown,
        /// xml, donnees, code, journal, pdf, bureautique, texte).
        #[qproperty(QString, format)]
        /// Libellé du format, pour la barre d'état.
        #[qproperty(QString, libelle)]
        /// Dernier message d'erreur ; vide s'il n'y en a pas eu.
        #[qproperty(QString, erreur)]
        type Document = super::DocumentRust;

        /// Repart d'un document vide, traité comme du texte brut.
        #[qinvokable]
        #[cxx_name = "nouveau"]
        fn nouveau(self: Pin<&mut Document>);

        /// Lit le fichier désigné par une URL `file://` et rend son texte.
        ///
        /// En cas d'échec, rend une chaîne vide et renseigne `erreur` : l'appelant
        /// doit donc regarder `erreur`, et non le texte rendu, pour conclure.
        #[qinvokable]
        #[cxx_name = "ouvrir"]
        fn ouvrir(self: Pin<&mut Document>, url: &QString) -> QString;

        /// Écrit le texte dans le fichier désigné, en UTF-8 et en CRLF. Les
        /// cibles bureautiques sont refusées : l'original d'un import ne
        /// s'écrase pas, et seul « Enregistrer sous » exporte le Markdown.
        #[qinvokable]
        #[cxx_name = "enregistrer"]
        fn enregistrer(self: Pin<&mut Document>, url: &QString, contenu: &QString) -> bool;

        /// Ligne de comptes de la barre d'état.
        #[qinvokable]
        #[cxx_name = "statistiques"]
        fn statistiques(&self, contenu: &QString) -> QString;

        /// Vrai si le document courant se rend en Markdown.
        #[qinvokable]
        #[cxx_name = "estMarkdown"]
        fn est_markdown(&self) -> bool;

        /// Vrai si l'URL désigne un PDF : il se feuillette, il ne s'édite pas.
        #[qinvokable]
        #[cxx_name = "estPdf"]
        fn est_pdf(&self, url: &QString) -> bool;

        /// Ouvre un PDF sans en lire le moindre texte : pose chemin, nom et
        /// format `pdf`, et rend vrai si le fichier existe. Le feuilleteur
        /// charge le fichier de son côté, hors du noyau.
        #[qinvokable]
        #[cxx_name = "ouvrirPdf"]
        fn ouvrir_pdf(self: Pin<&mut Document>, url: &QString) -> bool;

        /// Nom de fichier proposé pour un export PDF, à partir du chemin courant.
        #[qinvokable]
        #[cxx_name = "cheminExport"]
        fn chemin_export(&self, extension: &QString) -> QString;

        /// Remet `erreur` à vide, une fois le message montré à l'utilisateur.
        #[qinvokable]
        #[cxx_name = "oublierErreur"]
        fn oublier_erreur(self: Pin<&mut Document>);
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;

pub struct DocumentRust {
    chemin: QString,
    nom: QString,
    encodage: QString,
    format: QString,
    libelle: QString,
    erreur: QString,
}

/// Nom affiché tant que le document n'a pas de fichier.
const SANS_TITRE: &str = "Sans titre";

impl Default for DocumentRust {
    fn default() -> Self {
        Self {
            chemin: QString::from(""),
            nom: QString::from(SANS_TITRE),
            encodage: QString::from("UTF-8"),
            format: QString::from(texte::Format::Texte.cle()),
            libelle: QString::from(texte::Format::Texte.libelle()),
            erreur: QString::from(""),
        }
    }
}

impl qobject::Document {
    /// Met chemin, nom et format en accord avec un chemin local (vide = nouveau).
    fn poser_chemin(mut self: Pin<&mut Self>, chemin: &str) {
        let format = texte::format_du_chemin(chemin);
        let nom = if chemin.is_empty() {
            SANS_TITRE.to_string()
        } else {
            texte::nom_fichier(chemin)
        };
        self.as_mut().set_chemin(QString::from(chemin));
        self.as_mut().set_nom(QString::from(&nom));
        self.as_mut().set_format(QString::from(format.cle()));
        self.as_mut().set_libelle(QString::from(format.libelle()));
    }

    pub fn nouveau(mut self: Pin<&mut Self>) {
        self.as_mut().poser_chemin("");
        self.as_mut().set_encodage(QString::from("UTF-8"));
        self.as_mut().set_erreur(QString::from(""));
    }

    pub fn ouvrir(mut self: Pin<&mut Self>, url: &QString) -> QString {
        let chemin = texte::chemin_depuis_url(&url.to_string());
        // Un PDF se feuillette, il ne se lit pas comme du texte : des octets
        // binaires y passeraient pour un document illisible de gagné.
        let format = texte::format_du_chemin(&chemin);
        if format == texte::Format::Pdf {
            self.as_mut()
                .set_erreur(QString::from("Un PDF se feuillette ; il ne s'édite pas."));
            return QString::from("");
        }
        // Un document bureautique s'importe : le noyau en rend du Markdown,
        // fidélité limitée, et l'original ne s'écrit jamais par-dessus.
        if format == texte::Format::Bureautique {
            let octets = match std::fs::read(&chemin) {
                Ok(o) => o,
                Err(e) => {
                    self.as_mut()
                        .set_erreur(QString::from(&format!("Lecture impossible :\n{e}")));
                    return QString::from("");
                }
            };
            return match crate::bureautique::vers_markdown(&chemin, &octets) {
                Ok(markdown) => {
                    self.as_mut().poser_chemin(&chemin);
                    // Le Markdown est de nous : son encodage l'est aussi.
                    self.as_mut().set_encodage(QString::from("UTF-8"));
                    self.as_mut().set_erreur(QString::from(""));
                    QString::from(&markdown)
                }
                Err(explication) => {
                    self.as_mut().set_erreur(QString::from(&format!(
                        "Import impossible :\n{explication}"
                    )));
                    QString::from("")
                }
            };
        }
        let octets = match std::fs::read(&chemin) {
            Ok(o) => o,
            Err(e) => {
                self.as_mut()
                    .set_erreur(QString::from(&format!("Lecture impossible :\n{e}")));
                return QString::from("");
            }
        };
        let (contenu, encodage) = texte::decoder(&octets);
        self.as_mut().poser_chemin(&chemin);
        self.as_mut().set_encodage(QString::from(encodage));
        self.as_mut().set_erreur(QString::from(""));
        QString::from(&contenu)
    }

    pub fn enregistrer(mut self: Pin<&mut Self>, url: &QString, contenu: &QString) -> bool {
        let chemin = texte::chemin_depuis_url(&url.to_string());
        if texte::format_du_chemin(&chemin) == texte::Format::Bureautique {
            self.as_mut().set_erreur(QString::from(
                "Un document bureautique s'importe en Markdown ; il ne s'écrit pas dessus.\nEnregistrez-le sous un autre nom et un autre type.",
            ));
            return false;
        }
        let octets = texte::encoder_utf8_crlf(&contenu.to_string());
        if let Err(e) = std::fs::write(&chemin, octets) {
            self.as_mut()
                .set_erreur(QString::from(&format!("Écriture impossible :\n{e}")));
            return false;
        }
        self.as_mut().poser_chemin(&chemin);
        // Le fichier vient d'être écrit en UTF-8, quel que soit l'encodage lu.
        self.as_mut().set_encodage(QString::from("UTF-8"));
        self.as_mut().set_erreur(QString::from(""));
        true
    }

    pub fn statistiques(&self, contenu: &QString) -> QString {
        let (mots, caracteres, lignes) = texte::statistiques(&contenu.to_string());
        QString::from(&format!(
            "{mots} mots · {caracteres} caractères · {lignes} lignes"
        ))
    }

    pub fn est_markdown(&self) -> bool {
        let format = self.format().to_string();
        format == texte::Format::Markdown.cle() || format == texte::Format::Bureautique.cle()
    }

    pub fn est_pdf(&self, url: &QString) -> bool {
        texte::format_du_chemin(&texte::chemin_depuis_url(&url.to_string())) == texte::Format::Pdf
    }

    pub fn ouvrir_pdf(mut self: Pin<&mut Self>, url: &QString) -> bool {
        let chemin = texte::chemin_depuis_url(&url.to_string());
        if chemin.is_empty() || !std::path::Path::new(&chemin).is_file() {
            self.as_mut()
                .set_erreur(QString::from(&format!("Feuilletteur impossible :\n{chemin}")));
            return false;
        }
        self.as_mut().poser_chemin(&chemin);
        // Aucun texte n'est lu : l'encodage ne se pose pas, et la barre d'état
        // affiche le libellé du format, « PDF », à sa place.
        self.as_mut().set_encodage(QString::from(""));
        self.as_mut().set_erreur(QString::from(""));
        true
    }

    pub fn chemin_export(&self, extension: &QString) -> QString {
        let chemin = self.chemin().to_string();
        if chemin.is_empty() {
            return QString::from("");
        }
        QString::from(&format!(
            "{}{}",
            texte::sans_extension(&chemin),
            extension.to_string()
        ))
    }

    pub fn oublier_erreur(mut self: Pin<&mut Self>) {
        self.as_mut().set_erreur(QString::from(""));
    }
}
