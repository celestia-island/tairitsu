# Hub de documentation Tairitsu (Français)

> Framework full-stack propulsé par le modèle de composants WASM

## Pour commencer

| Document | Description |
|:--|:--|
| [Tutoriel de démarrage](getting-started.md) | De zéro à une application full-stack fonctionnelle. Couvre `tairitsu new`, votre premier composant, exécution serveur + client et déploiement. |
| [Démarrage rapide](quick-start.md) | Installation et vérification en 5 minutes. |
| [Carte du workspace](workspace-map.md) | Tour de la structure du monorepo. |
| [Build, test et release](build-test-release.md) | Comment utiliser les recettes `just` pour le flux de développement. |

## Migration

| Document | Description |
|:--|:--|
| [De web-sys aux liaisons WIT](migration.md) | Transition de `wasm-bindgen`/`web-sys` vers les liaisons WIT du Component Model. |

## Référence

| Document | Description |
|:--|:--|
| [Glossaire](glossary.md) | Termes clés : WIT, Component Model, VNode, Signal, Platform, Container, etc. |
| [Dépannage](troubleshooting.md) | Problèmes courants et solutions. |

## Architecture

| Document | Description |
|:--|:--|
| [Vue d'ensemble du système](../system/overview.md) | Architecture en quatre couches : Interface → Runtime → Platform → Tooling |
| [Runtime et modèle de conteneur](../system/runtime.md) | Cycle de vie Image/Container/Registry, liaison WIT, invocation dynamique |
| [VDOM et rendu](../system/vdom.md) | Diffing du DOM virtuel, patching, système d'événements, planificateur réactif |
| [Pipeline W3C WebIDL → WIT](../system/wit-pipeline.md) | Comment 50+ spécifications WebIDL deviennent des interfaces WIT |
| [Backends web doubles](../system/web-backends.md) | Stratégie WitPlatform vs WebPlatform |
| [Architecture Browser Glue](../system/browser-glue.md) | Couche TypeScript connectant l'ABI WIT au DOM |
| [Stratégie de versionnage](../system/versioning.md) | Versionnage sémantique dans le workspace multi-crate |

## Référence des paquets

| Document | Description |
|:--|:--|
| [Vue d'ensemble des paquets en couches](../components/index.md) | Hiérarchie des Crates en quatre couches avec graphe de dépendances |
| [Liste des paquets du workspace](../components/packages.md) | Description détaillée de chaque Crate |
