# Backend web : un seul chemin — contrat WIT + implémentation hôte en JS

`packages/web` n'a **qu'un seul** chemin de backend web. Le « double backend » décrit par les
premiers documents — `web` (`wasm-bindgen` / `web-sys`) à côté de `wit-bindings` — n'existe plus :
le côté `web` n'avait aucun consommateur, et ses dépendances comme ses branches `cfg` sont retirées.

## Ce qu'est ce chemin

| Élément | Valeur |
| --- | --- |
| Cible de compilation | `wasm32-wasip2` (WASI 0.2 / 0.3, Component Model) |
| Contrat | Mondes WIT `tairitsu-browser:*` (`packages/browser-worlds/wit/`) |
| Bindings côté guest | `wit-bindgen` (feature `wit-bindings`) |
| Implémentation côté navigateur | Hôte JS (`packages/browser-glue/`) |
| Capacités navigateur du guest | Hôte Component Model, via les imports injectés par l'hôte |

Le point d'entrée est `WitPlatform` (`packages/web/src/wit_platform.rs`, feature `wit-bindings`).
`BrowserPlatform` (`packages/web/src/browser.rs`) est une implémentation factice — callbacks
immédiats, rectangles par défaut — et n'est pas un backend web-sys.

## Pourquoi ce seul chemin subsiste

1. **Le Component Model change la façon dont les « capacités » sont livrées.** `wasm32-unknown-unknown` +
   `web-sys` compile les capacités navigateur directement dans la section imports du guest : impossible
   de déclarer *qui* est l'hôte ni *quelles* capacités sont accordées. Un contrat WIT expose la surface
   de capacités sous forme d'interfaces imports vérifiables et versionnables, que l'hôte autorise
   instance par instance.
2. **Transpilation au build, pas auto-hébergement à l'exécution.** jco transpile un component en core
   wasm plus un runtime JS d'ABI canonique au moment du build ; les navigateurs n'ont **pas** de
   `WebAssembly.Component` natif. « Contrat en WIT, implémentation en JS » est donc la seule forme
   viable aujourd'hui — pas un compromis.
3. **Les responsabilités de l'hôte deviennent explicites.** Instanciation, autorisation des capacités,
   ordonnancement asynchrone et cycle de vie (y compris arrêt forcé) reviennent à l'hôte tairitsu ;
   le guest se contente de décrire ce dont il a besoin.

## Niveaux de mondes

Les mondes WIT sont répartis en paquets nommés `tairitsu-browser:{domain}` sous
`packages/browser-worlds/wit/` (`browser-full.wit` et les paquets par domaine générés dans
`generated/`). Ce répertoire fait foi pour la liste des paquets.

## Build et packaging

La CLI `tairitsu build` de `packages/packager` ne prend en charge **que la cible `component`**
(`wasm32-wasip2` + chemin browser-glue). Toute autre cible échoue avec
`Unknown build target … Only 'component' is supported.`

Étapes de build d'un component :

1. Vérifier que la toolchain `wasm32-wasip2` est installée
2. `cargo build --target wasm32-wasip2 --lib`
3. Copier le component `.wasm` dans `output_dir`
4. Copier le runtime browser-glue dans `output_dir/browser-glue/`
5. Générer l'`index.html` de l'hôte (charge le component via browser-glue)

> **Prérequis** : lancer d'abord `npm run build` dans `packages/browser-glue/` pour compiler le
> code TypeScript du glue.

## Recommandations

- Écrivez les nouveaux projets directement contre `wit-bindings` et les mondes WIT ; il n'existe plus
  de chemin hérité imposant de « rester sur `web` d'abord, migrer ensuite ».
- Quand une nouvelle capacité navigateur est nécessaire, déclarez d'abord l'interface côté WIT, puis
  implémentez-la dans l'hôte JS — n'importez pas une bibliothèque de bindings DOM dans le guest.
- La forme des contrats d'API asynchrones suit les déclarations actuelles dans
  `packages/browser-worlds/wit/` ; cette page ne reprend pas les signatures individuelles.