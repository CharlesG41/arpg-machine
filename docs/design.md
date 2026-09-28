# Design — ARPG Machine

*Brouillon de brainstorm, septembre 2026.*

## 1. Pitch

Un ARPG top-down à personnage persistant. Le joueur **se déplace seulement** :
sa barre de sorts cast toute seule selon ses cooldowns, ses triggers et ses
règles de ciblage. Toute la profondeur est dans le build, comme dans God Tower.
Mais au lieu de regarder un combat figé contre deux boss, tu traverses des maps
pleines de monstres. Le résultat du build se voit directement : quand il est
bon, l'écran se vide.

**Univers** : celui de God Tower (familles de sorts, ennemis, style visuel),
vu en ARPG. Ni la tour ni les dieux n'en font partie : la tour était un nom de
code, et les dieux étaient propres à la structure de run de God Tower.

**Références** : Path of Exile 1 (profondeur de build, maps, feeling de clear),
Vampire Survivors / Halls of Torment (auto-cast, mouvement seulement),
God Tower (barre spatiale, supports, artefacts, statuts).

**Créneau** : les « bullet heaven » ont l'auto-cast, mais des builds peu
profonds. Les ARPG ont la profondeur, mais demandent de l'exécution manuelle.
Personne ne fait vraiment les deux.

## 2. Piliers

1. **Le build est le gameplay.** La compétence principale, c'est de construire
   la machine. Le déplacement est une compétence secondaire : placement, kite,
   regrouper les packs.
2. **Le clear est la récompense visible.** Un bon build se *voit* : les packs
   fondent, les statuts se propagent, les morts déclenchent d'autres morts.
3. **La machine s'explique.** Il faut toujours pouvoir comprendre pourquoi le
   build marche ou échoue : feedback des triggers, dégâts par sort, récap de
   fin de map. (C'est la règle de lisibilité de God Tower.)
4. **Chaque build a une faiblesse.** Clear vs single target vs défense. Aucun
   build ne doit tout faire sans compromis.
5. **Expérimenter coûte peu.** Réorganiser la barre et les supports doit être
   gratuit ou presque. Le plaisir de God Tower, c'est de tester des combinaisons.

## 3. Contrôles : mouvement + un slot de mouvement

- WASD / clic pour bouger. Aucun bouton pour les sorts de la barre.
- **Les sorts visent tout seuls** (décidé, voir « Ciblage » plus bas). La
  direction du mouvement ne sert de visée qu'à quelques sorts dont c'est le
  fantasme (traînée de feu, charge).

### Ciblage (décidé)

**Problème de la visée par direction du mouvement :** pour kiter, tu t'éloignes
des ennemis. Un cône ou un projectile tiré « devant toi » partirait donc à
l'opposé des ennemis. En début de personnage, avec 1 ou 2 sorts, ce serait
frustrant.

**Chaque sort a un ciblage par défaut qui découle de sa forme :**

| Forme | Ciblage par défaut |
|---|---|
| Projectile, chaîne | L'ennemi le plus proche |
| Cône, rayon | Vers l'ennemi le plus proche à portée |
| Zone au sol | Le pack le plus dense à portée |
| Nova, aura, orbite | Autour de toi (pas de ciblage) |
| Traînée, charge | La direction du mouvement |

- **Début de personnage** : les défauts marchent tout seuls. Le sort touche
  toujours quelque chose, sans rien configurer.
- **Plus tard** : des **supports de ciblage** changent le défaut. Par exemple :
  priorité élites/boss, pack le plus dense, PV les plus bas, direction du
  mouvement. C'est la version ARPG du Tab de God Tower : dans une même barre,
  certains sorts vident les packs et d'autres visent le boss. C'est comme ça
  que le build gère la tension clear vs single target.
- **Anti-overkill** : les sorts ne ciblent pas un ennemi qui va déjà mourir des
  dégâts en route. Sans ça, cinq projectiles partent sur le même trash à
  moitié mort. C'est ce qui rend l'auto-ciblage intelligent.
- Le ciblage ne fait pas le « click » : il rend le build fiable. Le click
  vient de la zone, de la propagation et des effets à la mort (§6).

### Le slot de mouvement

Un **slot dédié**, séparé de la barre, accueille un sort de mouvement. Il vient
de l'arbre comme les autres sorts. Exemples : dash, blink, buff de vitesse de
déplacement.

- **C'est le seul bouton du jeu** (décidé : manuel). Tu le déclenches toi-même, et il a un
  cooldown. Un dash ou un blink automatique se battrait contre ton propre
  déplacement.
- **Il remplace la question de l'esquive**, mais l'esquive fait maintenant
  partie du build : le choix du sort de mouvement est une décision de build.
- **Il a un socket de support**, comme les autres sorts. C'est là que ça
  devient intéressant : Barrier quand tu dashes, sol en feu derrière le blink,
  Charge sur la barre à chaque utilisation, Chill sur les ennemis traversés.
- **Les artefacts peuvent écouter l'événement « sort de mouvement utilisé »**,
  comme ils écoutent déjà les casts, les soins ou les coups reçus dans God
  Tower.
- Les familles de mouvement ont des profils différents :
  - dash : court, traverse les ennemis, bon pour les effets « au contact » ;
  - blink : téléportation, passe les obstacles, cooldown plus long ;
  - buff de vitesse : pas de burst, mais dure plusieurs secondes. Bon pour
    kiter et pour les effets « pendant que tu bouges ».

## 4. Boucle de jeu (hybride persistant/roguelite)

```
Hub → choisir une map (+ modificateurs) → clear → boss de map
    → loot (artefacts, supports, légendaires) → ajuster le build → maps plus dures
```

- **Niveau max 50** (décidé), **1 point d'arbre par niveau**, et environ
  **5 heures** pour monter un personnage au max.
- **Les sorts viennent de l'arbre** (décidé). Comme dans God Tower, chaque nœud
  traversé donne un sort. Avec environ 50 points, l'arbre doit avoir bien plus
  que 50 nœuds pour que chaque personnage fasse des choix.
- **Chaque personnage a son propre arbre, généré à partir d'une seed**
  (idée centrale, voir la section suivante).
- **Les artefacts et les supports droppent sur les monstres** (décidé). C'est
  la chasse au loot.
- **Les artefacts ont des niveaux** (décidé). Voir la section 7.
- **Respec gratuit** (décidé), pour l'arbre comme pour la barre et les
  supports. Le build n'est jamais un engagement : tester une idée ne coûte
  rien. Tout ce qui est permanent (tiers d'artefacts, sorts légendaires)
  appartient au compte ou à l'objet, jamais à un nœud de l'arbre.
- La puissance long terme vient des artefacts, de l'arbre et des versions
  améliorées ou légendaires des sorts.
- Endgame : un système de maps avec modificateurs, qui escalade (style atlas).

### Un arbre unique par personnage

Avec le respec gratuit, un personnage niveau 50 pourrait jouer tous les
builds, et il n'y aurait aucune raison d'en créer un autre. La réponse : **à la
création, chaque personnage reçoit un arbre généré à partir d'une seed**.

- **Le respec reste gratuit, mais seulement dans ton arbre.** Ton personnage
  peut jouer tous les builds *que son arbre permet*, pas tous les builds du
  jeu.
- **Un nouveau personnage = de nouveaux builds possibles.** C'est la raison
  de recommencer. Le personnage devient une « longue run » de ~5 h, et le
  compte sert de méta-progression. C'est l'hybride persistant/roguelite, dans
  l'esprit de God Tower.
- **God Tower le fait déjà** : arbres générés à partir d'une seed, frontière de
  découverte (des nœuds révélés au fil de la progression), portails, et
  garanties de génération (`campaign_generation.rs`, `campaign_safety.rs`).
  C'est le système de drafting de God Tower, étalé sur la vie d'un personnage.

**L'arbre se révèle en montant de niveau** (reco), comme la frontière de
découverte de God Tower. À la création, tu vois seulement le départ. Les nœuds
se dévoilent en jouant. Ça évite qu'on recrée des personnages en boucle
jusqu'à trouver l'arbre parfait (reroll), et la découverte fait partie du
plaisir : « ah, mon arbre a Ember Engine près d'un portail… ».

**Ce que la génération doit garantir** (comme `campaign_safety.rs`) : au moins
un sort de mouvement accessible tôt, assez de dégâts et de défense pour
monter au niveau 50, et pas d'arbre injouable.

**Les légendaires prennent tout leur sens.** Un nouvel arbre qui te donne
accès à un de tes sorts légendaires, c'est un moment excitant. Et vouloir
jouer un légendaire que ton arbre actuel n'a pas, c'est une raison de créer
un personnage.

**Bonus : les seeds se partagent.** « Essaie la seed 48213, elle a un combo
Poison + portail de fou. » Et des seeds du jour ou de la semaine, avec un
classement, comme le daily board de God Tower.

**Tous les sorts sont sur chaque arbre** (décidé). C'est la disposition qui
change. Avec environ **250 sorts** et **50 points**, un personnage atteint
environ 20 % de l'arbre. Deux seeds donnent donc des voisinages et des
combinaisons complètement différents, sans qu'aucun sort soit jamais
« absent » du jeu.

### Portail (niveau 30)

Au **niveau 30**, le personnage débloque **un portail** : un lien entre deux
nœuds éloignés de son arbre, comme les portails de God Tower.

- **Un seul portail à la fois** (décidé).
- **Le changer coûte une ressource, à coût fixe** (décidé). Tu paies, un
  nouveau portail entre **deux nouveaux nœuds** est tiré, puis **tu choisis :
  garder l'ancien ou prendre le nouveau**. Le pari ne peut donc pas empirer ton
  arbre, il coûte seulement la ressource.
- C'est la seule chose payante dans l'arbre (le respec des points reste
  gratuit), donc changer de portail est une vraie décision.
- Le tirage étant aléatoire, le portail ne peut pas servir à atteindre
  n'importe quel sort : l'arbre garde son identité.
- Si des points passaient par l'ancien portail, les nœuds devenus
  inaccessibles sont remboursés automatiquement (le respec est gratuit de
  toute façon).
- La ressource est un drop des monstres. Elle sert aussi de sink pour
  l'économie.

À trancher (arbre) :
- **Que devient un personnage niveau 50?** Il continue l'endgame (et farme
  légendaires, artefacts et supports pour le compte) jusqu'à ce que tu aies
  envie de builds que son arbre ne permet pas. Les deux boucles coexistent.
- Combien de personnages peut-on garder? Illimité, ou quelques slots?

### Compte vs personnage

| Compte (persistant) | Personnage |
|---|---|
| Sorts légendaires (décidé) | Seed et arbre |
| Artefacts et leurs tiers (décidé) | Niveau (1–50) et points d'arbre |
| Supports, sans tiers (décidé) | Portail (niveau 30) |
| | Capacité d'artefacts et slots de sort (liés au niveau) |

**Décidé : artefacts et supports au niveau du compte.**

**Les supports sont des déblocages** (décidé) : une fois trouvé, un support est
disponible pour tous tes personnages. Il n'a pas de tiers. **Chaque support ne
peut être utilisé qu'une fois dans le build d'un personnage.** Tu ne peux donc
pas mettre Inferno sur cinq sorts : choisir où va chaque support fait partie
du puzzle. Un support mis dans un artefact légendaire compte aussi comme son
utilisation.

Comme un support débloqué ne peut plus servir en double, il sort de la table
de drop, comme les légendaires (décidé). Les drops de support vont toujours vers
un support qui te manque. Avec un arbre différent
à chaque personnage, ton personnage ne peut pas tout jouer. Le compte peut
donc tout garder sans casser la raison de recommencer. Si les artefacts
restaient sur le personnage, chaque nouveau personnage repartirait de zéro
loot, ce qui est très punitif pour des personnages de 5 h.

La capacité liée au niveau protège un nouveau personnage : même avec tous les
artefacts du compte, il commence avec 2 de capacité. Il n'y a donc pas de
personnage niveau 1 qui écrase tout.

## 5. Ce qu'on garde de God Tower

| God Tower | ARPG Machine |
|---|---|
| Barre de sorts (8–10 slots, tailles 1–3, adjacence) | Pareil. Les slots se débloquent avec le niveau. L'adjacence reste un levier central. |
| Supports (1 socket, 2 si légendaire) | Pareil. On ajoute des supports spatiaux (pierce, chain, +projectiles, zone). |
| Artefacts (taille, stats, triggers) | Artefacts qui droppent sur les monstres, avec des tiers (fusion). **Capacité en taille, comme dans God Tower.** |
| Ability Tree + budget de Moves + portails | Arbre persistant et **source des sorts**. Points gagnés par niveau, portails = keystones qui relient des régions éloignées. |
| PP / MP, Poison, Burn, Chill, Freeze, Barrier, Regen, Haste, Charge | Tous gardés, avec en plus des effets à la mort et de la propagation. |
| Tab = cible désignée | **Règles de ciblage par sort** (le plus proche, le plus dense, élite/boss, PV le plus bas). Elles deviennent un levier de build, via un support ou un réglage. |
| Boss Contracts (risque opt-in) | Modificateurs de map : plus de risque, plus de loot. |
| Demon's Bargain / Cursed Mirror (Divine Intervention retirée avec les dieux) | Événements en map : pactes, choix risqués. À réévaluer sans les dieux. |
| Sorts améliorés « prouvés » en combat | Pareil : un sort amélioré trouvé doit tuer un boss de map pour se débloquer pour de bon. |

## 6. Ce qui est nouveau : l'espace

### Formes de sorts

Projectile, nova, cône, rayon, zone au sol, orbite, chaîne, invocation/totem, aura.

### Modificateurs spatiaux

Nombre de projectiles, pierce, chain, fork, taille de zone, durée, portée,
vitesse de projectile.

### Le « click » : propagation et snowball

Le feeling de clear vient moins des dégâts bruts que de **ce qui se passe quand
un ennemi meurt** :

- **Propagation** : le Poison ou le Burn saute aux ennemis proches à la mort.
  Un ennemi Frozen éclate et Chill les autres autour.
- **Réaction en chaîne** : explosion de corps, projectiles qui partent du
  cadavre.
- **Momentum** : Charge ou Haste quand tu tues. Tuer fait caster plus vite, ce
  qui fait tuer plus vite. C'est une boucle positive **voulue**. Quand elle
  s'enclenche, le build « click ».

**Limites du momentum (décidé) : un cap et une décroissance rapide.** Par
exemple : maximum 20 stacks, et tout est perdu après ~2 secondes sans kill.
Une petite jauge affiche le momentum. Pas de rendements décroissants : trop
durs à lire.

Le momentum n'est pas obligatoire. C'est une source de click parmi d'autres
(propagation, réactions en chaîne), et chaque build choisit les siennes.

Les boss cassent volontairement le snowball : un seul ennemi, peu de kills. Ça
crée la tension clear vs single target. Le build doit vider les packs *et* tuer
le boss.

### Densité

Comme le joueur ne fait que bouger, les maps doivent être denses. Il y a des
packs partout, et avancer est toujours récompensé.

## 7. Artefacts : drops et niveaux

Les artefacts droppent sur les monstres et montent de niveau. Deux modèles
possibles :

| | XP à l'usage | Fusion (2 identiques du même tier → tier +1) |
|---|---|---|
| Valeur d'un drop en double | Aucune (c'est du déchet) | **Chaque doublon compte** |
| Expérimentation (pilier 5) | Punie : un nouvel artefact repart niveau 1, donc tu gardes l'ancien | Libre : le tier suit l'objet, pas le temps passé avec |
| Chasse au loot | Faible, tu grind avec ce que tu as | Forte, tu cherches *le* doublon |
| Décisions | Aucune, c'est automatique | Fusionner maintenant, ou garder deux copies pour deux builds? |
| Familiarité | ARPG classique | Auto-battlers (étoiles dans TFT), cohérent avec God Tower |

**Décidé : la fusion.** Elle donne une valeur à chaque drop et protège
l'expérimentation.

- Fusion : 2 copies du même artefact au même tier → 1 copie au tier suivant.
  La progression de copies est donc exponentielle (tier 4 = 8 copies de tier 1).
- Garder l'identité fixe des artefacts de God Tower (un effet précis), sans
  affixes aléatoires au début. Le tier fait monter les chiffres, et à certains
  tiers il débloque un bonus d'effet. Avec des affixes aléatoires, on devrait
  décider quels affixes survivent à la fusion, ce qui complique tout.
- Les tiers de map limitent le tier maximum qui peut dropper. Les maps plus
  dures droppent directement des tiers plus hauts, pour que le grind ne soit
  pas juste de la fusion.
- L'XP à l'usage peut rester comme **maîtrise cosmétique** (comme la Spell
  Mastery de God Tower) : records, titres, pas de puissance.

### Artefacts légendaires (idée)

Le tier 5 est le tier max. Un artefact qui l'atteint devient **légendaire** et
gagne **1 socket de support**. Ça donne une ligne d'arrivée à la fusion, et une
règle unique pour tout le jeu : **légendaire = un socket de plus**, sur un sort
comme sur un artefact.

- Coût : 2⁴ = **16 copies de tier 1** pour un tier 5. Les maps plus dures, qui
  droppent des tiers plus hauts, raccourcissent le chemin.
- **Qu'est-ce qu'un support fait sur un artefact?** Dans God Tower, les
  supports modifient un sort (cooldown, dégâts, statut ajouté au cast). Pour un
  artefact, le support modifierait son **déclenchement**. Par exemple, un
  artefact qui applique du Burn quand tu es touché, avec Inferno, applique du
  Burn doublé.
- Problème : un artefact de stats pures (juste +PP, par exemple) n'a pas de
  déclenchement. Solutions :
  - **compatibilité par tags** (décidé) : un support ne se socket que sur un
    artefact qui partage un de ses tags (burn, poison, barrier…). Les tags
    existent déjà dans God Tower ;
  - ou un effet spécial « sur artefact » pour chaque support : plus flexible,
    mais un deuxième texte à designer pour chaque support.

### Capacité d'artefacts

**Direction : la capacité monte avec le niveau, de 2 au départ à 7 au max.**
Les artefacts font 1 ou 2 de taille (0 pour certaines variantes spéciales),
comme dans God Tower. Un total de 7 donne entre 4 et 7 artefacts équipés, selon
le mélange.

Paliers proposés, pour un niveau max de 50 (environ 5 h de jeu) :

| Niveau | 1 | 5 | 10 | 18 | 28 | 40 |
|---|---|---|---|---|---|---|
| Capacité | 2 | 3 | 4 | 5 | 6 | 7 |

- Les paliers sont **concentrés tôt** : le puzzle s'agrandit vite pendant la
  campagne, et l'endgame commence avec presque toute la capacité. Le dernier
  palier reste un objectif, sans bloquer le build pendant des dizaines
  d'heures.
- La barre de sorts monte aussi avec le niveau. Il faut aligner les deux
  courbes, pour que chaque niveau ne donne pas en même temps un slot de sort
  et de la capacité d'artefact.
- Idée : des keystones de l'arbre qui échangent de la capacité (−1 slot de
  sort, +1 capacité d'artefact, ou l'inverse). Comme le respec est gratuit, ce
  serait un vrai choix de build, pas un piège.

## 8. Sorts légendaires

**Décidé : drop direct, un seul par sort.** Chaque sort a exactement un drop
légendaire possible. Quand il droppe, il s'applique automatiquement au sort, qui
devient légendaire pour toujours sur ton compte. Pas de sceau à gérer, pas de
doublon : un sort déjà légendaire sort de la table de drop.

Ce que donne un légendaire :
- **1 socket de support de plus** (décidé). C'est la même règle pour tous les
  sorts, y compris le sort de mouvement ;
- **des valeurs un peu plus fortes** : dégâts, soins, Barrier, stacks de
  statut. Reco de départ : +20 % sur les valeurs de base, à ajuster en
  playtest.
- Pas d'effet unique : le légendaire reste lisible, et on n'a pas à designer
  un effet par sort.

Le statut appartient **au compte** (décidé) : il survit aux respecs et passe
à tous tes personnages. La collection de légendaires devient l'objectif à long
terme du compte.

### Taux de drop

**Le modèle :** chaque ennemi tué a une chance fixe de dropper un légendaire.
Le sort est tiré **parmi ceux qui ne sont pas encore légendaires**. Avec un
taux constant et un pool qui rétrécit :

- le temps moyen entre deux légendaires reste le même du premier au dernier ;
- le temps pour tout collectionner est linéaire. À 1 % avec 100 sorts :
  100 kills en moyenne par légendaire × 100 sorts = **10 000 kills** ;
- quand il ne reste qu'un sort, le prochain drop est forcément celui-là. Il
  n'y a pas de « dernier sort introuvable ».

**Le vrai paramètre, c'est le temps de jeu, pas le nombre de kills.** Dans un
ARPG où l'écran se vide, on tue énormément :

| Kills par minute (build qui click) | 10 000 kills (100 % à un taux de 1 %) |
|---|---|
| 100 | ~1 h 40 |
| 300 | ~33 min |

À 1 % par ennemi, un bon build aurait un légendaire toutes les quelques
secondes, et toute la collection en une soirée. Proposition :

- **Cible : 1 légendaire par 30 minutes de jeu** (décidé). Avec **250
  sorts**, ça fait ~125 heures pour tout collectionner, ou l'équivalent
  d'environ 25 personnages montés au niveau 50 (endgame compris). Le temps d'endgame compte aussi : ce sont 100 heures de
  jeu en tout, peu importe la répartition. Le taux se calcule ensuite à partir
  des kills par minute mesurés dans le prototype.
- **Pondérer par type d'ennemi** : trash presque 0, rares et élites plus haut,
  boss de map avec une vraie chance. Sinon, la chasse aux légendaires ne
  récompense que le clear, et les boss ne donnent rien de spécial.
- Les légendaires droppent pour n'importe quel sort, même ceux que tu
  n'utilises pas. Avec le respec gratuit, un légendaire inattendu devient une
  **raison d'essayer un nouveau build** : ça rejoint le pilier 5.

À trancher :
- Les poids par type d'ennemi (trash, rares, élites, boss).

## 9. Maps

**Décidé : zone + barre de progression** (modèle Greater Rift de Diablo 3),
**3 à 5 minutes par map, pas de campagne.**

- Une zone générée de taille moyenne, dense.
- Chaque kill remplit une barre de progression. Les rares et les élites
  valent plus que le trash.
- À 100 %, le boss de map apparaît. Le tuer termine la map.
- La vitesse de clear **est** la progression : un build qui click remplit la
  barre deux fois plus vite, et ça se voit.
- Pas besoin de fouiller chaque recoin : on avance, on tue, on remplit.
- Pas de campagne linéaire : les maps commencent dès le niveau 1, avec des
  tiers qui montent. On joue environ 25 personnages, et une campagne rejouée
  25 fois deviendrait une corvée.

Ordres de grandeur :
- **Niveau 1 → 50 en ~5 h** : environ **60 à 100 maps**.
- **Un légendaire par 30 min** : environ **1 légendaire toutes les 6 à 10
  maps**.

Les modèles écartés, pour mémoire : zone à explorer à la PoE (trop de contenu
et de backtracking), arène à vagues (pas d'avancée), couloir (répétitif). Des
éléments d'arène pourraient revenir comme événements dans une map.

### Choisir la prochaine map (décidé)

Après chaque map, tu choisis **parmi 3 maps**. Pour chacune, tu vois ses
**modificateurs et son boss** avant d'entrer. C'est le pilier de God Tower :
lire ce qui s'en vient et préparer son build. Avec le respec gratuit, tu peux
ajuster ton arbre avant d'entrer. Pas de maps en objets (PoE), et donc pas de
problème de réserve de maps.

### Après le niveau 50 (décidé)

Les **tiers de map montent à l'infini**, comme les Greater Rifts. La
difficulté monte, et les récompenses aussi : artefacts de tier plus élevé,
plus de ressource de portail. Un **classement du tier le plus haut atteint par
seed** donne un objectif au personnage niveau 50. Il continue à pousser
jusqu'à ce que tu aies envie de builds que son arbre ne permet pas.

### La mort (décidé)

**Pénalité légère** : la map est ratée, mais tu gardes l'XP et le loot déjà
ramassé. Tout le jeu encourage à essayer des builds, donc mourir ne doit pas
coûter cher.

**Mode hardcore** en option : le personnage meurt pour de bon. Comme un
personnage est déjà une longue run liée à sa seed, le hardcore colle
naturellement. Classements séparés.

### Boss de map

**Direction (décidé) : les boss ont des mécaniques.** Un boss n'est pas juste
« tu attends 30 secondes et il meurt ». Tes sorts sont automatiques, mais le
combat te demande quand même quelque chose.

Deux familles de mécaniques, qui répondent aux deux choses que le joueur
contrôle :

**1. Mécaniques de positionnement** (ce que tu fais *pendant* le combat, avec
ton mouvement et ton sort de mouvement) :
- zones télégraphiées à esquiver (cercles au sol, rayons qui tournent, charges) ;
- zones sûres qui bougent, ou une arène qui rétrécit ;
- le boss est invulnérable sauf quand tu te tiens dans une zone précise
  (tes sorts le frappent seulement de là) ;
- des points faibles qui apparaissent : les supports de ciblage décident si
  ta barre les vise.

**2. Mécaniques qui testent le build** (ce que tu as préparé *avant*,
l'esprit des Boss Contracts de God Tower) :
- **phase d'adds** : le boss invoque des vagues. Le clear compte aussi dans le
  combat de boss, et les builds momentum peuvent se recharger ;
- **bouclier** à casser avec un burst avant qu'il se régénère ;
- **immunité temporaire à un statut** (immunisé au Burn pendant une phase) :
  un build qui mise tout sur un seul statut doit avoir un plan B ;
- **soins du boss** : le Grievous Wounds (appliqué par le Burn) devient utile ;
- **perturbation de la barre** : désactive un slot, gèle les sorts de gauche,
  inverse l'adjacence pendant quelques secondes.

**Reco : chaque boss combine au moins une mécanique de positionnement et une
mécanique de build.** Comme le boss est visible dans le choix parmi 3 maps, et
que le respec est gratuit, tu peux ajuster ton build pour le contrer. C'est le
pilier de God Tower : lire ce qui s'en vient et préparer sa machine.

Décidé :
- **Durée d'un combat de boss : 30 à 60 secondes**, dans une map de 3 à 5
  minutes.
- **3 boss au départ**, pour bien peaufiner les mécaniques avant d'en ajouter.
- **Pas de boss d'endgame spéciaux** pour l'instant.

Exemple de boss qui combine les deux familles :

> **Ember Warden** (Gardien de braise) : lance des cercles de feu au sol (à esquiver). À 50 %
> de PV, il devient immunisé au Burn pendant 15 secondes et invoque des adds.
> Un build tout Burn doit avoir un deuxième type de dégâts, ou profiter des
> adds pour recharger son momentum.

À explorer :
- **Map infinie** (décidé) : on peut aller dans n'importe quelle direction
  aussi longtemps qu'on veut. Le boss apparaît quand la barre est pleine.
- **Sanctuaires** (décidé) : des modificateurs à ramasser pendant la map
  (+20 % de dégâts ou +15 % de vitesse pendant 30 s, etc.). Les rares
  (Ancient) durent jusqu'à la fin de la map, boss compris. Ils donnent une
  raison de bouger et de choisir un chemin.
- **Modificateurs de map** (plus tard) dans l'esprit des Boss Contracts :
  risque visible et opt-in, contre plus de loot. Avec le choix parmi 3 maps,
  les modificateurs sont visibles avant d'entrer. Les 20 Boss Contracts de
  God Tower sont un bon point de départ.
- La courbe exacte des tiers par rapport au niveau du personnage.

## 10. Direction artistique

**Le problème :** avec l'IA, c'est difficile d'obtenir un style cohérent et de
belles animations de personnages.

**Constats :**
- **Dans un ARPG de clear, le spectacle vient surtout des effets**
  (particules, flashs, morts, propagation, screen shake), pas des animations
  de personnages. La caméra est loin, les ennemis sont petits et meurent en
  une seconde.
- **Des jeux à succès du genre ont des visuels très simples** : Vampire
  Survivors, Brotato, 20 Minutes Till Dawn, Nova Drift. Leur force, c'est la
  lisibilité et le « juice ».
- **Les effets en code, l'IA les fait bien et de façon cohérente.** God Tower
  le prouve déjà : ses effets de sorts (`spell_vfx`) sont procéduraux, avec 12
  styles cohérents. Ce qui est incohérent, ce sont les *images générées*, pas
  le code.
- **On ne peut pas juger le « click » avec des visuels vides.** Le feeling de
  clear dépend du feedback visuel. On ne peut donc pas tout repousser à plus
  tard.

**Options :**

| Option | Comment | + | − |
|---|---|---|---|
| A. Placeholder et on attend une meilleure IA | Des carrés maintenant, on remplace plus tard | Rien à faire maintenant | Pari sur le futur. Le feeling ne peut pas être évalué. |
| B. Style procédural / géométrique | Formes simples, silhouettes, tout le spectacle en effets de code (shaders, particules) | Cohérent par construction, l'IA le fait bien, lisible avec des centaines d'ennemis | Moins « fantasy », une identité à trouver |
| C. Pack d'assets acheté | Un pack pixel art ou 2D d'un seul artiste (itch.io, etc.) | Cohérent (un seul artiste), des animations incluses | Limité au contenu du pack, style pas unique |
| D. 3D low-poly | Modèles générés par IA (Meshy, Tripo) + bibliothèque d'animations (Mixamo) + un shader commun | Les animations viennent d'une bibliothèque, pas de l'IA | Plus complexe techniquement (moteur 3D) |

**Décidé : B, avec une architecture qui permet de remplacer les visuels plus
tard.**
- Personnages et ennemis en silhouettes lisibles, avec une animation
  procédurale simple (étirement, rebond, flash quand touché, désintégration à
  la mort).
- Tout le spectacle dans les effets en code, en reprenant l'approche de
  `spell_vfx` de God Tower.
- La logique du jeu ne connaît que des identifiants visuels (« ennemi type
  X »). Le rendu décide de quoi ça a l'air. Si une meilleure IA ou un artiste
  arrive, on remplace le rendu, pas le jeu.

## 11. Défense

On garde les couches de God Tower : Barrier, Regen, PR/MR, Heal, Leech. En
ARPG, le positionnement s'ajoute par-dessus. Un build fragile peut survivre en
kitant bien, et un build tanky peut foncer dans le tas.

## 12. Lisibilité

- Dégâts et kills par sort, en direct (petit meter) et dans le récap de map.
- Feedback visuel distinct pour chaque trigger d'adjacence ou de support.
- Récap de mort : ce qui t'a tué, et quels sorts n'ont pas fait leur job.

## 13. Scope et risques

- **Contenu** : un ARPG persistant consomme énormément de contenu (maps,
  ennemis, loot, endgame). C'est le plus gros risque.
- **Perf** : des centaines d'ennemis et de projectiles. Il faut une grille
  spatiale et un moteur de simulation pensé pour ça dès le départ.
- **Agency** : le mouvement seulement peut sembler passif. La visée par
  direction et la densité des maps doivent compenser.
- **Balance** : les boucles de snowball explosent facilement. Il faut des caps
  ou des rendements décroissants, pensés dès le départ.
- **Priorité** : le raid multijoueur de God Tower est déjà en cours. Ce projet
  reste un spin-off tant qu'on n'a pas décidé autrement.

## 14. Prototype

Voir [prototype.md](prototype.md).
