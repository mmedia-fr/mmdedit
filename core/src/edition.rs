// SPDX-License-Identifier: GPL-3.0-or-later
//! Édition assistée : encadrements Markdown, préfixes de ligne, recherche.
//!
//! Ces fonctions ne touchent pas au texte : elles rendent un *plan* — quel
//! intervalle remplacer, par quoi, et quelle sélection laisser derrière. C'est
//! l'interface qui l'applique, par `remove()` puis `insert()` sur le TextArea,
//! de sorte que l'annulation (Ctrl+Z) continue de fonctionner pas à pas.
//!
//! **Les positions sont comptées comme Qt les compte** : en unités UTF-16, et
//! non en octets ni en caractères. Un document contenant un emoji décalerait
//! sinon toute la mise en forme.

/// Intervalle à remplacer, remplacement, et sélection à laisser ensuite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub debut: usize,
    pub fin: usize,
    pub remplacement: String,
    pub selection_debut: usize,
    pub selection_longueur: usize,
}

/// Préfixes de ligne reconnus, du plus long au plus court : « ### » doit être
/// essayé avant « ## » et « # », sinon un titre 3 se lit comme un titre 1.
const PREFIXES: &[&str] = &["### ", "## ", "# ", "- ", "1. ", "> "];

fn unites(texte: &str) -> Vec<u16> {
    texte.encode_utf16().collect()
}

fn chaine(unites: &[u16]) -> String {
    String::from_utf16_lossy(unites)
}

/// Compte les unités égales à `unite` à partir de `index`, dans le sens `pas`.
///
/// Sert à distinguer « * » (italique) d'un « * » appartenant à un « ** » (gras) :
/// sans ce comptage, appliquer l'italique sur du gras retirait un astérisque de
/// chaque côté au lieu d'imbriquer les deux styles.
fn repetitions(texte: &[u16], index: isize, unite: u16, pas: isize) -> usize {
    let mut n = 0;
    let mut i = index;
    while i >= 0 && (i as usize) < texte.len() && texte[i as usize] == unite {
        n += 1;
        i += pas;
    }
    n
}

/// Pose ou retire un encadrement (`**`, `*`, `~~`, `` ` ``) — bascule.
///
/// Trois cas : les marqueurs sont dans la sélection (on retire), ils l'encadrent
/// juste à l'extérieur (on retire aussi), sinon on pose. Sans sélection, on pose
/// la paire et le curseur se place au milieu.
pub fn basculer_encadrement(texte: &str, debut: usize, fin: usize, marqueur: &str) -> Plan {
    let t = unites(texte);
    let m = unites(marqueur);
    let n = m.len();
    let (debut, fin) = (debut.min(t.len()), fin.min(t.len()));

    if debut == fin {
        // Pas de sélection : la paire est posée et le curseur vient au milieu.
        return Plan {
            debut,
            fin,
            remplacement: format!("{marqueur}{marqueur}"),
            selection_debut: debut + n,
            selection_longueur: 0,
        };
    }

    let selection = &t[debut..fin];
    let car = m[0];

    // Cas 1 : les marqueurs sont dans la sélection. Le nombre doit être EXACT,
    // sinon « * » sur « **mot** » retirerait un astérisque de chaque côté.
    if selection.len() >= 2 * n
        && selection[..n] == m[..]
        && selection[selection.len() - n..] == m[..]
        && repetitions(selection, 0, car, 1) == n
        && repetitions(selection, selection.len() as isize - 1, car, -1) == n
    {
        let interieur = &selection[n..selection.len() - n];
        return Plan {
            debut,
            fin,
            remplacement: chaine(interieur),
            selection_debut: debut,
            selection_longueur: interieur.len(),
        };
    }

    // Cas 2 : les marqueurs encadrent la sélection, à l'extérieur — même exigence.
    if debut >= n
        && fin + n <= t.len()
        && t[debut - n..debut] == m[..]
        && t[fin..fin + n] == m[..]
        && repetitions(&t, debut as isize - 1, car, -1) == n
        && repetitions(&t, fin as isize, car, 1) == n
    {
        return Plan {
            debut: debut - n,
            fin: fin + n,
            remplacement: chaine(selection),
            selection_debut: debut - n,
            selection_longueur: selection.len(),
        };
    }

    // Cas 3 : pose. Le texte reste sélectionné, marqueurs exclus, de sorte qu'un
    // second clic sur le même bouton annule le style.
    Plan {
        debut,
        fin,
        remplacement: format!("{marqueur}{}{marqueur}", chaine(selection)),
        selection_debut: debut + n,
        selection_longueur: selection.len(),
    }
}

/// Encadre la sélection par deux textes libres (lien, image).
pub fn encadrer(texte: &str, debut: usize, fin: usize, avant: &str, apres: &str) -> Plan {
    let t = unites(texte);
    let (debut, fin) = (debut.min(t.len()), fin.min(t.len()));
    let selection = chaine(&t[debut..fin]);
    let n = unites(avant).len();
    Plan {
        debut,
        fin,
        remplacement: format!("{avant}{selection}{apres}"),
        selection_debut: debut + n,
        selection_longueur: unites(&selection).len(),
    }
}

/// Début et fin (hors saut de ligne) de la ligne qui contient `position`.
fn bornes_ligne(t: &[u16], position: usize) -> (usize, usize) {
    const LF: u16 = b'\n' as u16;
    let position = position.min(t.len());
    let debut = t[..position].iter().rposition(|u| *u == LF).map_or(0, |i| i + 1);
    let fin = t[position..]
        .iter()
        .position(|u| *u == LF)
        .map_or(t.len(), |i| position + i);
    (debut, fin)
}

/// Pose, remplace ou retire un préfixe de ligne — bascule.
///
/// Recliquer sur le même préfixe le retire ; passer d'un niveau de titre à un
/// autre (ou d'une liste à une citation) remplace le préfixe existant.
pub fn basculer_prefixe(texte: &str, position: usize, prefixe: &str) -> Plan {
    let t = unites(texte);
    let (debut, fin) = bornes_ligne(&t, position);
    let ligne = chaine(&t[debut..fin]);

    let actuel = PREFIXES.iter().find(|p| ligne.starts_with(**p));
    let nouvelle = match actuel {
        Some(p) if *p == prefixe => ligne[p.len()..].to_string(),
        Some(p) => format!("{prefixe}{}", &ligne[p.len()..]),
        None => format!("{prefixe}{ligne}"),
    };
    let longueur = unites(&nouvelle).len();
    Plan {
        debut,
        fin,
        remplacement: nouvelle,
        // Le curseur se retrouve en fin de ligne, comme dans la version PySide6.
        selection_debut: debut + longueur,
        selection_longueur: 0,
    }
}

/// Minuscule d'une unité UTF-16, pour les comparaisons insensibles à la casse.
///
/// Les demi-codets (caractères hors du plan de base, emojis) n'ont pas de casse
/// et sont comparés tels quels.
fn minuscule(u: u16) -> u16 {
    match char::from_u32(u as u32) {
        Some(c) => c.to_lowercase().next().map_or(u, |m| {
            let mut tampon = [0u16; 2];
            let encode = m.encode_utf16(&mut tampon);
            if encode.len() == 1 {
                encode[0]
            } else {
                u
            }
        }),
        None => u,
    }
}

fn egal(a: u16, b: u16, casse: bool) -> bool {
    if casse {
        a == b
    } else {
        minuscule(a) == minuscule(b)
    }
}

fn est_mot(u: u16) -> bool {
    char::from_u32(u as u32).map_or(false, |c| c.is_alphanumeric() || c == '_')
}

fn correspond(t: &[u16], motif: &[u16], i: usize, casse: bool, mot_entier: bool) -> bool {
    if i + motif.len() > t.len() {
        return false;
    }
    if !(0..motif.len()).all(|k| egal(t[i + k], motif[k], casse)) {
        return false;
    }
    if !mot_entier {
        return true;
    }
    let avant_ok = i == 0 || !est_mot(t[i - 1]);
    let apres = i + motif.len();
    let apres_ok = apres >= t.len() || !est_mot(t[apres]);
    avant_ok && apres_ok
}

/// Cherche `motif` à partir de `depuis`, en avant ou en arrière, et reboucle.
///
/// Rend la position de début de l'occurrence, ou `None` si le motif est absent
/// du document entier. Le rebouclage reproduit le dialogue de la version
/// PySide6, qui repart du début quand il atteint la fin.
pub fn chercher(
    texte: &str,
    motif: &str,
    depuis: usize,
    casse: bool,
    mot_entier: bool,
    en_arriere: bool,
) -> Option<usize> {
    let t = unites(texte);
    let m = unites(motif);
    if m.is_empty() || m.len() > t.len() {
        return None;
    }
    let dernier = t.len() - m.len();
    let essais: Vec<usize> = if en_arriere {
        let depart = depuis.min(t.len()).saturating_sub(1).min(dernier);
        (0..=depart).rev().chain((0..=dernier).rev()).collect()
    } else {
        let depart = depuis.min(dernier + 1);
        (depart..=dernier).chain(0..=dernier).collect()
    };
    essais
        .into_iter()
        .find(|i| correspond(&t, &m, *i, casse, mot_entier))
}

/// Remplace toutes les occurrences ; rend le texte obtenu et le nombre de remplacements.
pub fn remplacer_tout(
    texte: &str,
    motif: &str,
    remplacement: &str,
    casse: bool,
    mot_entier: bool,
) -> (String, usize) {
    let t = unites(texte);
    let m = unites(motif);
    if m.is_empty() {
        return (texte.to_string(), 0);
    }
    let r = unites(remplacement);
    let mut sortie: Vec<u16> = Vec::with_capacity(t.len());
    let mut compte = 0;
    let mut i = 0;
    while i < t.len() {
        if correspond(&t, &m, i, casse, mot_entier) {
            sortie.extend_from_slice(&r);
            i += m.len();
            compte += 1;
        } else {
            sortie.push(t[i]);
            i += 1;
        }
    }
    (chaine(&sortie), compte)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Applique un plan, pour vérifier le texte obtenu comme le ferait l'interface.
    fn appliquer(texte: &str, plan: &Plan) -> String {
        let t = unites(texte);
        let mut sortie = chaine(&t[..plan.debut]);
        sortie.push_str(&plan.remplacement);
        sortie.push_str(&chaine(&t[plan.fin..]));
        sortie
    }

    #[test]
    fn encadrement_pose_sur_selection() {
        let plan = basculer_encadrement("un mot ici", 3, 6, "**");
        assert_eq!(appliquer("un mot ici", &plan), "un **mot** ici");
        // Le texte reste sélectionné, marqueurs exclus.
        assert_eq!((plan.selection_debut, plan.selection_longueur), (5, 3));
    }

    #[test]
    fn encadrement_retire_marqueurs_dans_la_selection() {
        let plan = basculer_encadrement("un **mot** ici", 3, 10, "**");
        assert_eq!(appliquer("un **mot** ici", &plan), "un mot ici");
        assert_eq!((plan.selection_debut, plan.selection_longueur), (3, 3));
    }

    #[test]
    fn encadrement_retire_marqueurs_hors_selection() {
        let plan = basculer_encadrement("un **mot** ici", 5, 8, "**");
        assert_eq!(appliquer("un **mot** ici", &plan), "un mot ici");
        assert_eq!((plan.selection_debut, plan.selection_longueur), (3, 3));
    }

    #[test]
    fn italique_sur_gras_imbrique_au_lieu_de_retirer() {
        // Le cas qui impose le comptage exact : « * » sur « **mot** » doit poser
        // une troisième paire, pas déshabiller le gras.
        let plan = basculer_encadrement("**mot**", 0, 7, "*");
        assert_eq!(appliquer("**mot**", &plan), "***mot***");
    }

    #[test]
    fn encadrement_sans_selection_place_le_curseur_au_milieu() {
        let plan = basculer_encadrement("ab", 1, 1, "**");
        assert_eq!(appliquer("ab", &plan), "a****b");
        assert_eq!((plan.selection_debut, plan.selection_longueur), (3, 0));
    }

    #[test]
    fn prefixe_pose_remplace_retire() {
        let pose = basculer_prefixe("titre", 2, "# ");
        assert_eq!(appliquer("titre", &pose), "# titre");

        let remplace = basculer_prefixe("# titre", 3, "## ");
        assert_eq!(appliquer("# titre", &remplace), "## titre");

        let retire = basculer_prefixe("## titre", 3, "## ");
        assert_eq!(appliquer("## titre", &retire), "titre");
    }

    #[test]
    fn prefixe_ne_touche_que_la_ligne_du_curseur() {
        let texte = "un\ndeux\ntrois";
        let plan = basculer_prefixe(texte, 4, "- ");
        assert_eq!(appliquer(texte, &plan), "un\n- deux\ntrois");
    }

    #[test]
    fn lien_et_image() {
        let plan = encadrer("voir ici", 5, 8, "[", "](https://)");
        assert_eq!(appliquer("voir ici", &plan), "voir [ici](https://)");
        assert_eq!((plan.selection_debut, plan.selection_longueur), (6, 3));
    }

    #[test]
    fn recherche_en_avant_et_rebouclage() {
        let texte = "alpha beta alpha";
        assert_eq!(chercher(texte, "alpha", 0, false, false, false), Some(0));
        assert_eq!(chercher(texte, "alpha", 1, false, false, false), Some(11));
        // Au-delà de la dernière occurrence, la recherche repart du début.
        assert_eq!(chercher(texte, "alpha", 12, false, false, false), Some(0));
        assert_eq!(chercher(texte, "gamma", 0, false, false, false), None);
    }

    #[test]
    fn recherche_en_arriere() {
        let texte = "alpha beta alpha";
        assert_eq!(chercher(texte, "alpha", 16, false, false, true), Some(11));
        assert_eq!(chercher(texte, "alpha", 11, false, false, true), Some(0));
    }

    #[test]
    fn recherche_casse_et_mot_entier() {
        assert_eq!(chercher("Alpha alpha", "alpha", 0, true, false, false), Some(6));
        assert_eq!(chercher("Alpha alpha", "alpha", 0, false, false, false), Some(0));
        // « bet » ne doit pas être trouvé dans « beta » en mot entier.
        assert_eq!(chercher("beta", "bet", 0, false, true, false), None);
        assert_eq!(chercher("bet beta", "bet", 0, false, true, false), Some(0));
    }

    #[test]
    fn remplacement_global() {
        let (texte, compte) = remplacer_tout("a b a", "a", "z", false, false);
        assert_eq!((texte.as_str(), compte), ("z b z", 2));
        let (texte, compte) = remplacer_tout("beta bet", "bet", "X", false, true);
        assert_eq!((texte.as_str(), compte), ("beta X", 1));
        // Motif vide : rien ne bouge, et surtout pas de boucle infinie.
        let (texte, compte) = remplacer_tout("abc", "", "z", false, false);
        assert_eq!((texte.as_str(), compte), ("abc", 0));
    }

    #[test]
    fn positions_comptees_comme_qt() {
        // « 🙂 » occupe deux unités UTF-16 : la sélection qui suit doit en tenir
        // compte, sinon l'encadrement se pose à côté.
        let texte = "🙂 mot";
        let plan = basculer_encadrement(texte, 3, 6, "**");
        assert_eq!(appliquer(texte, &plan), "🙂 **mot**");
    }
}
