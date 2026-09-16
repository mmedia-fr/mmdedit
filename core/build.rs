// SPDX-License-Identifier: GPL-3.0-or-later
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Le rcc qui empaquette les fichiers QML est celui de l'hôte, et il compresse
    // en zstd dès que le gain le justifie. Or le Qt officiel pour Android est
    // construit sans zstd : l'édition de liens s'arrête alors sur « undefined
    // symbol: qt_resourceFeatureZstd ». Le défaut n'apparaît qu'une fois les
    // fichiers QML assez gros pour valoir une compression — d'où un socle qui
    // passait, et un éditeur qui ne passe plus. zlib fait aussi bien ici.
    std::env::set_var("CXX_QT_AUTORCC_OPTIONS", "--no-zstd");

    CxxQtBuilder::new_qml_module(QmlModule::new("fr.mmedia.mmdedit").qml_files([
        "qml/Main.qml",
        "qml/DialogueRecherche.qml",
        "qml/DialogueTexte.qml",
    ]))
    .qt_module("Qml")
    .files([
        "src/socle.rs",
        "src/document.rs",
        "src/pont_edition.rs",
        "src/pont_presse_papier.rs",
    ])
    .build();
}
