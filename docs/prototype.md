# Prototype

Le jeu est entièrement en anglais ; les docs restent en français.

**Question à laquelle il répond :** est-ce qu'un build peut « click »? Et est-ce
qu'un style procédural (silhouettes + effets en code) suffit à rendre le clear
satisfaisant?

## Lancer

```
cd ~/Documents/ChatGPT/arpg-machine && cargo run --release
```

Planification :
- **Skill Tree** : clic gauche pour acheter la route vers un nœud, clic droit
  pour rembourser (gratuit), glisser pour déplacer, molette pour zoomer.
  « New character » génère un nouvel arbre.
- **Loadout** : glisser-déposer les sorts sur la barre de sorts, les supports
  dans le socket d'un sort, et les artefacts sur la barre d'artefacts. Glisser
  une carte hors de sa barre la retire. Un clic sur un sort ou un artefact
  l'ajoute au bout.
- Tab : changer d'onglet. Entrée : jouer.

En map :
- WASD / ZQSD / flèches : bouger
- Espace (ou Maj) : dash
- Tab : tableau des dégâts par sort
- F2 : autopilote (le jeu joue tout seul)
- Échap : retour à la planification

Outils sans fenêtre :

```
cargo run --release --example bench [tier]
cargo run --release --example spell_bench
cargo run --release --example catalogue
```

- `bench` : les builds prêts, 5 seeds, autopilote.
- `spell_bench` : chaque sort seul, comparé à sa catégorie.
- `catalogue` : tous les sorts avec leur description générée.

## Contenu

- **Skill tree** généré à partir d'une seed, sur le modèle de God Tower : une
  Origin, 6 sorties, une grappe de 5 sorts derrière chaque sortie, et des ponts
  entre grappes voisines. Chaque sort apparaît une seule fois ; seule la
  disposition change. Acheter un nœud achète le chemin le plus court depuis ce
  qu'on possède déjà, 1 point par nœud. Rembourser un nœud rembourse aussi ce
  qui ne tient plus à l'Origin. 25 points par défaut (réglable jusqu'à 60 :
  ça remplace les niveaux).
- **231 sorts adaptés du catalogue de God Tower**, dans les 15 familles (voir
  [spell-adaptation.md](spell-adaptation.md)). Chaque sort = une livraison
  (projectile, chaîne, nova, cône, blast, zone, orbite, sur soi) + ses effets
  de God Tower. Les descriptions sont générées à partir des chiffres.
- **Stats** : AD, AP, PR (résistance physique), MR (résistance magique), PV.
  Base : 30 AD, 30 AP, 20 PR, 20 MR, 300 PV. Dégâts réduits par
  100 / (100 + résistance).
- **Barre de 8 slots** (tailles 1–3, adjacence gauche/droite) et **barre
  d'artefacts de 5** de capacité (tailles 1–2).
- **8 supports**, une fois chacun par build : Multishot, Pierce, Amplify,
  Contagion, Combustion, Frenzy, Hunter, Echo.
- **16 artefacts**, dont 4 de stats (Sage's Ring +15 AP, Warband +15 AD, Iron
  Plate +25 PR, Spirit Veil +25 MR).
- **Mode sandbox** : tous les sorts, sans l'arbre. Les 4 builds prêts (Starter,
  Plague, Inferno, Steel) l'activent.
- **Une map infinie**, générée par chunks de 1200 × 1200 autour de toi (7 à 11
  packs par chunk), dans n'importe quelle direction. Les chunks loin derrière
  sont oubliés. Les kills remplissent une barre de 500 points ; à 100 %, le
  boss apparaît (et plus aucun nouveau pack pendant le combat).
- **Sanctuaires** (environ 1 par chunk sur 2) : marcher dessus donne un
  modificateur. Fury +20 % de dégâts, Swiftness +15 % de vitesse, Haste +20 %
  de vitesse de cooldown, Warding 60 Barrier et +15 PR/MR (30 s chacun),
  Vitality soigne 30 %. **Ancient** (rare, doré) : +10 % de dégâts jusqu'à
  la fin de la map, boss compris, et ça s'additionne. Des flèches sur le bord
  de l'écran pointent vers les sanctuaires hors de vue.
- **Ennemis** (des formes géométriques), avec leurs résistances (PR / MR) :
  - carré (Grunt), 0 / 0 : fonce sur toi ;
  - triangle (Runner), 0 / 0 : rapide et fragile ;
  - pentagone (Caster), 0 / 30 : garde ses distances et tire des orbes lents
    (magiques) qu'on peut esquiver ;
  - hexagone (Brute, rare), 30 / 0 : quand tu es proche, il prépare un slam au
    sol télégraphié (0,9 s) ;
  - octogone (boss). Chaque boss a ses propres stats : l'Ember Warden a
    4000 PV, 25 PR, 40 MR. Cercles de feu télégraphiés ; à 50 % de PV,
    immunisé au Burn 15 s et 3 vagues d'adds.
- **Anti-overkill**, **momentum** (cap 20, décroissance 2 s), **Barrier**.
- **Tier** : +30 % PV et +20 % dégâts des ennemis par tier.

## Architecture

- `src/lib.rs` et les modules `world`, `content`, `grid`, `math`, `rng` :
  la simulation. Pas de dépendance graphique, pas à pas fixe de 60 Hz,
  déterministe pour une seed. Elle ne connaît que des identifiants et émet des
  `Event`s.
- `src/render.rs` : tout le visuel (silhouettes, particules, flashs,
  secousses), construit à partir de l'état et des événements. On peut le
  remplacer sans toucher aux règles (décision de direction artistique).
- `src/tree.rs` (lib) : génération et règles du skill tree.
- `src/planner.rs` : écrans Skill Tree et Loadout (glisser-déposer).
- `src/ui.rs` : HUD et écran de résultat.
- `examples/bench.rs` : benchmark sans fenêtre.

## Chiffres (autopilote, tier 0, 5 seeds)

| Build | Clear | Boss | Victoires | Pic kills/min |
|---|---|---|---|---|
| Starter (4 sorts, aucun support ni artefact) | ~1 min 45 | ~1 min 50 | 3/5 | ~610 |
| Plague | ~19 s | ~14 s | 5/5 | ~2200 |
| Inferno | ~22 s | ~31 s | 5/5 | ~1600 |
| Steel | ~33 s | ~23 s | 5/5 | ~1200 |

- L'écart entre un build qui click et un build de départ est énorme :
  environ 4 fois plus de kills par minute et un clear 5 fois plus rapide.
  C'est exactement le feeling visé.
- L'autopilote est simple (il marche vers l'ennemi le plus proche et s'écarte
  quand ça devient serré). Un joueur fait mieux, surtout avec le Starter.
- À ~1500 kills/min, un taux de légendaire de 1 % par kill donnerait un
  légendaire toutes les 4 secondes : ça confirme qu'il faut fixer la cible en
  temps de jeu (design §8).
- Les maps sont plus courtes que la cible de 3 à 5 minutes : c'est voulu pour
  tester vite. On allongera avec les tiers et la progression.

## Pas dans le prototype

Niveaux (remplacés par le réglage de points), portail de l'arbre, loot et
drops, fusion, légendaires, supports sur le dash, modificateurs de map,
plusieurs boss.
