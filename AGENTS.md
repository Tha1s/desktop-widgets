# AGENTS.md

## Environnement
- Le dépôt est stocké sur l'hôte Windows (`/mnt/c/Dev/projects/desktop-widgets`) mais le code est écrit depuis WSL. Garder les outils/comptes de chemins en conséquence ; les opérations git et les fichiers restent sur le disque Windows.
- L'app est Windows-only et ne se cross-compile pas proprement depuis WSL. Build/vérification du dev via `cargo.exe` invoqué depuis WSL (le dossier `/mnt/c` est visible par Windows). Prérequis sur l'hôte Windows : Rust (toolchain MSVC), MSVC Build Tools, Windows SDK, runtime WebView2.
- **Smart App Control (SAC/WDAC) est actif sur cette machine et bloque les build-scripts cargo** des crates courantes (anyhow, serde, proc-macro2, quote, typeid…) : erreur 4551 / « stratégie Device Guard », que cargo soit lancé depuis WSL ou via `cmd.exe`, quel que soit le dossier (target purgé, testé). Aucun build cargo complet n'est possible ici tant que SAC est actif. En revanche un exe local non signé **s'exécute** normalement (vérifié) : le problème est le BUILD, pas le RUN.
- Workflow imposé : **builder sur un hôte sans SAC** (runner GitHub Actions `windows-latest`, ou le PC de la copine), puis récupérer l'exe et l'exécuter ici.
- Les machines cibles (ex. PC sans WSL) exécutent le binaire en Windows natif — le dev WSL n'est pas requis chez elles.
- Frontend en HTML/CSS/JS vanilla (pas de Node/npm) — cohérent avec une webview par widget.

## État du projet
- Projet greenfield / phase conception uniquement. Aucun code, manifeste, ni commande de build/lint/test/typecheck n'existe encore. Ne pas les inventer.

## Ce que c'est
- "Desktop Hub" : moteur de widgets Windows (widgets flottants derrière les fenêtres normales, au-dessus du fond d'écran) + un drawer d'apps comme premier widget.
- Stack : Tauri (Rust + WebView2). Pas Electron. Windows-only pour l'instant.
- Séparé volontairement du projet "hub launcher" (Tauri façon Raycast) — aucun code ni dépendance partagé.

## Contexte canonique
- Le spec/design de référence vit dans `PROJECT_BRIEF.md` (français). Le lire avant de concevoir ou coder. Ce fichier est actuellement non-suivi (untracked).

## Architecture (issue du brief)
- Chaque widget = une webview isolée (HTML/CSS/JS propres). Le JS d'un widget ne doit pas accéder au FS/réseau directement — tout passe par `bridge/` via une API `invoke()` définie.
- Config/position persistées par id de widget (JSON individuel / clé dédiée), jamais une structure globale unique.
- Arborescence : `core/` (gestion fenêtres, positionnement WorkerW, persistance layouts, cycle de vie), `widgets/<nom>/`, `bridge/` (API invoke : FS limité, drag & drop, lancement d'apps).
- Widgets packagés dans le binaire unique pour l'instant, mais écrits comme si l'ouverture à des plugins externes viendrait plus tard.

## Jalon bloquant (à faire en premier)
- Valider le hack Windows `WorkerW` en Tauri : faire flotter une fenêtre transparente derrière les icônes / fenêtres normales, au-dessus du fond d'écran. Référence : trouver/créer le `WorkerW` d'`explorer.exe`, reparenter via `FindWindowEx`, `SendMessage` `0x052C`, `SetParent` (crate `windows` ou `winapi`). Même technique que Rainmeter.

## Pièges
- `.vscode/`, `PROJECT_BRIEF.md`, et `rapidsave.com_*.mp4` (~10 Mo de vidéo de référence) sont non-suivis. Éviter `git add .` — la vidéo est un binaire de référence, pas du code source.
