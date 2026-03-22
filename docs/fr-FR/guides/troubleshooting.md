# Guide de dépannage

Problèmes courants et solutions lors de l'utilisation de Tairitsu browser-glue et du Component Model.

## Erreurs de compilation

### Cible wasm32-wasip2 introuvable

**Erreur :**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**Solution :**
```bash
rustup target add wasm32-wasip2
```

### Incompatibilité de version wit-bindgen

**Erreur :**
```
error: failed to select a version for `wit-bindgen`
```

**Solution :**
Vérifiez que la version de `wit-bindgen` correspond dans `Cargo.toml` :
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### Erreurs de compilation TypeScript

**Erreur :**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**Solution :**
Régénérez la glue et recompilez :
```bash
cd packages/browser-glue
npm run build
```

## Erreurs d'exécution

### Imports hôte manquants

**Erreur :**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**Solution :**
1. Vérifiez que l'import map est configuré :
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. Vérifiez que les fichiers browser-glue existent dans le répertoire de sortie.

### Échec d'initialisation du composant

**Erreur :**
```
Error: Component instantiation failed: undefined import
```

**Solution :**
Vérifiez que tous les imports WIT requis ont des implémentations correspondantes dans browser-glue.

### Erreurs de transpilation jco

**Erreur :**
```
Error: Failed to transpile component
```

**Solution :**
1. Vérifiez que jco est installé :
```bash
npm install -g @bytecodealliance/jco
```

2. Vérifiez que le composant WASM est valide :
```bash
wasm-tools print component.wasm
```

## Techniques de débogage

### Activer les logs de débogage

Dans la console du navigateur :
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### Inspecter les liaisons WIT

Afficher les liaisons générées :
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### DevTools du navigateur

1. Ouvrez DevTools (F12)
2. Vérifiez la Console pour les erreurs
3. Onglet Network pour les modules non chargés
4. Onglet Sources pour le débogage

### Validation du composant

```bash
# Valider la structure du composant
wasm-tools validate component.wasm

# Afficher le contenu du composant
wasm-tools print component.wasm
```

## Problèmes courants

### Handle introuvable

**Symptôme :** `null` retourné par les opérations DOM

**Cause :** Le handle a été collecté par le garbage collector ou non enregistré

**Solution :** Assurez-vous que les éléments restent référencés en JavaScript

### Événement non déclenché

**Symptôme :** Les gestionnaires d'événements ne sont pas appelés

**Cause :** Inadéquation de l'ID d'écouteur ou type d'événement incorrect

**Solution :** Vérifiez que `addEventListener` retourne un ID d'écouteur valide

### Fuites de mémoire

**Symptôme :** Augmentation de l'utilisation mémoire au fil du temps

**Cause :** Handles non libérés après utilisation

**Solution :** Appelez `dropHandle()` une fois les objets traités

## Problèmes de performance

### Chargement lent du composant

**Solutions :**
1. Utilisez le build release : `cargo build --release`
2. Activez LTO dans `Cargo.toml` :
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### Latence élevée des événements

**Solutions :**
1. Évitez les opérations synchrones dans les gestionnaires
2. Utilisez `requestAnimationFrame` pour les mises à jour visuelles
3. Debouncez les événements rapides

## Obtenir de l'aide

1. Consultez les issues existantes : https://github.com/anomalyco/opencode/issues
2. Revoyez la documentation dans le répertoire `docs/`
3. Examinez le code d'exemple dans `examples/website/`
