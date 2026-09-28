# ARPG Machine (nom de travail)

> Tu construis la machine. Elle vide l'écran.

Un ARPG top-down où le joueur contrôle **seulement le déplacement**, plus un
sort de mouvement (dash, blink, vitesse). Une barre de sorts, des supports et
des artefacts s'occupent de tout le reste automatiquement. Le système de build vient de [God Tower](../auto-battler) :
barre spatiale, adjacence, supports, artefacts, statuts.

Le « click » d'un build, c'est quand la machine tue plus vite que les ennemis
arrivent, et que l'écran se vide au fur et à mesure que tu avances.

## Statut

Design en cours, et un premier prototype jouable (Rust + macroquad) :

```
cargo run --release
```

## Docs

- [Design](docs/design.md) : vision, piliers, boucle, systèmes
- [Décisions et questions ouvertes](docs/open-questions.md)
- [Prototype](docs/prototype.md) : contrôles, contenu, premiers chiffres
