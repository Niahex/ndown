# Suivi du Projet Ndown

## Architecture Générale

### Structure de l'Application (`app.rs`)
- **Type**: Point d'entrée principal de l'application
- **Composants**:
  - `left_sidebar`: FileExplorer (explorateur de fichiers)
  - `center`: Zone centrale contenant TopBar + EditorArea
  - `right_sidebar`: OutlinePanel (panneau de structure)
- **États**:
  - `left_visible`: bool - Visibilité de la sidebar gauche
  - `right_visible`: bool - Visibilité de la sidebar droite
- **Fichier initial**: `story.md` chargé au démarrage

### Gestion des Événements
- Toggle des sidebars (ouverture/fermeture)
- Sélection de fichiers depuis FileExplorer
- Mise à jour du titre dans TopBar lors du chargement de fichier
- Focus clavier automatique sur l'éditeur au démarrage

---

## File Explorer

### Fonctionnalités (`file_explorer/mod.rs`)
- **Chargement**: Lecture du répertoire courant au démarrage
- **Filtrage**: Ignore les fichiers cachés (commençant par `.`)
- **Tri**: Fichiers triés alphabétiquement
- **Affichage**: Liste scrollable avec icône 📄 et nom de fichier

### Actions
- `FileExplorerAction::FileSelected(String)`: Émis lors du clic sur un fichier
- Gestion via `handle_file_actions()` qui retourne `Option<String>`

### Interface
- **Header**: 
  - Titre "EXPLORATEUR" (THEME_FONT_BOLD, 12px, NORD_FROST_2)
  - Bouton toggle "☰" pour fermer la sidebar
- **Liste**: PortalList avec items cliquables
- **Dimensions**: 250px de largeur, Fill en hauteur
- **Style**: Fond NORD_POLAR_1, padding 10px

### Raccourcis Clavier
- Aucun raccourci clavier direct (navigation par clic uniquement)

---

## Editor

### Architecture Modulaire (`editor/mod.rs`)

#### Composants Principaux
- **EditorArea**: Widget principal de l'éditeur
- **Document**: Modèle de données (voir section Model)
- **EditorView**: Logique de rendu (voir section View)

#### Styles de Texte
- `draw_text_reg`: Texte régulier (12.1px)
- `draw_text_bold`: Texte gras (12.1px)
- `draw_text_italic`: Texte italique (12.1px)
- `draw_text_code`: Code inline (12.1px, NORD_AURORA_GREEN)
- `draw_text_header1`: Titre niveau 1 (29.0px, NORD_FROST_1)
- `draw_text_header2`: Titre niveau 2 (21.8px, NORD_FROST_2)
- `draw_text_header3`: Titre niveau 3 (19.4px, NORD_FROST_2)
- `draw_text_header4`: Titre niveau 4 (16.9px, NORD_FROST_2)
- `draw_text_header5`: Titre niveau 5 (14.5px, NORD_FROST_2)
- `draw_text_quote`: Citation (13.3px, NORD_AURORA_ORANGE, italique)

#### États de l'Éditeur
- `document`: Document - Contenu structuré en blocs
- `cursor_block`: usize - Index du bloc contenant le curseur
- `cursor_char`: usize - Position du caractère dans le bloc
- `selection_anchor`: Option<(usize, usize)> - Point d'ancrage de la sélection
- `is_dragging`: bool - État de glisser-déposer
- `blink_timer`: Timer - Animation du curseur
- `block_y_offsets`: Vec<f64> - Cache des positions Y des blocs
- `last_rendered_width`: f64 - Largeur précédente pour invalidation du layout

#### Fonctionnalités Principales

##### Chargement de Fichiers
- `load_file(cx, filename)`: Chargement synchrone
- Parsing automatique des marqueurs Markdown (#, ##, >, etc.)
- Réinitialisation du curseur à (0, 0)
- Invalidation du layout

##### Navigation par Mots
- `find_prev_word()`: Trouve le début du mot précédent
- `find_next_word()`: Trouve le début du mot suivant
- Utilisé avec Ctrl+Flèches

##### Gestion de la Sélection
- `get_selection_range()`: Retourne la plage ordonnée (start, end)
- Support du Shift pour étendre la sélection
- Wrap automatique avec `*`, `**`, `` ` `` si sélection active

##### Sauvegarde
- Sauvegarde asynchrone via `Ctrl+S`
- Utilise `document.snapshot()` pour copie thread-safe
- Sauvegarde dans `story.md` par défaut

### Raccourcis Clavier

#### Navigation
- **ArrowUp**: Bloc précédent
- **ArrowDown**: Bloc suivant
- **ArrowLeft**: Caractère précédent (ou bloc précédent si début)
- **ArrowRight**: Caractère suivant (ou bloc suivant si fin)
- **Ctrl+ArrowLeft**: Mot précédent
- **Ctrl+ArrowRight**: Mot suivant

#### Sélection
- **Shift+Flèches**: Étendre la sélection
- **Ctrl+A**: Sélectionner tout le bloc courant (contenu uniquement)

#### Édition
- **ReturnKey**: Nouveau bloc (Paragraph)
- **Backspace**: 
  - Supprimer caractère précédent
  - Fusionner avec bloc précédent si début de bloc
  - Convertir en Paragraph si bloc spécial (Heading, Quote)
  - Ctrl+Backspace: Supprimer mot précédent
- **Delete**: 
  - Supprimer caractère suivant
  - Fusionner avec bloc suivant si fin de bloc
  - Ctrl+Delete: Supprimer mot suivant

#### Fichier
- **Ctrl+S**: Sauvegarde asynchrone

#### Formatage Automatique
- **Ctrl+B**: Gras (wrap sélection ou insertion curseur)
- **Ctrl+I**: Italique (wrap sélection ou insertion curseur)
- Typing `# ` → Heading1
- Typing `## ` → Heading2
- Typing `### ` → Heading3
- Typing `#### ` → Heading4
- Typing `##### ` → Heading5
- Typing `> ` → Quote
- Typing `**texte**` → Gras
- Typing `*texte*` → Italique
- Typing `` `code` `` → Code inline (Fond grisé + Police monospace)
- Sélection + `*` ou `**` ou `` ` `` → Wrap automatique

---

## Editor Model

### Document (`editor/model/document.rs`)

#### Structure
- `blocks`: Vec<Block> - Liste des blocs de contenu
- `next_id`: u64 - Générateur d'ID unique
- `temp_markdown_buf`: String - Buffer de cache pour export
- `temp_char_buf`: Vec<char> - Buffer de cache pour parsing

#### Méthodes Principales

##### Gestion des Blocs
- `new()`: Document par défaut avec 3 blocs d'exemple
- `generate_id()`: Génère un ID unique incrémental
- `snapshot()`: Copie légère pour export asynchrone

##### Conversion Markdown
- `try_convert_block(block_idx)`: Détecte et convertit les préfixes (#, ##, >)
- `apply_inline_formatting(block_idx)`: Parse et applique `**`, `*`, `` ` ``
- Retourne le nombre de caractères supprimés lors de la conversion

##### Édition de Texte
- `insert_text_at(block_idx, char_idx, text)`: Insertion avec mise à jour des styles
- `remove_char_at(block_idx, char_idx)`: Suppression avec nettoyage des spans vides
- `wrap_selection(block_idx, start, end, marker)`: Entoure la sélection avec un marqueur
- `merge_block_with_prev(block_idx)`: Fusionne deux blocs consécutifs
- `delete_range(start, end)`: Suppression multi-blocs

##### Sauvegarde
- `save_to_file(filename)`: Export Markdown avec streaming I/O
- Utilise `BufWriter` pour performance
- Reconstruit les marqueurs Markdown (#, ##, >, etc.)
- Appelle `block.write_markdown_to_writer()` pour chaque bloc

#### Algorithme de Formatage Inline
1. Conversion du bloc en Markdown brut
2. Parsing séquentiel avec états (bold, italic, code)
3. Détection des paires fermantes valides
4. Construction de nouveaux StyleSpans
5. Remplacement du texte et des styles si changement détecté

### Block (`editor/model/block.rs`)

#### Types de Blocs
```rust
enum BlockType {
    Paragraph,
    Heading1, Heading2, Heading3, Heading4, Heading5,
    Quote,
    CodeBlock,
}
```

#### Structure StyleBits
- `is_bold`: bool
- `is_italic`: bool
- `is_code`: bool

#### Structure StyleSpan
- `len`: usize - Nombre de caractères
- `style`: StyleBits - Styles appliqués

#### Structure Block
- `id`: u64 - Identifiant unique
- `ty`: BlockType - Type de bloc
- `text`: String - Contenu textuel brut (sans marqueurs)
- `styles`: Vec<StyleSpan> - Segments de style
- `layout_cache`: Option<BlockLayoutCache> - Cache de rendu
- `is_dirty`: bool - Indicateur de modification

#### Méthodes
- `new(id, ty, text)`: Constructeur avec style par défaut
- `text_len()`: Nombre de caractères (Unicode-aware)
- `full_text()`: Référence au texte brut
- `write_markdown_to(buf)`: Export Markdown dans un String
- `write_markdown_to_writer(writer)`: Export Markdown streaming (zero-copy)
- `to_markdown()`: Conversion complète en String
- `mark_dirty()`: Invalide le cache de layout

#### Cache de Layout
```rust
struct BlockLayoutCache {
    height: f64,
    width: f64,
}
```
- Évite le recalcul du layout si le bloc n'a pas changé
- Invalidé par `mark_dirty()`

---

## Editor View

### EditorView (`editor/view.rs`)

#### Structure
Contient des références mutables vers tous les DrawText et DrawColor nécessaires au rendu.

#### Méthodes Principales

##### `draw_document(cx, params) -> (f64, Option<HitResult>)`
Fonction centrale de rendu avec optimisations avancées.

**Paramètres** (via `DrawParams`):
- `doc`: &mut Document
- `layout`: &Layout
- `rect`: Rect - Zone de rendu
- `cursor`: (usize, usize) - Position du curseur
- `selection`: Option<((usize, usize), (usize, usize))>
- `finger_hit`: Option<DVec2> - Position du clic/tap
- `scroll`: DVec2 - Offset de scroll
- `y_offsets_cache`: &mut Vec<f64> - Cache des positions Y

**Retour**:
- `f64`: Hauteur totale du contenu
- `Option<HitResult>`: Position cliquée (block_idx, char_idx)

#### Optimisations de Rendu

##### 1. Cache des Positions Y
- Stocke la position Y de chaque bloc
- Permet le calcul du `start_block_idx` via `partition_point`
- Évite de parcourir les blocs hors écran en début de document

##### 2. Culling Vertical
- Détection des blocs au-dessus de l'écran (skip sans rendu)
- Détection des blocs en-dessous de l'écran (break early)
- Utilise `is_cache_ready` pour activer le culling

##### 3. Cache de Layout par Bloc
- `BlockLayoutCache` stocke height/width
- Réutilisé si `!block.is_dirty`
- Permet de skip le calcul de layout pour blocs non modifiés

##### 4. Rendu Conditionnel
- `should_draw`: Vérifie si le bloc est visible à l'écran
- Skip le rendu des DrawText/DrawSelection si hors écran
- Calcule quand même le layout pour maintenir le cache

#### Algorithme de Rendu

1. **Initialisation**
   - Calcul de `start_block_idx` via cache Y
   - Positionnement initial de `current_y`

2. **Boucle sur les Blocs Visibles**
   - Vérification du cache de layout
   - Culling vertical (skip si au-dessus, break si en-dessous)
   - Sélection du DrawText selon BlockType et StyleBits

3. **Rendu des Spans**
   - Layout du texte via `draw_text.layout()`
   - Calcul de la largeur et hauteur
   - Rendu de la sélection (rectangles colorés)
   - Détection des clics (hit testing)
   - Rendu du texte
   - Calcul de la position du curseur

4. **Finalisation**
   - Mise à jour du cache de layout si nécessaire
   - Rendu du curseur si dans le bloc courant
   - Hit testing pour les zones vides (fin de ligne)
   - Incrémentation de `current_y`

5. **Retour**
   - Hauteur totale calculée
   - Résultat du hit testing

#### Hit Testing
- Détection par span (rectangle de texte)
- Approximation par largeur moyenne de caractère
- Gestion des clics en fin de ligne
- Gestion des clics avant le début de ligne

---

## Top Bar

### Fonctionnalités (`top_bar/mod.rs`)
- **Titre Central**: Affiche le nom du fichier courant
- **Boutons Toggle**: 
  - `left_toggle`: Ouvre la sidebar gauche (visible si fermée)
  - `right_toggle`: Ouvre la sidebar droite (visible si fermée)
- **Layout**: Horizontal avec espacement automatique (Fill)

### Interface
- **Dimensions**: Fill en largeur, Fit en hauteur
- **Padding**: 10px sur tous les côtés
- **Fond**: NORD_POLAR_0
- **Titre**: THEME_FONT_BOLD, 14px, NORD_SNOW_2
- **Boutons**: 30x30px, texte "☰", NORD_SNOW_2

### Raccourcis Clavier
- Aucun raccourci clavier (contrôle via clics uniquement)

---

## Outline Panel

### Fonctionnalités (`outline_panel/mod.rs`)
- **Affichage**: Panneau de structure du document (actuellement statique)
- **Contenu**: Exemple de hiérarchie de titres
- **Header**: Titre "STRUCTURE" + bouton toggle

### Interface
- **Dimensions**: 250px de largeur, Fill en hauteur
- **Padding**: 10px
- **Fond**: NORD_POLAR_1
- **Header**:
  - Bouton toggle "☰" (30x30px)
  - Label "STRUCTURE" (THEME_FONT_BOLD, 12px, NORD_FROST_2)
- **Contenu**: Label avec exemple de structure (14px, NORD_SNOW_0)

### Raccourcis Clavier
- Aucun raccourci clavier

### État Actuel
- **Statique**: Affiche un contenu d'exemple fixe
- **À Implémenter**: 
  - Extraction automatique des titres du document
  - Navigation par clic vers les sections
  - Mise à jour dynamique lors de l'édition

---

## Thème

### Palette Nord (`theme.rs`)

#### Polar Night (Fonds Sombres)
- `NORD_POLAR_0`: #2E3440 - Fond principal
- `NORD_POLAR_1`: #3B4252 - Fond containers
- `NORD_POLAR_2`: #434C5E
- `NORD_POLAR_3`: #4C566A - Sélection

#### Snow Storm (Textes Clairs)
- `NORD_SNOW_0`: #D8DEE9 - Texte muted
- `NORD_SNOW_1`: #E5E9F0
- `NORD_SNOW_2`: #ECEFF4 - Texte principal

#### Frost (Accents Bleus/Cyan)
- `NORD_FROST_0`: #8FBCBB
- `NORD_FROST_1`: #88C0D0 - Heading1, Accent
- `NORD_FROST_2`: #81A1C1 - Headings 2-5, Labels
- `NORD_FROST_3`: #5E81AC

#### Aurora (Couleurs Vives)
- `NORD_AURORA_RED`: #BF616A
- `NORD_AURORA_ORANGE`: #D08770 - Quotes
- `NORD_AURORA_YELLOW`: #EBCB8B
- `NORD_AURORA_GREEN`: #A3BE8C - Code inline
- `NORD_AURORA_PURPLE`: #B48EAD

#### Couleurs Personnalisées
- `COLOR_MUTE`: #3B425266 (Polar1 @ 40% opacity)
- `COLOR_ACCENT`: #88C0D040 (Frost1 @ 25% opacity)

### Overrides de Thème
- `THEME_COLOR_BG_APP`: NORD_POLAR_0
- `THEME_COLOR_BG_CONTAINER`: NORD_POLAR_1
- `THEME_COLOR_TEXT_DEFAULT`: NORD_SNOW_2
- `THEME_COLOR_TEXT_MUTE`: NORD_SNOW_0
- `THEME_COLOR_ACCENT`: NORD_FROST_1

---

## Polices

### Fonts Utilisées
- **Regular**: UbuntuNerdFont-Regular.ttf
- **Bold**: UbuntuNerdFont-Bold.ttf
- **Italic**: UbuntuNerdFont-Italic.ttf
- **Code**: UbuntuSansMonoNerdFont-Regular.ttf

### Styles de Police
- `THEME_FONT_REGULAR`: Police par défaut
- `THEME_FONT_BOLD`: Titres et labels importants
- `THEME_FONT_ITALIC`: Citations et emphase
- `THEME_FONT_CODE`: Code inline et blocs de code

---

## Animations

### Curseur Clignotant (`editor/mod.rs`)
- **États**: `on` (visible) / `off` (transparent)
- **Durée**: 0.1s de transition
- **Timer**: 0.5s entre chaque bascule
- **Couleurs**:
  - On: #ffffff (blanc opaque)
  - Off: #ffffff00 (blanc transparent)
- **Reset**: À chaque interaction clavier ou souris

---

## Optimisations Mémoire

### Document
- **Buffers Réutilisables**: 
  - `temp_markdown_buf`: Évite les allocations lors du formatage
  - `temp_char_buf`: Évite les allocations lors du parsing
- **Snapshot Léger**: Copie sans les buffers temporaires pour export async

### Block
- **Cache de Layout**: Évite le recalcul si non modifié
- **Streaming I/O**: Export Markdown sans allocation intermédiaire
- **Encode UTF-8**: Buffer stack de 4 bytes pour écriture char par char

### EditorView
- **Cache des Positions Y**: Évite le parcours complet lors du scroll
- **Culling Vertical**: Skip les blocs hors écran
- **Layout Conditionnel**: Réutilise le cache si `!is_dirty`

---

## Gestion des Erreurs

### Chargement de Fichiers
- `std::fs::read_to_string()`: Retourne `Result<String, io::Error>`
- Gestion silencieuse des erreurs (pas de feedback utilisateur actuellement)

### Sauvegarde
- Sauvegarde asynchrone avec log des erreurs
- Messages via `makepad_widgets::log!()`
- Pas de blocage de l'UI

---

## Points d'Amélioration Identifiés

### File Explorer
- [ ] Support des sous-dossiers (navigation hiérarchique)
- [ ] Icônes différenciées (dossiers vs fichiers)
- [ ] Filtrage par extension
- [ ] Création/suppression de fichiers

### Editor
- [ ] Undo/Redo
- [ ] Recherche et remplacement
- [ ] Support des listes (-, *, 1.)
- [ ] Support des liens Markdown
- [ ] Support des images
- [ ] Numérotation des lignes
- [ ] Minimap
- [ ] Multi-curseurs

### Outline Panel
- [ ] Extraction automatique des titres
- [ ] Navigation par clic
- [ ] Mise à jour en temps réel
- [ ] Indicateur de position actuelle

### Top Bar
- [ ] Indicateur de sauvegarde
- [ ] Breadcrumb de navigation
- [ ] Boutons d'action rapide (save, export, etc.)

### Général
- [ ] Gestion des erreurs avec feedback utilisateur
- [ ] Support multi-fichiers (tabs)
- [ ] Préférences utilisateur
- [ ] Export HTML/PDF
- [ ] Mode prévisualisation
- [ ] Thèmes personnalisables

---

## Dépendances Principales

### Makepad
- `makepad_widgets`: Framework UI
- Widgets utilisés:
  - `Window`, `View`, `Label`, `Button`
  - `PortalList`, `ScrollBars`
  - `DrawText`, `DrawColor`
  - `Cx`, `Cx2d`, `Event`, `Scope`

### Standard Library
- `std::fs`: Lecture/écriture de fichiers
- `std::io`: Streaming I/O
- `std::thread`: Sauvegarde asynchrone
- `std::path::PathBuf`: Manipulation de chemins

---

## Conventions de Code

### Naming
- **Modules**: snake_case (`file_explorer`, `editor`)
- **Structs**: PascalCase (`EditorArea`, `Document`)
- **Fonctions**: snake_case (`load_file`, `apply_inline_formatting`)
- **Constantes**: SCREAMING_SNAKE_CASE (`NORD_POLAR_0`)

### Organisation
- **Modules**: Un fichier `mod.rs` par composant principal
- **Sous-modules**: Dossier dédié (`editor/model/`)
- **Live Design**: Déclaration en début de fichier
- **Implémentations**: Ordre standard (LiveHook, Widget, méthodes custom)

### Commentaires
- Commentaires en français dans le code
- Documentation des algorithmes complexes
- TODOs explicites pour les améliorations futures
