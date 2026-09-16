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
        /// Format du document : « markdown », « xml » ou « texte ».
        #[qproperty(QString, format)]
        /// Libellé du format, pour la barre d'état.
        #[qproperty(QString, libelle)]
        /// Dernier message d'erreur ; vide s'il n'y en a pas eu.
        #[qproperty(QString, erreur)]
        type Document = super::DocumentRust;

        /// Repart d'un document vide, traité comme du Markdown.
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

        /// Écrit le texte dans le fichier désigné, en UTF-8 et en CRLF.
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
            format: QString::from(texte::Format::Markdown.cle()),
            libelle: QString::from(texte::Format::Markdown.libelle()),
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
        self.format().to_string() == texte::Format::Markdown.cle()
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
