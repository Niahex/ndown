
## **Implementation Plan - Makepad Markdown Block Editor**

### **Problem Statement:**
Créer un éditeur markdown complet par blocs avec édition live utilisant Makepad. Lorsqu'un utilisateur édite un bloc, les blocs précédents se
rendent en markdown tandis que le bloc actif reste en mode édition. L'application doit supporter tous les éléments markdown standards et avancés,
avec une architecture modulaire basée sur des plugins, un système de fichiers avec tabs, et des fonctionnalités premium (sync cloud, encryption
AES).

### **Requirements:**

Navigation & Édition:
- Flèches directionnelles pour naviguer dans le texte ET entre blocs
- Enter = créer nouveau bloc
- Shift+Enter = retour à ligne dans le bloc
- Clic pour sélectionner un bloc
- Navigation verticale intelligente: flèche haut sur première ligne → bloc précédent (fin), flèche bas sur dernière ligne → bloc suivant (début)
- Home/End = début/fin du bloc
- Ctrl+Home/End = début/fin du document
- Page Up/Down = page suivante/précédente

Éléments Markdown:
- Titres: # ## ### #### ##### (auto-expandables si contenu suit, style Obsidian)
- Listes à puces et numérotées avec:
  - Auto-continuation (Enter crée nouvel item)
  - Indentation (Tab/Shift+Tab pour sous-listes)
  - Conversion automatique (taper -  ou 1. )
  - Double Enter sort de la liste
  - Puce/numéro vide + Enter = retour texte normal
  - Changer de bloc avec puce vide = supprime le bloc
- Formatage inline: gras, italique, code
- Liens externes et internes ([[fichier]] style Obsidian + [texte](url) standard)
- Images (chemins locaux + URLs) avec preview, fit-to-width, syntaxe taille personnalisée, redimensionnables
- Code blocks avec détection langage + syntax highlighting (bibliothèque supportant nombreux langages)
- Tableaux avec édition visuelle:
  - Tab/Shift+Tab pour naviguer entre cellules
  - Enter = nouvelle rangée
  - Shift+Enter = nouvelle ligne dans cellule
  - Alt+Flèches = déplacer rangée/colonne
  - Ctrl+D = preview pour dupliquer avec placement
  - Delete/Backspace = preview pour supprimer
- Callouts Obsidian (> [!note], > [!warning], etc.) - basiques + étendus + personnalisables
- Citations (>) avec support multi-niveaux (3-5 niveaux max)
- Checkboxes (- [ ], - [x]) avec états personnalisables
- Dividers: --- (simple), === (double), -.- (pointillé)

Architecture:
- Système de plugins pour types de blocs (dynamic loading depuis fichiers externes)
- Parser markdown custom style Obsidian-like
- État performant et léger
- Tous les fichiers éditables via UI et config

Fonctionnalités Avancées:
- Undo/Redo
- Recherche (Ctrl+F) dans tous les blocs
- Raccourcis clavier:
  - Ctrl+B/I = wrap/toggle si sélection, insert markers si curseur seul
  - Autres raccourcis markdown standards
- Drag & drop de blocs avec ligne d'insertion bleue
- Virtualisation + lazy rendering pour performance

Gestionnaire de Fichiers:
- Tabs pour fichiers multiples
- Gestionnaire à gauche avec:
  - Arborescence navigable
  - Créer/renommer/supprimer fichiers et dossiers
  - Fuzzy search (Ctrl+P + champ visible)
- Liens internes avec preview popup au hover, clic pour ouvrir

UI & Thèmes:
- Panneaux configurables, redimensionnables, déplaçables (docking system style VS Code)
- Expand/collapse des panneaux
- Outline/table des matières (optionnel): auto-scroll, cliquable, synchronisation bidirectionnelle, filtrable par niveau
- Système de thèmes: presets prédéfinis + overrides personnalisables (fichiers JSON)

Persistance & Sécurité:
- Sauvegarde fichiers .md standard
- Auto-save (5s par défaut, configurable) + manuel (Ctrl+S)
- Encryption AES par défaut: création compte au premier lancement, mot de passe global
- Permissions par fichier (pour collaboration future)

Features Premium (à long terme):
- Sync cloud (plugin premium) - feature payante
- Backend custom pour auth
- Cloud provider (S3, Azure, etc.) pour stockage
- Collaboration temps réel style Google Docs
- Export PDF, HTML, Markdown, eBook

Plateformes:
- Linux, Android, Web (possiblement)

### **Background:**

Makepad Framework:
Makepad est un framework Rust pour créer des applications UI performantes et cross-platform. Il utilise un système de rendu GPU et un modèle de
composants réactifs.

Architecture Proposée:
1. Block System: Système de plugins modulaire où chaque type de bloc (Heading, List, Table, etc.) est un plugin chargeable dynamiquement
2. Parser: Parser markdown custom léger inspiré d'Obsidian, optimisé pour l'édition par blocs
3. State Management: Architecture simple et performante avec Vec de blocs et index du bloc actif
4. Rendering: Virtualisation pour rendre uniquement les blocs visibles + lazy loading
5. Storage: Abstraction storage (local/cloud) dès le début pour faciliter le plugin sync premium

### **Proposed Solution:**

Développer l'application en phases incrémentales, en commençant par les éléments markdown de base et l'architecture de blocs, puis en ajoutant
progressivement les fonctionnalités avancées. Chaque tâche produira un incrément fonctionnel et démontrable.

Architecture technique:
- **Core**: Gestion des blocs, navigation, état
- **Plugin System**: Registry de plugins de blocs avec API claire
- **Parser**: Module de parsing markdown par bloc
- **Renderer**: Système de rendu avec virtualisation
- **Storage**: Abstraction pour fichiers locaux (et cloud futur)
- **UI**: Composants Makepad pour l'éditeur, gestionnaire fichiers, panneaux

### **Task Breakdown:**

✅ Task 1: Setup projet Makepad et structure de base
- ✅ Créer projet Makepad avec dépendances nécessaires
- ✅ Définir structure de dossiers (core, plugins, parser, ui, storage)
- ✅ Créer application Makepad minimale avec fenêtre vide
- ✅ Setup système de build pour Linux
- **Tests**: ✅ Application compile et lance une fenêtre vide
- **Demo**: ✅ Fenêtre Makepad s'ouvre et affiche un fond uni

Task 2: Modèle de données pour blocs et état de base
- Définir struct Block avec id, type, contenu, métadonnées
- Créer enum BlockType pour types de base (Text, Heading)
- Implémenter struct EditorState avec Vec de blocs et index bloc actif
- Ajouter méthodes de base: créer bloc, supprimer bloc, naviguer entre blocs
- **Tests**: Créer/supprimer blocs, changer bloc actif
- **Demo**: État en mémoire avec quelques blocs, affichage debug dans console

Task 3: Affichage simple de blocs en mode texte
- Créer composant Makepad BlockView pour afficher un bloc
- Implémenter rendu basique: bloc actif avec fond différent, autres blocs en texte simple
- Afficher liste verticale de blocs
- **Tests**: Afficher 3-5 blocs, identifier visuellement le bloc actif
- **Demo**: Interface montrant plusieurs blocs texte, un surligné comme actif

Task 4: Input clavier de base et création de blocs
- Capturer événements clavier dans Makepad
- Implémenter édition texte dans bloc actif (saisie caractères, backspace)
- Implémenter Enter pour créer nouveau bloc
- Implémenter Shift+Enter pour retour à ligne dans bloc
- **Tests**: Taper du texte, créer nouveaux blocs avec Enter
- **Demo**: Éditeur fonctionnel où on peut taper et créer des blocs

Task 5: Navigation au clavier entre blocs
- Implémenter flèches haut/bas pour navigation inter-blocs (quand sur première/dernière ligne)
- Implémenter Home/End pour début/fin de bloc
- Implémenter Ctrl+Home/End pour début/fin document
- Implémenter Page Up/Down
- Gérer position curseur lors des transitions entre blocs
- **Tests**: Naviguer entre blocs avec toutes les touches
- **Demo**: Navigation fluide au clavier dans un document de 10+ blocs

Task 6: Clic souris pour sélection de blocs
- Capturer événements souris sur blocs
- Changer bloc actif au clic
- Positionner curseur à l'endroit du clic dans le texte
- **Tests**: Cliquer sur différents blocs, vérifier changement d'état
- **Demo**: Navigation mixte clavier + souris fonctionnelle

Task 7: Architecture plugin system - Registry et API
- Définir trait BlockPlugin avec méthodes: parse, render, edit, handle_input
- Créer PluginRegistry pour enregistrer et récupérer plugins
- Implémenter chargement statique de plugins (dynamic loading plus tard)
- Créer plugin TextBlockPlugin de base
- **Tests**: Enregistrer plugin, récupérer plugin par type
- **Demo**: Système de plugins fonctionnel avec plugin texte

Task 8: Plugin Heading avec parsing basique
- Créer HeadingBlockPlugin pour titres # à #####
- Implémenter parsing: détecter # au début, extraire niveau et texte
- Implémenter rendu: tailles de police différentes par niveau
- Conversion automatique: taper #  transforme bloc en heading
- **Tests**: Créer headings de différents niveaux, vérifier parsing et rendu
- **Demo**: Éditeur avec titres de différentes tailles visuellement distincts

Task 9: Parser markdown inline - gras, italique, code
- Créer module parser pour formatage inline
- Implémenter détection **gras**, *italique*,  code
- Intégrer rendu inline dans BlockView
- **Tests**: Parser différentes combinaisons de formatage
- **Demo**: Texte avec formatage inline rendu correctement

Task 10: Raccourcis clavier pour formatage (Ctrl+B, Ctrl+I)
- Implémenter Ctrl+B: wrap sélection avec **, toggle si déjà formaté, insert si curseur seul
- Implémenter Ctrl+I: même logique avec *
- Implémenter Ctrl+
 pour code inline
- **Tests**: Tester wrap, toggle, insert pour chaque raccourci
- **Demo**: Formatage rapide au clavier fonctionnel

**Task 11: Plugin List - listes à puces basiques**
- Créer
ListBlockPlugin pour listes à puces
- Parsing: détecter -  au début
- Conversion automatique: taper -
 transforme en liste
- Auto-continuation: Enter dans liste crée nouvel item
- Double Enter sort de la liste
- Puce vide + Enter = retour texte normal
- **Tests**: Créer listes, auto-continuation, sortie de liste
- **Demo**: Listes à puces fonctionnelles avec comportements intelligents

**Task 12: Listes numérotées et indentation**
- Ajouter support listes numérotées (
1. , 2.
, etc.)
- Implémenter Tab/Shift+Tab pour indentation (sous-listes)
- Gérer numérotation automatique
- Comportement puce/numéro vide identique aux puces
- **Tests**: Listes numérotées, indentation multi-niveaux
- **Demo**: Listes complexes avec sous-listes fonctionnelles

**Task 13: Système de sauvegarde fichiers markdown**
- Créer module
Storage avec trait pour abstraction
- Implémenter LocalStorage
 pour fichiers .md
- Sérialiser blocs en markdown standard
- Parser fichier .md en blocs au chargement
- Implémenter sauvegarde manuelle (Ctrl+S)
- **Tests**: Sauvegarder document, recharger, vérifier intégrité
- **Demo**: Éditer, sauvegarder, fermer, rouvrir - contenu préservé

**Task 14: Auto-save avec debounce**
- Implémenter timer avec debounce (5s par défaut)
- Déclencher sauvegarde automatique après modifications
- Ajouter indicateur visuel (sauvegarde en cours, dernière sauvegarde)
- **Tests**: Vérifier debounce, pas de sauvegarde excessive
- **Demo**: Éditer sans sauvegarder manuellement, fichier mis à jour automatiquement

**Task 15: Gestionnaire de fichiers - UI de base**
- Créer composant
FileExplorer
 pour panneau gauche
- Afficher arborescence de dossiers/fichiers
- Implémenter navigation dans dossiers
- Clic sur fichier charge le fichier dans l'éditeur
- **Tests**: Naviguer dans arborescence, ouvrir différents fichiers
- **Demo**: Panneau fichiers fonctionnel avec navigation

**Task 16: Système de tabs pour fichiers multiples**
- Créer composant
TabBar
 pour onglets
- Gérer liste de fichiers ouverts
- Clic sur tab change de fichier actif
- Bouton fermeture sur tabs
- Indicateur de modifications non sauvegardées
- **Tests**: Ouvrir plusieurs fichiers, naviguer entre tabs, fermer tabs
- **Demo**: Éditeur multi-fichiers avec tabs fonctionnels

**Task 17: Opérations fichiers - créer, renommer, supprimer**
- Ajouter menu contextuel dans
FileExplorer

- Implémenter création fichier/dossier
- Implémenter renommage
- Implémenter suppression (avec confirmation)
- **Tests**: Toutes opérations CRUD sur fichiers/dossiers
- **Demo**: Gestion complète de fichiers depuis l'UI

**Task 18: Fuzzy search pour fichiers**
- Créer composant
CommandPalette
 pour recherche
- Implémenter algorithme fuzzy matching
- Raccourci Ctrl+P ouvre la palette
- Champ de recherche aussi visible dans FileExplorer

- **Tests**: Rechercher fichiers avec patterns variés
- **Demo**: Recherche rapide de fichiers fonctionnelle

**Task 19: Undo/Redo système**
- Implémenter stack d'historique des modifications
- Capturer changements de blocs comme actions
- Implémenter Ctrl+Z (undo) et Ctrl+Y (redo)
- Gérer limite d'historique (configurable)
- **Tests**: Séquences undo/redo complexes
- **Demo**: Éditer, undo plusieurs fois, redo - état correct

**Task 20: Recherche globale (Ctrl+F)**
- Créer composant
SearchPanel

- Implémenter recherche dans tous les blocs
- Afficher résultats avec contexte
- Navigation entre résultats (Enter, Shift+Enter)
- Highlight des résultats dans l'éditeur
- **Tests**: Rechercher patterns, naviguer entre résultats
- **Demo**: Recherche fonctionnelle avec navigation

**Task 21: Drag & drop de blocs**
- Capturer événements drag sur blocs
- Afficher ligne d'insertion bleue pendant drag
- Réorganiser blocs au drop
- Gérer edge cases (drag sur soi-même, limites document)
- **Tests**: Drag & drop dans différentes positions
- **Demo**: Réorganisation de blocs par drag & drop

**Task 22: Virtualisation pour performance**
- Implémenter calcul de blocs visibles dans viewport
- Rendre uniquement blocs visibles + buffer
- Lazy loading au scroll
- Gérer hauteurs variables de blocs
- **Tests**: Document avec 1000+ blocs, vérifier performance
- **Demo**: Scroll fluide dans très grand document

**Task 23: Plugin Divider**
- Créer
DividerBlockPlugin
- Parser: --- (simple), === (double), -.-
 (pointillé)
- Rendu: lignes horizontales avec styles différents
- **Tests**: Créer différents types de dividers
- **Demo**: Document avec dividers visuellement distincts

**Task 24: Plugin Blockquote (citations)**
- Créer
BlockquotePlugin
- Parser: détecter >
 au début, support multi-niveaux (max 3-5)
- Rendu: indentation et style visuel par niveau
- **Tests**: Citations simples et imbriquées
- **Demo**: Citations avec plusieurs niveaux de profondeur

**Task 25: Plugin Link - liens externes et internes**
- Créer
LinkPlugin pour parsing inline
- Support [texte](url) et [[fichier]]
- Rendu: liens cliquables avec style
- Hover sur [[]]
 montre preview popup
- Clic ouvre fichier (interne) ou URL (externe)
- **Tests**: Différents types de liens, preview, navigation
- **Demo**: Liens fonctionnels avec preview

**Task 26: Plugin Checkbox pour todo lists**
- Créer
CheckboxPlugin
- Parser: - [ ] (non coché), - [x]
 (coché)
- Rendu: checkbox cliquable
- Toggle état au clic
- **Tests**: Créer todos, cocher/décocher
- **Demo**: Todo list interactive fonctionnelle

**Task 27: Checkboxes - états personnalisables**
- Créer fichier config JSON pour états custom
- Parser états additionnels depuis config
- UI pour éditer états (Settings panel)
- **Tests**: Ajouter états custom, utiliser dans document
- **Demo**: Checkboxes avec états personnalisés (en attente, urgent, etc.)

**Task 28: Plugin Callout Obsidian**
- Créer
CalloutPlugin
- Parser: > [!type]
 avec types basiques (note, tip, warning, danger)
- Rendu: blocs colorés avec icônes
- Support types étendus (info, success, question, etc.)
- **Tests**: Différents types de callouts
- **Demo**: Document avec callouts visuellement distincts

**Task 29: Callouts personnalisables**
- Fichier config JSON pour callouts custom
- UI pour créer/éditer types de callouts
- Définir couleur, icône, label
- **Tests**: Créer callout custom, utiliser dans document
- **Demo**: Callouts personnalisés fonctionnels

**Task 30: Plugin Heading - expand/collapse**
- Ajouter état expand/collapse aux headings
- Auto-détection: heading devient expandable si contenu suit
- Icône toggle à côté du heading
- Cacher/montrer blocs enfants au toggle
- Persister état dans métadonnées
- **Tests**: Expand/collapse sections, vérifier persistance
- **Demo**: Document avec sections collapsables style Obsidian

**Task 31: Plugin Table - structure de base**
- Créer
TablePlugin
- Parser syntaxe markdown table avec |

- Structure de données: grille de cellules
- Rendu basique de table
- **Tests**: Parser différentes tables, afficher correctement
- **Demo**: Tables markdown rendues visuellement

**Task 32: Table - mode édition visuel**
- Implémenter mode édition spécial pour tables
- Tab/Shift+Tab pour navigation entre cellules
- Enter = nouvelle rangée
- Shift+Enter = nouvelle ligne dans cellule
- Édition inline du contenu des cellules
- **Tests**: Navigation et édition dans tables
- **Demo**: Édition de table fluide au clavier

**Task 33: Table - manipulation rangées/colonnes**
- Implémenter preview overlay pour Ctrl+D (dupliquer)
- Navigation par flèches dans preview pour choisir position
- Même système pour Delete/Backspace (supprimer)
- Alt+Flèches pour déplacer rangée/colonne
- **Tests**: Toutes opérations de manipulation
- **Demo**: Manipulation complète de tables

**Task 34: Plugin Image - parsing et affichage**
- Créer
ImagePlugin
- Parser: ![alt](path) et ![alt](url)

- Charger images locales et distantes
- Rendu: fit-to-width par défaut
- **Tests**: Afficher images locales et URLs
- **Demo**: Document avec images affichées

**Task 35: Image - taille personnalisée et redimensionnement**
- Parser syntaxe
!alt path{width=500}

- Implémenter poignées de redimensionnement visuel
- Sauvegarder taille dans syntaxe markdown
- **Tests**: Redimensionner images, vérifier persistance
- **Demo**: Images redimensionnables interactivement

**Task 36: Plugin CodeBlock - structure de base**
- Créer
CodeBlockPlugin
- Parser: blocs  lang  avec détection langage
- Rendu: fond différent, police monospace
- Afficher label du langage
- **Tests**: Code blocks avec différents langages
- **Demo**: Code blocks visuellement distincts avec labels

Task 37: CodeBlock - syntax highlighting
- Intégrer bibliothèque syntax highlighting (syntect ou similaire)
- Implémenter coloration pour langages populaires
- Optimiser performance (cache, lazy highlighting)
- **Tests**: Highlighting pour JavaScript, Python, Rust, etc.
- **Demo**: Code coloré syntaxiquement

Task 38: Système de panneaux configurables - docking
- Implémenter docking system style VS Code
- Zones: gauche (fichiers), centre (éditeur), droite (outline optionnel)
- Panneaux redimensionnables avec splitters
- Expand/collapse des panneaux
- **Tests**: Redimensionner, expand/collapse panneaux
- **Demo**: Layout flexible et configurable

Task 39: Outline / Table des matières
- Créer composant OutlinePanel pour panneau droit
- Extraire tous les headings du document
- Afficher arborescence hiérarchique
- Clic sur heading scroll vers ce heading
- Auto-scroll: synchroniser avec position dans éditeur
- **Tests**: Navigation bidirectionnelle éditeur ↔ outline
- **Demo**: Outline synchronisé fonctionnel

Task 40: Outline - filtrage par niveau
- Ajouter contrôles pour filtrer niveaux de headings
- Checkboxes ou slider pour niveaux 1-5
- Mise à jour dynamique de l'arborescence
- **Tests**: Filtrer différents niveaux, vérifier affichage
- **Demo**: Outline filtrable pour navigation ciblée

Task 41: Système de thèmes - infrastructure
- Définir structure JSON pour thèmes
- Créer ThemeManager pour charger/appliquer thèmes
- Thème par défaut (light)
- Appliquer couleurs aux composants
- **Tests**: Charger thème, vérifier application des couleurs
- **Demo**: Application avec thème cohérent

Task 42: Thèmes - presets et personnalisation
- Créer presets: light, dark, et 2-3 autres
- UI Settings pour sélectionner thème
- UI pour éditer thème (overrides)
- Sauvegarder thème custom
- **Tests**: Changer thèmes, éditer couleurs, vérifier persistance
- **Demo**: Personnalisation complète de l'apparence

Task 43: Configuration globale - fichier settings.json
- Créer fichier settings.json pour config app
- Paramètres: auto-save delay, thème, raccourcis, etc.
- UI Settings panel pour éditer config
- Tous les paramètres éditables via UI et fichier
- **Tests**: Modifier settings via UI et fichier, vérifier application
- **Demo**: Configuration complète de l'application

Task 44: Encryption AES - setup et création compte
- Intégrer bibliothèque crypto Rust (ring ou similaire)
- Écran de création compte au premier lancement
- Générer clé AES depuis mot de passe (PBKDF2)
- Stocker hash sécurisé pour vérification
- **Tests**: Créer compte, vérifier génération clé
- **Demo**: Onboarding avec création compte sécurisé

Task 45: Encryption - chiffrement fichiers
- Implémenter chiffrement/déchiffrement de fichiers .md
- Chiffrer avant sauvegarde, déchiffrer au chargement
- Gérer erreurs (mauvais mot de passe, corruption)
- **Tests**: Sauvegarder/charger fichiers chiffrés, vérifier intégrité
- **Demo**: Fichiers stockés chiffrés, lisibles uniquement avec mot de passe

Task 46: Plugin system - dynamic loading
- Implémenter chargement dynamique de plugins depuis fichiers
- Définir format de plugin (dylib/so)
- API stable pour plugins externes
- Dossier plugins/ pour plugins custom
- **Tests**: Charger plugin externe, vérifier fonctionnement
- **Demo**: Ajouter nouveau type de bloc via plugin externe

Task 47: Permissions fichiers (préparation collaboration)
- Ajouter métadonnées de permissions aux fichiers
- Structure: lecture, écriture, propriétaire
- UI pour gérer permissions (basique)
- Pas d'enforcement pour l'instant (infrastructure seulement)
- **Tests**: Définir permissions, vérifier stockage
- **Demo**: UI de gestion des permissions

Task 48: Abstraction storage pour cloud (préparation sync)
- Refactorer Storage trait pour abstraction complète
- Implémenter CloudStorage stub (non fonctionnel)
- Architecture pour plugin sync premium
- Détection présence plugin sync
- **Tests**: Basculer entre local et cloud storage (stub)
- **Demo**: Architecture prête pour plugin sync

Task 49: Polish UI et UX
- Animations et transitions fluides
- Feedback visuel (hover, focus, active states)
- Messages d'erreur clairs
- Loading states
- Tooltips et aide contextuelle
- **Tests**: Vérifier tous les états visuels
- **Demo**: Application polie et professionnelle

Task 50: Tests d'intégration et documentation
- Tests end-to-end des workflows principaux
- Documentation utilisateur (README, guide)
- Documentation développeur (architecture, plugins)
- Exemples de plugins custom
- **Tests**: Suite complète de tests d'intégration
- **Demo**: Application complète, testée, documentée, prête pour release
