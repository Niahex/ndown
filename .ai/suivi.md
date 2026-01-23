# Suivi du développement de ndown

## État actuel
- ✅ Structure de base de l'application (Makepad)
- ✅ Intégration initiale de l'interface (TopBar, Sidebars, Editor)
- ✅ **Éditeur de texte "Custom" fonctionnel** :
    - Saisie, Suppression, Sélection, Copier/Couper, Undo/Redo.
- ✅ **Rendu Markdown Riche** :
    - Parser (Titres, Gras, Italique, Code).
    - **Styles de police réels** (Gras, Italique, Headers).
    - Positionnement précis du curseur avec polices variables.

## Prochaines étapes
1.  **Rendu WYSIWYG (What You See Is What You Get)** :
    - Masquer les marqueurs Markdown (`#`, `**`, `*`, `` ` ``) au rendu.
    - Gérer la correspondance complexe entre curseur logique (texte brut) et visuel (texte rendu).
2.  **Scrolling** : Gérer le défilement vertical.
3.  **Gestion de fichiers** : Charger et sauvegarder.
