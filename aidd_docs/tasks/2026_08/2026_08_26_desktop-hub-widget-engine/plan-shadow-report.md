---
source: aidd_docs/tasks/2026_08/2026_08_26_desktop-hub-widget-engine/plan.md
generated_at: 2026-08-26
status: clean
---

# Shadow Areas Report

Source: `aidd_docs/tasks/2026_08/2026_08_26_desktop-hub-widget-engine/plan.md`
Generated: 2026-08-26

Total gaps: 0 | Blocker: 0 | Major: 0 | Minor: 0

---

## Gaps by Category

Aucun trou restant. Les 14 gaps du scan initial ont été résolus dans le plan amendé :

- **Build env** → `AGENTS.md` + phase-1 (`cargo.exe` depuis WSL, prérequis Windows, vanilla frontend).
- **Seed registry** → phase-2 (tous les `widgets/*/widget.json`, `enabled` par widget).
- **Transfert de skins** → phase-3 (copie manuelle, pas de synchro en v1).
- **Pivot de rotation** → phase-4 (centre du widget, Ctrl+drag).
- **Métrique RAM / barre de baisse** → phase-6 (count renderer + private working set, −1 au masquage).
- **Scaling DPI** → phase-0 + phase-5 (hit-map en pixels physiques, critère 125 %/150 %).
- **Ancrage multi-écran** → phase-2 + plan.md (`monitor_index`/`anchor`/`offset`, fallback primaire).
- **Octroi des permissions** → phase-3 + phase-6 + plan.md (manifest ∧ allow-list moteur).
- **Qui masque un widget** → phase-6 (API `hide/show` + geste debug double-clic).
- **Échec WorkerW** → phase-1 (chaîne de repli, top-level `always-on-bottom` + log).
- **Crash webview** → phase-6 (reload 1×, désactivation après 3 échecs).
- **Fichier skin manquant** → phase-3 (404 + log, repli sur le skin packagé).
- **Deps crates** → phase-0 (`png`, `webview2-com` si composition).
