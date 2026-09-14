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

"""Permet « python -m mmdedit »."""

from .mainwindow import main

if __name__ == "__main__":
    main()
