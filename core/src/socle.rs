// SPDX-License-Identifier: GPL-3.0-or-later
//! Objet témoin : une propriété lue et une méthode appelée depuis QML prouvent la liaison.

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
        type Socle = super::SocleRust;

        #[qinvokable]
        #[cxx_name = "plateforme"]
        fn plateforme(&self) -> QString;
    }
}

use cxx_qt_lib::QString;

pub struct SocleRust {
    noyau: QString,
}

impl Default for SocleRust {
    fn default() -> Self {
        Self { noyau: QString::from(&format!("mmdedit_core {}", env!("CARGO_PKG_VERSION"))) }
    }
}

impl qobject::Socle {
    pub fn plateforme(&self) -> QString {
        QString::from(&format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
    }
}
