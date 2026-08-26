# Brainstorm — Desktop Hub : structure du moteur & modularité

Date : 2026-08-26
Source : session aidd-refine-01-brainstorm (2 machines, widgets partagés)

## Idée consolidée

- **Desktop Hub**, moteur de widgets Tauri (Windows), widgets flottant derrière
  les fenêtres normales, au-dessus du fond d'écran — utilisé sur **deux machines**
  (utilisateur + copine).
- **Un seul binaire identique sur les deux PC.** Tout ce qui diffère — designs,
  positions, tailles, angles — vit dans un **dossier de données par machine**.
- **Mêmes widgets, designs différents** : chaque widget = **logique + design
  séparables**. Le design est une **peau complète** (HTML/CSS/ressources) chargée
  depuis le dossier de la machine, qui remplace le rendu par défaut du widget sans
  toucher à sa logique. Pas de thème global.
- **Le moteur gère le placement** : chaque widget est **resizable** et **rotatable
  librement** (angle quelconque), comme une photo posée en biais sur le bureau.
- **Interaction sur la forme tournée seule** : les pixels du widget répondent au
  clic, les zones transparentes du carré englobant laissent passer la souris
  (hit-testing porté par le moteur).
- **RAM** : la source du design (disque vs binaire) ne change rien ; le levier
  reste le nombre de webviews actives (décharger les widgets masqués).

## Assumptions et risques ouverts (à confirmer au design)

1. **Rotation libre en WebView2** : Windows ne sait pas tourner une fenêtre
   nativement → fenêtre transparente englobante + contenu tourné, hit-testing sur
   l'empreinte, redimensionnement dans le repère tourné. Risque technique
   principal ; le jalon WorkerW (brief) reste le prérequis commun.
2. **Service des skins** : le moteur doit charger du HTML/CSS depuis le disque
   dans la webview **sans** donner d'accès FS au JS du widget — à définir (bridge
   de lecture contrôlée).
3. **Format de peau** : non verrouillé (dossier `index.html` + assets ? fallback
   vers le design packagé quand aucun skin présent ?).
4. **Persistance par widget** : position, taille, angle stockés par id de widget,
   par machine.
5. **Multi-écran** : toujours ouvert (hérité du brief).
