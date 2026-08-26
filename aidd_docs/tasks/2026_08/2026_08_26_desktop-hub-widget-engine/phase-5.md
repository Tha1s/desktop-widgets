---
status: pending
---

# Instruction: Click-through sur forme arbitraire

## Architecture projection

> Tree of the final files. ✅ create · ✏️ modify · ❌ delete

```txt
src-tauri/src/
├── core/
│   └── hit_test.rs              ✅ masque alpha → région GDI (repère local + pixels physiques)
├── lib.rs                       ✏️ SetWindowRgn (région = empreinte opaque)
src/widgets/placeholder/
└── index.html                   ✏️ cercle dans un carré (forme non-rectangulaire de test)
```

## User Journey

```mermaid
flowchart TD
  A[Widget tourné rendu] --> B[Masque alpha (repère local)]
  B --> C[Construire la région GDI depuis le masque]
  C --> D[SetWindowRgn : hors région ≠ partie de la fenêtre]
  D --> E[Clics sur le transparent passent au bureau]
  F[Rotation / resize / forme] --> G[Reconstruire la région]
```

## Tasks to do

### `1)` Masque d'alpha
> Masque opaque/transparent dans le repère local du widget, en pixels physiques.

1. `core/hit_test.rs` : source du masque (capture moteur ou masque déclaré par le widget — choix verrouillé en phase 0)
2. Stockage du masque **en pixels physiques**, dans le repère local (non tourné), indexable par pixel

### `2)` Région GDI → click-through
> La région de fenêtre EST le click-through : hors région, le widget n'existe pas pour l'input.

1. Construire une `HRGN` depuis le masque (`CreateRectRgn` par plage opaque + `CombineRgn`, ou polygone pour formes simples)
2. Appliquer via `SetWindowRgn` (pixels physiques, coordonnées fenêtre)
3. **Ne pas** s'appuyer sur `WM_NCHITTEST`/`HTTRANSPARENT` : ça ne fait pas passer les clics au bureau pour une fenêtre top-level (même thread seulement — prouvé en phase 0)
4. Le scaling DPI ≠ 100 % (125 %/150 %) est géré en construisant la région en pixels physiques — jamais de mélange CSS/logique pixels

### `3)` Mise à jour
> La région reste exacte après manipulation.

1. Reconstruire + réappliquer `SetWindowRgn` à l'ouverture, à la rotation, au resize, et sur signal du widget (bridge)
2. Vérifier la cohérence avec l'overlay de debug (phase 4)

## Test acceptance criteria

| Task | Acceptance criteria |
| ---- | ------------------- |
| 1 | Cercle tourné dans un carré : les clics sur les coins transparents atteignent le bureau/les icônes |
| 2 | Les clics sur l'empreinte opaque (cercle) fonctionnent normalement |
| 3 | La zone de click-through reste correcte après rotation/resize |
| 4 | Le hit-test reste aligné sur l'empreinte à 125 % et 150 % de scaling Windows |
