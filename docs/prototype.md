# Prototype

Le jeu est entièrement en anglais ; les docs restent en français.

**Question à laquelle il répond :** est-ce qu'un build peut « click »? Et est-ce
qu'un style procédural (silhouettes + effets en code) suffit à rendre le clear
satisfaisant?

## Lancer

```
cargo run --release
```

- WASD / ZQSD / flèches : bouger
- Espace (ou Maj) : dash
- Tab : tableau des dégâts par sort
- F2 : autopilote (le jeu joue tout seul)
- Échap : retour au build

Benchmark sans fenêtre (tous les builds prêts, 5 seeds, autopilote) :

```
cargo run --release --example bench [tier]
```

## Contenu

- **Une map** de 4200 × 4200 générée à partir d'une seed, avec environ 110 packs.
  Les kills remplissent une barre (500 points, rares = 6). À 100 %, le boss
  apparaît.
- **Barre de 6 slots** avec les tailles 1–2 et l'adjacence gauche/droite.
- **6 sorts**, un par forme de ciblage :
  - Arcane Bolt : projectile, l'ennemi le plus proche ;
  - Chain Lightning : chaîne, le plus proche puis 3 sauts ;
  - Frost Nova : nova autour de toi, Chill ;
  - Flame Breath (taille 2) : cône vers le plus proche, Burn ;
  - Venom Pool (taille 2) : zone au sol sur le pack le plus dense,
    Poison ;
  - Orbiting Blades : orbite permanente.
- **8 supports**, une fois chacun par build : Multishot, Pierce, Amplify,
  Contagion (le Poison se propage à la mort), Combustion (les ennemis qui
  brûlent explosent), Frenzy (momentum : cap de 20 stacks et décroissance en
  2 s), Hunter (ciblage boss → rares → le plus proche), Echo (se lance avec le
  sort de gauche).
- **Anti-overkill** : les projectiles ne visent pas un ennemi que les dégâts
  en route vont déjà tuer.
- **Dash** : le slot de mouvement manuel (sans support dans le prototype).
- **Boss : Ember Warden.** Cercles de feu télégraphiés (positionnement).
  À 50 % de PV : immunisé au Burn pendant 15 s, et 3 vagues d'adds (test du
  build).
- **4 builds prêts** pour comparer : Starter, Contagion, Combustion, Frenzy.
- **Tier** : +30 % PV et +20 % dégâts des ennemis par tier.

## Architecture

- `src/lib.rs` et les modules `world`, `content`, `grid`, `math`, `rng` :
  la simulation. Pas de dépendance graphique, pas à pas fixe de 60 Hz,
  déterministe pour une seed. Elle ne connaît que des identifiants et émet des
  `Event`s.
- `src/render.rs` : tout le visuel (silhouettes, particules, flashs,
  secousses), construit à partir de l'état et des événements. On peut le
  remplacer sans toucher aux règles (décision de direction artistique).
- `src/ui.rs` : écran de build, HUD, écran de résultat.
- `examples/bench.rs` : benchmark sans fenêtre.

## Premiers chiffres (autopilote, tier 0, 5 seeds)

| Build | Clear | Boss | Victoires | Pic kills/min |
|---|---|---|---|---|
| Starter | ~2 min 35 | — | 0/5 | ~450 |
| Contagion | ~28 s | ~35 s | 5/5 | ~1700 |
| Combustion | ~33 s | ~32 s | 5/5 | ~1400 |
| Frenzy | ~32 s | ~36 s | 5/5 | ~1600 |

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

Niveaux, arbre, artefacts, loot, légendaires, supports sur le dash,
modificateurs de map, plusieurs boss.
