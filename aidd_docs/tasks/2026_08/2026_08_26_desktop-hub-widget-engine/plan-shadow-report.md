---
source: aidd_docs/tasks/2026_08/2026_08_26_desktop-hub-widget-engine/plan.md
generated_at: 2026-08-26
---

# Shadow Areas Report

Source: `aidd_docs/tasks/2026_08/2026_08_26_desktop-hub-widget-engine/plan.md`
Generated: 2026-08-26

Total gaps: 6 | Blocker: 2 | Major: 3 | Minor: 1

---

## Warnings

- Scan portant sur `plan.md` + `phase-*.md` ; les découvertes du spike (phase 0, vérifiées sur la machine réelle) sont intégrées au jugement.

---

## Gaps by Category

### missing failure mode

#### Newly Introduced

**[blocker]** Which mechanism replaces `WM_NCHITTEST`/`HTTRANSPARENT` to make clicks on transparent pixels reach the desktop, given that `HTTRANSPARENT` only forwards hit-tests within the same thread?
> Click-through = hit-map d'alpha par pixel (`WM_NCHITTEST`, repère local + inverse-rotation) (plan.md, Decision) / > Sur `WM_NCHITTEST` : point écran → … → `HTTRANSPARENT` ou `HTCLIENT` (phase-5, task 2)

**[blocker]** Which fallback parent puts the widget behind the desktop icons when no usable WorkerW exists, since `always-on-bottom` keeps it in front of them?
> Repli si aucune WorkerW trouvée/créée (explorer arrêté, desktop slideshow/Spotlight) : fenêtre top-level `always-on-bottom` + log (phase-1, task 2)

**[major]** How does the engine validate the chosen WorkerW before `SetParent`, to avoid clipping the widget invisible under a phantom WorkerW?
> Récupérer le HWND Tauri (`Window::hwnd`) et `SetParent` sous le WorkerW (phase-1, task 2)

**[minor]** How does the engine avoid spawning phantom WorkerW windows when it keeps sending `0x052C`?
> Sinon créer le WorkerW via `SendMessage 0x052C` (WM_SPAWN_WORKERW) (phase-1, task 2)

### unstated assumption

#### Newly Introduced

**[major]** Which build host compiles the Windows binary, given that Smart App Control blocks local cargo build scripts on this machine?
> Vérifier le build via `cargo.exe build` invoqué depuis WSL (le dossier `/mnt/c` est visible par Windows) — première commande réelle du repo (phase-1, task 1)

**[major]** What is the Smart App Control posture of the target machine (girlfriend's PC) for a distributed unsigned executable?
> Un seul binaire Windows ; dossier de données par machine (`%APPDATA%/desktop-hub`) pour skins + layouts (plan.md, Decision)
