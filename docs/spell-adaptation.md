# Adapter les sorts de God Tower

Source : `design/spells-150` et `design/spells-250` dans le repo
`auto-battler` (250 sorts, 15 familles). Le catalogue de l'ARPG est dans
[`src/spells.rs`](../src/spells.rs).

## Méthode

Chaque sort de God Tower garde son **nom, sa famille, sa taille, son cooldown
et ses ratios** (AD, AP, PR, MR, % de PV). On lui ajoute une **livraison**,
c'est-à-dire comment il atteint les ennemis dans une map :

| Texte God Tower | Livraison ARPG |
|---|---|
| « frappe la cible » | Projectile vers l'ennemi le plus proche |
| « l'ennemi avec le moins de PV % / le plus de PV » | Projectile avec ce ciblage |
| « frappe la cible, puis un autre ennemi » | Chaîne |
| « tous les ennemis » | Nova (autour de toi) ou Blast (sur le pack le plus dense) |
| « frappe N fois, X s d'intervalle » / « après X s » | Zone (coups répétés sur un point) |
| Barrier, soin, buff, charge, Haste | Sur soi (se lance seulement en combat) |

Les effets sont des **briques de données** (dégâts, Burn, Poison, Chill,
Freeze, Bleed, réduction de PR/MR, consommer un statut, vol de vie, soin sur
kill, Barrier, soin, régénération, buff de stat, Haste, charge des voisins,
payer des PV, épines, conditions). Un nouveau sort = une entrée de données,
pas du code. Les descriptions sont générées à partir des mêmes données.

**Cooldowns** : ×0,4 (un ARPG nettoie des packs, pas un duel). Les charges
suivent la même échelle.

## Ce qui ne se traduit pas

Les règles propres au duel sont abandonnées ou reformulées :
- les barres de sorts ennemies (« si le sort ennemi est chargé à 75 % ») ;
- les Barriers ennemies, Grievous Wounds (les ennemis ne se soignent pas) ;
- les contracts et les marques qui lisent les sorts des ennemis.

Exemples d'adaptations : **Heartseeker** (+1 dégât par % de PV manquant)
devient ×1,4 sous 50 % de PV ; **Siegebreaker** (double contre la Barrier)
devient ×1,5 contre les rares et les boss ; **Thorncoat** et **Ice Ward**
(riposte aux coups reçus) touchent les ennemis au contact.

## Étape 1 (faite) : 63 sorts

Les 15 familles, de 3 à 5 sorts chacune, choisis parmi ceux qui se traduisent
le mieux et qui couvrent toutes les livraisons.

## Étape 2 (faite) : 231 sorts

Tous les sorts du catalogue qui se traduisent, soit 231 sur 250. Il a fallu de
nouvelles briques :

- **Marques** (Dusk, Brand, Brittle, Duel, Vow, Gravity) : bonus contre une
  cible marquée.
- **Réserves** : « stocke X depuis le dernier cast » (casts des autres sorts,
  casts des sorts de taille 1, Burn consommé, dégâts absorbés par la Barrier,
  Barrier gagnée, PV payés, soins reçus, dégâts de Poison).
- **Comptes** : artefacts équipés, familles distinctes, slots vides, marques
  sur la cible, stacks sur la cible, ennemis proches qui remplissent une
  condition, sorts encore en cooldown.
- **Conditions sur la cible** (avec « sinon ») et **conditions sur soi**
  (voisins, tailles des voisins, AD vs AP, kill récent).
- **Buffs de slot** : Empower (le voisin fait plus de dégâts) et Catalyze (le
  voisin applique plus de Poison et de Burn).
- **Alternance** entre deux effets, **réduction de dégâts**, **soins reçus
  augmentés**, **soins convertis en Barrier**, **gain permanent** pour la map,
  **soins et Barrier selon les PV perdus récemment**, **payer des PV contre de
  la Barrier**, **charger le sort le plus proche d'être prêt**.

Les 19 sorts laissés de côté dépendent trop du duel : Static Cage, Veil of
Silence, Grief Engine, Haunting Bell, Memory Keeper, Spirit Link, Time Split,
Thunderbrand, Serpent's Tithe, Venom Courier, Reactive Solvent, Basalt
Exchange, Orbit, Contagion (le nom est déjà pris par un support), Pulsekeeper,
Scarlet Rhythm, Bloodletter's Oath, Resonance Lattice, Cauterize.

## Balance

`cargo run --release --example spell_bench` teste chaque sort à côté d'un
Arcane Bolt, 60 s sur la même map, sans dégâts reçus, et le compare à la
médiane de sa livraison.

- Les sorts de zone (nova, cône, blast, zone) nettoient bien plus que les
  projectiles : c'est voulu, les projectiles visent le boss et les rares.
- Beaucoup de sorts « faibles seuls » sont des sorts de synergie : Flashover
  sans Burn, Septic Burst sans Poison, Dusk Mark, Pandemic, Capacitor sans
  Barrier. Ils se jugent dans un build, pas seuls.
- Prochaine passe : jouer, puis ajuster les chiffres avec ce benchmark.
