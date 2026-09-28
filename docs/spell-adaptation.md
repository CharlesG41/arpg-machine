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

## Étape 2 : monter à 150

- Ajouter environ 90 sorts du catalogue, en priorité ceux qui créent des
  interactions (adjacence, consommation de statuts, scaling).
- Quelques briques manquantes pour certains sorts : marques (bonus contre une
  cible marquée), réserves (« stocke X depuis le dernier cast »), sorts qui
  alternent entre deux effets, orbes invoquées.
- Un benchmark par sort (chaque sort seul, puis avec quelques supports) pour
  repérer ceux qui sont trop forts ou inutiles.
