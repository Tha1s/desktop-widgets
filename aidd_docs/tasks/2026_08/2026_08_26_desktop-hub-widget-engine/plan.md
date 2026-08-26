---
objective: "Le moteur Desktop Hub flotte des widgets Tauri sous le bureau Windows, chacun dans une webview isolée avec un design remplaçable par machine, manipulable (drag, resize, rotation libre, click-through sur forme arbitraire) et persisté par widget."
status: blocked
---

# Plan: Moteur Desktop Hub

## Overview

| Field      | Value |
| ---------- | ----- |
| **Goal**   | Construire le moteur de widgets Tauri de bout en bout (WorkerW → persistance → skins → manipulation → click-through → bridge/lifecycle), prêt à accueillir le drawer. |
| **Source** | `aidd_docs/tasks/2026_08/2026_08_26_desktop-hub-widget-engine/brainstorm.md` + `PROJECT_BRIEF.md` + `AGENTS.md` |

## Phases

| # | Phase | File |
| - | ----- | ---- |
| 0 | Spike : WorkerW + transparence + click-through | [`phase-0.md`](./phase-0.md) |
| 1 | Scaffold Tauri + hack WorkerW | [`phase-1.md`](./phase-1.md) |
| 2 | Skeleton moteur + persistance par widget | [`phase-2.md`](./phase-2.md) |
| 3 | Contrat widget + skins par machine | [`phase-3.md`](./phase-3.md) |
| 4 | Manipulation : drag, resize, rotation libre | [`phase-4.md`](./phase-4.md) |
| 5 | Click-through sur forme arbitraire | [`phase-5.md`](./phase-5.md) |
| 6 | Bridge + cycle de vie | [`phase-6.md`](./phase-6.md) |

## Resources

| Source | Verified |
| ------ | -------- |
| https://v2.tauri.app/learn/window-customization/ | Tauri v2 : fenêtres transparentes, sans décorations, startDragging, HWND natif accessible → reparenting et transparence faisables |
| https://github.com/tauri-apps/tauri/issues/12450 | Transparence sur fenêtre enfant/reparentée : bug connu → le spike (phase 0) doit le prouver sur la vraie machine |
| https://github.com/tauri-apps/tauri/issues/13270 | `transparent: true` élimine les barres noires au resize → à activer par défaut |
| https://learn.microsoft.com/en-us/microsoft-edge/webview2/samples/webview2samplewincomp | WebView2 en composition mode possible (rendu hôte + input) → option B si le mode fenêtré bloque le hit-test alpha |
| https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/browser-features | WebView2 = Chromium ; rotation CSS faisable ; les limites sont au niveau fenêtre |

## Decisions

| Decision | Why |
| -------- | --- |
| Un seul binaire Windows ; dossier de données par machine (`%APPDATA%/desktop-hub`) pour skins + layouts | Deux machines, mêmes widgets, designs différents — rien de spécifique dans le binaire |
| Skins servis via un schéma d'URI custom Rust (`skin://`) | JS sans accès FS ; compatible CSP ; prépare l'ouverture à des plugins |
| Rotation = `transform` CSS dans la webview + fenêtre boîte englobante ; click-through au niveau fenêtre | Windows ne tourne pas un HWND ; modèle le plus simple donnant la rotation libre |
| Click-through = hit-map d'alpha par pixel (`WM_NCHITTEST`, repère local + inverse-rotation) | Formes arbitraires (cercle dans un carré, glassmorphism) ; indépendant de la forme |
| Source de la hit-map (capture moteur vs masque déclaré) et mode WebView2 (fenêtré vs composition) tranchés par le spike (phase 0) | Inconnues techniques résiduelles ; à prouver sur la vraie machine avant le build |
| Config par widget = fichier JSON individuel | Jamais de structure globale unique (AGENTS.md) |
| Crate `windows` pour Win32 | Accès `FindWindowEx`/`SendMessage`/`SetParent`/`SetWindowRgn`/`WM_NCHITTEST` |
| Build : `cargo.exe` depuis WSL pour le dev ; machines cibles en Windows natif (ex. PC sans WSL) | Tauri v2 Windows ne se cross-compile pas depuis WSL ; `/mnt/c` est visible par Windows |
| Rotation libre pivotée autour du **centre du widget** (repère local) | Bbox déterministe, resize simple, pas de géométrie dépendante du curseur |
| Permissions = `permissions[]` du manifest (octroi, éditable dans le data_dir) ∧ allow-list compilée dans le moteur | Modèle perso/single-binary ; pas d'UI d'installation ; filet de sécurité moteur |
| Persistance ancrée `{ monitor_index, anchor, offset, size, rotation }`, fallback moniteur primaire | Changement de layout d'écrans → plus rien ne part hors-écran ; multi-écran complet reste différé |
| Hit-map d'alpha en **pixels physiques** (résolution du PNG de CapturePreview) | Alignement correct du click-through sous scaling DPI ≠ 100 % |
