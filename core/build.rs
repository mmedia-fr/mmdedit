// SPDX-License-Identifier: GPL-3.0-or-later
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("fr.mmedia.mmdedit").qml_file("../qml/Main.qml"))
        .qt_module("Qml")
        .files(["src/socle.rs", "src/document.rs"])
        .build();
}
