# Décisions et questions ouvertes

## Décidé

### Structure
- **Hybride persistant/roguelite.** Personnage max niveau 50 en ~5 h, 1 point
  d'arbre par niveau. Le compte garde la progression long terme.
- **Un arbre généré à partir d'une seed pour chaque personnage.** Environ 250
  sorts, tous présents sur chaque arbre, mais disposés autrement. L'arbre se
  révèle en montant de niveau (reco).
- **Les sorts viennent de l'arbre.** Respec gratuit, dans l'arbre du
  personnage.
- **Portail au niveau 30**, un seul à la fois. Le changer coûte une ressource
  (coût fixe) : un nouveau lien entre deux nouveaux nœuds est tiré, et tu
  choisis de garder l'ancien ou de prendre le nouveau.
- **Univers de God Tower**, sans la tour ni les dieux.

### Loot
- **Artefacts** : drops des monstres, au niveau du compte. Fusion de 2
  identiques du même tier → tier suivant. Capacité en taille, de 2 à 7 selon
  le niveau.
- **Supports** : drops des monstres. Ce sont des déblocages au niveau du
  compte, sans tiers. Chaque support ne peut servir qu'une fois par build. Un
  support débloqué sort de la table de drop.
- **Artefacts légendaires au tier 5** : +1 socket de support. Un support ne
  va que sur un artefact qui partage un de ses tags.
- **Sorts légendaires** : un seul drop possible par sort, appliqué
  automatiquement, au niveau du compte. +1 socket de support (tous les sorts,
  y compris le mouvement) et des valeurs un peu plus fortes. Taux fixe par
  kill, tiré parmi les sorts pas encore légendaires. Cible : 30 min de jeu
  par légendaire (~125 h pour 250 sorts).

### Build
- **La barre garde les tailles 1–3 et l'adjacence gauche/droite** de God
  Tower.
- **Momentum** (bonus quand tu tues) limité par un cap et une décroissance
  rapide : il retombe vite sans kills, donc face au boss.

### Contrôles
- **Mouvement seulement**, et la barre de sorts cast toute seule.
- **Ciblage automatique** : un défaut selon la forme du sort (le plus
  proche, pack dense, autour de toi), modifiable par des supports de
  ciblage, avec anti-overkill. La direction du mouvement sert seulement aux
  traînées et aux charges.
- **Un slot de mouvement dédié** (dash, blink, buff de vitesse), déclenché à
  la main. C'est le seul bouton. Il a un socket de support.

### Maps
- **Zone + barre de progression** (Greater Rift) : les kills remplissent la
  barre, et à 100 % le boss apparaît. 3 à 5 minutes par map.
- **Pas de campagne.** Les maps commencent dès le niveau 1.
- **Choix parmi 3 maps** après chaque map, avec modificateurs et boss visibles.
- **Tiers infinis après le niveau 50**, avec un classement par seed.
- **Boss avec des mécaniques** : au moins une de positionnement et une qui
  teste le build. Combats de 30 à 60 s, 3 boss au départ, pas de boss
  d'endgame spéciaux pour l'instant.
- **Style procédural / géométrique** : silhouettes, animations en code, tout
  le spectacle dans les effets. La logique ne connaît que des identifiants
  visuels, pour pouvoir remplacer le rendu plus tard.
- **Mort légère** : map ratée, XP et loot gardés. **Mode hardcore** en option.

## Ouvert

### Build
1. Les paliers exacts de capacité d'artefacts et de slots de sort.
2. Les poids du drop de légendaires par type d'ennemi.

### Maps
3. **Modificateurs de map** : plus tard. Point de départ : les 20 Boss
   Contracts de God Tower.
4. La courbe des tiers de map par rapport au niveau du personnage.

### Identité
5. **Qu'est-ce qui donne une raison d'avancer?** (pas encore d'idée) Un thème, un antagoniste?
    Et est-ce qu'on garde Demon's Bargain et Cursed Mirror comme événements
    en map?
6. **Nom.**

### Plus tard
7. **Solo seulement, ou co-op plus tard?** Le raid de God Tower pourrait
    servir de base technique.
