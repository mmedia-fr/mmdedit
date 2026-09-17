// SPDX-License-Identifier: GPL-3.0-or-later
//! Objet témoin : une propriété lue et une méthode appelée depuis QML prouvent la liaison.
//!
//! Il porte aussi ce que l'interface doit savoir du programme lui-même — version,
//! textes d'aide — pour que le QML n'ait pas à les redire de son côté.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, noyau)]
        /// Version du programme, telle que la porte le noyau.
        #[qproperty(QString, version)]
        type Socle = super::SocleRust;

        #[qinvokable]
        #[cxx_name = "plateforme"]
        fn plateforme(&self) -> QString;

        /// Aide-mémoire des balises Markdown, en Markdown.
        #[qinvokable]
        #[cxx_name = "aideBalises"]
        fn aide_balises(&self) -> QString;
    }
}

use cxx_qt_lib::QString;

pub struct SocleRust {
    noyau: QString,
    version: QString,
}

impl Default for SocleRust {
    fn default() -> Self {
        Self {
            noyau: QString::from(&format!("mmdedit_core {}", env!("CARGO_PKG_VERSION"))),
            version: QString::from(env!("CARGO_PKG_VERSION")),
        }
    }
}

impl qobject::Socle {
    pub fn plateforme(&self) -> QString {
        QString::from(&format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
    }

    pub fn aide_balises(&self) -> QString {
        QString::from(crate::texte::AIDE_BALISES)
    }
}
