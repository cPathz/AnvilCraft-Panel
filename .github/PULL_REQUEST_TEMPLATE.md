## ¿Qué cambia?

<!-- Descripción breve de los cambios. -->

## Tipo de cambio

- [ ] Bug fix (cambio que arregla un problema)
- [ ] Nueva feature (cambio que agrega funcionalidad)
- [ ] Breaking change (cambio que rompe compatibilidad)
- [ ] Documentación
- [ ] Refactor (cambio que no arregla bug ni agrega feature)

## ¿Cómo se prueba?

<!-- Describe cómo probaste los cambios. -->

## Checklist

- [ ] El código compila sin warnings nuevos
- [ ] Si agregaste UI strings, los agregué a `src/lib/locales/en.json`
- [ ] No incluí dev debris (logs, scratch/, .claude/, etc.)
- [ ] No rompí la sync de versiones (si tocas versiones, asegúrate de que coincidan en `package.json`, `Cargo.toml` y `tauri.conf.json`)
- [ ] Si agregaste un loader, lo registré en `src-tauri/src/loaders/registry.rs` Y en `src/lib/loaders/catalog.ts`

## Issues relacionadas

<!-- Referencia issues relacionadas con `Fixes #123` o `Closes #456`. -->
