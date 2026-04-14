# Registro de Desarrollo (Dev Log)

Este archivo sirve como bitácora local y privada para llevar el seguimiento de los avances y tareas pendientes del proyecto. Al estar en `.gitignore`, no se subirá a GitHub.

## Historial de Cambios Recientes

- **2026-04-13** | **v0.1.3 Beta**:
    - Edición de nombre de instancia in-situ (display name) desde detalles.
    - Sincronización bidireccional del puerto entre el Panel y `server.properties`.
    - Unificación profesional del identificador de la aplicación (`AnvilCraftPanel`).
    - Reparación del sistema de descarga de Java 16 (vía binarios JDK archivados).
    - Inclusión de **Java 25 (LTS)** en la lista de entornos portátiles.
    - Mejora de legibilidad y descripciones en la sección de Java Portátil (Adoptium).
    - Corrección de desbordamiento y sincronización de scroll en la consola.
    - Localización completa del tooltip "Abrir Carpeta" en el detalle de instancia.
- **2026-04-12** | **v0.1.2 Beta**: 
    - Implementación completa de internacionalización (ES/EN) con `svelte-i18n`.
    - Unificación del sistema de notificaciones (Toasts) y eliminación de alertas nativas.
    - Mapeo localizado de estados de progreso desde el backend (Rust).
    - Consolidación de identidad visual y versión unificada en todo el panel.
    - Rediseño minimalista de la sección "Acerca de" y barra superior.
- **2026-01-21** | **UI & Layout**: Refactorización y optimización de rendimiento para la consola, la página de inicio (`Home`) y el layout principal.
- **2026-01-21** | **Forge**: Mejoras en el progreso de instalación de Forge con streaming en tiempo real, mejoras visuales y guardado de logs persistentes.
- **2026-01-21** | **Consola**: Autocompletado de comandos y versionado, soporte avanzado para argumentos y almacenamiento de datos optimizado.
- **2026-01-18** | **Manejo de Instancias**: Borrado de instancias implementado y mejora de la selección de versiones usando API de CorpMore.
- **2026-01** | **Mantenimiento**: Se añadió Licencia MIT, `README.md` actualizado y disclaimer sobre el uso de IA.

---

## Tareas Pendientes (TODO)

- [x] Integración entre la UI de creación/ajuste de instancias (`CreateInstanceModal.svelte`, `InstanceSettings.svelte`) y el backend en Rust (`java.rs`, `version.rs`, `mod.rs`).
- [ ] Optimizar el sistema de logs para evitar redundancia en el guardado.
- [ ] Revisión de accesibilidad en el panel de skins.
