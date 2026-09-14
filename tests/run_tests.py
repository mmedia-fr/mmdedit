# MMdedit — lecteur / editeur Markdown simple.
# Copyright (C) 2026 M-Media
#
# Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le
# modifier selon les termes de la Licence Publique Generale GNU telle que
# publiee par la Free Software Foundation, soit la version 3 de la Licence,
# soit (a votre choix) toute version ulterieure.
#
# Ce programme est distribue dans l'espoir qu'il sera utile, mais SANS AUCUNE
# GARANTIE, sans meme la garantie implicite de QUALITE MARCHANDE ou
# D'ADEQUATION A UN USAGE PARTICULIER. Voyez la Licence Publique Generale GNU
# pour plus de details.
#
# Vous devriez avoir recu une copie de la Licence Publique Generale GNU avec
# ce programme (fichier LICENSE). Sinon, voyez <https://www.gnu.org/licenses/>.

"""Lance les vérifications fonctionnelles de MMdedit.

    python tests\run_tests.py

Les tests pilotent une vraie fenêtre Qt en mode « offscreen » : aucune fenêtre
n'apparaît, mais les widgets, la coloration syntaxique, l'aperçu et le
presse-papier sont réellement exercés.

Ce ne sont pas des tests unitaires : ils vérifient le comportement observable
de l'application. Chaque script est autonome et sort en code 0 si tout passe.
"""

import os
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
PROJET = os.path.dirname(ICI)

SUITES = [
    ("test_mmdedit.py", "base : rendu, encodage, export PDF, bascule des vues"),
    ("test_bascules.py", "barre d'outils : bascules de mise en forme, icône"),
    ("test_formats.py", "formats XML/texte, aperçu, persistance de la sélection"),
    ("test_modifie.py", "pas de faux marquage « modifié » à l'ouverture"),
    ("test_copie_auto.py", "copie automatique, apercu, protection du presse-papier"),
    ("test_reglages.py", "reglages persistants : fenetre, options, apparence"),
]

# Suite à lancer à part : elle exige un vrai affichage (la coloration
# syntaxique ne s'exécute qu'au moment où le widget est peint, ce que le mode
# offscreen peut ne jamais déclencher).
#   python tests\test_modifie_reel.py
MANUELLES = [("test_modifie_reel.py", "faux marquage « modifié », en rendu réel")]


def main():
    env = dict(os.environ)
    env["PYTHONPATH"] = os.path.join(PROJET, "src")
    env["QT_QPA_PLATFORM"] = "offscreen"

    echecs = []
    for fichier, description in SUITES:
        print(f"\n=== {fichier} — {description} ===")
        r = subprocess.run(
            [sys.executable, os.path.join(ICI, fichier)], env=env, cwd=ICI
        )
        if r.returncode != 0:
            echecs.append(fichier)

    print("\n" + "=" * 60)
    if echecs:
        print(f"ECHEC : {', '.join(echecs)}")
        return 1
    print(f"Toutes les suites passent ({len(SUITES)} suites).")
    for fichier, description in MANUELLES:
        print(f"À lancer à part (rendu réel) : python tests\\{fichier} — {description}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
