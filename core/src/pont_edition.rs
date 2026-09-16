// SPDX-License-Identifier: GPL-3.0-or-later
//! Passerelle QML vers l'édition assistée (`crate::edition`).
//!
//! Chaque méthode rend un plan sous forme d'objet JavaScript — `debut`, `fin`,
//! `remplacement`, `selectionDebut`, `selectionLongueur` — que l'interface
//! applique par `remove()` puis `insert()` sur le TextArea, pour préserver la
//! pile d'annulation. Rien n'est décidé ici : tout est dans `edition`, qui est
//! pur et testé.

use crate::edition;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Edition = super::EditionRust;

        /// Pose ou retire un encadrement Markdown (`**`, `*`, `~~`, `` ` ``).
        #[qinvokable]
        #[cxx_name = "encadrement"]
        fn encadrement(
            &self,
            texte: &QString,
            debut: i32,
            fin: i32,
            marqueur: &QString,
        ) -> QMap_QString_QVariant;

        /// Pose, remplace ou retire un préfixe sur la ligne du curseur.
        #[qinvokable]
        #[cxx_name = "prefixe"]
        fn prefixe(&self, texte: &QString, position: i32, prefixe: &QString)
            -> QMap_QString_QVariant;

        /// Encadre la sélection par deux textes libres (lien, image).
        #[qinvokable]
        #[cxx_name = "entourer"]
        fn entourer(
            &self,
            texte: &QString,
            debut: i32,
            fin: i32,
            avant: &QString,
            apres: &QString,
        ) -> QMap_QString_QVariant;

        /// Position de l'occurrence cherchée, ou -1 si le motif est absent.
        #[qinvokable]
        #[cxx_name = "chercher"]
        fn chercher(
            &self,
            texte: &QString,
            motif: &QString,
            depuis: i32,
            casse: bool,
            mot_entier: bool,
            en_arriere: bool,
        ) -> i32;

        /// Remplace toutes les occurrences : rend `texte` et `compte`.
        #[qinvokable]
        #[cxx_name = "remplacerTout"]
        fn remplacer_tout(
            &self,
            texte: &QString,
            motif: &QString,
            remplacement: &QString,
            casse: bool,
            mot_entier: bool,
        ) -> QMap_QString_QVariant;
    }
}

use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QVariant};

type Carte = QMap<QMapPair_QString_QVariant>;

#[derive(Default)]
pub struct EditionRust;

/// Une position venue de QML est un nombre JavaScript : on la ramène dans les
/// bornes avant de s'en servir comme index.
fn position(valeur: i32) -> usize {
    valeur.max(0) as usize
}

fn carte_du_plan(plan: &edition::Plan) -> Carte {
    let mut carte = Carte::default();
    carte.insert(QString::from("debut"), QVariant::from(&(plan.debut as i32)));
    carte.insert(QString::from("fin"), QVariant::from(&(plan.fin as i32)));
    carte.insert(
        QString::from("remplacement"),
        QVariant::from(&QString::from(&plan.remplacement)),
    );
    carte.insert(
        QString::from("selectionDebut"),
        QVariant::from(&(plan.selection_debut as i32)),
    );
    carte.insert(
        QString::from("selectionLongueur"),
        QVariant::from(&(plan.selection_longueur as i32)),
    );
    carte
}

impl qobject::Edition {
    pub fn encadrement(
        &self,
        texte: &QString,
        debut: i32,
        fin: i32,
        marqueur: &QString,
    ) -> Carte {
        let plan = edition::basculer_encadrement(
            &texte.to_string(),
            position(debut),
            position(fin),
            &marqueur.to_string(),
        );
        carte_du_plan(&plan)
    }

    pub fn prefixe(&self, texte: &QString, position_curseur: i32, prefixe: &QString) -> Carte {
        let plan = edition::basculer_prefixe(
            &texte.to_string(),
            position(position_curseur),
            &prefixe.to_string(),
        );
        carte_du_plan(&plan)
    }

    pub fn entourer(
        &self,
        texte: &QString,
        debut: i32,
        fin: i32,
        avant: &QString,
        apres: &QString,
    ) -> Carte {
        let plan = edition::encadrer(
            &texte.to_string(),
            position(debut),
            position(fin),
            &avant.to_string(),
            &apres.to_string(),
        );
        carte_du_plan(&plan)
    }

    pub fn chercher(
        &self,
        texte: &QString,
        motif: &QString,
        depuis: i32,
        casse: bool,
        mot_entier: bool,
        en_arriere: bool,
    ) -> i32 {
        edition::chercher(
            &texte.to_string(),
            &motif.to_string(),
            position(depuis),
            casse,
            mot_entier,
            en_arriere,
        )
        .map_or(-1, |i| i as i32)
    }

    pub fn remplacer_tout(
        &self,
        texte: &QString,
        motif: &QString,
        remplacement: &QString,
        casse: bool,
        mot_entier: bool,
    ) -> Carte {
        let (resultat, compte) = edition::remplacer_tout(
            &texte.to_string(),
            &motif.to_string(),
            &remplacement.to_string(),
            casse,
            mot_entier,
        );
        let mut carte = Carte::default();
        carte.insert(
            QString::from("texte"),
            QVariant::from(&QString::from(&resultat)),
        );
        carte.insert(QString::from("compte"), QVariant::from(&(compte as i32)));
        carte
    }
}
