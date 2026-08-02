# 🗺️ Hoja de Ruta: Microsoft Store & Automatización (CI/CD)

Este documento detalla el plan estratégico para profesionalizar la distribución de AnvilCraft Panel a través de la Microsoft Store y automatizar el flujo de lanzamientos.

## 🎯 Objetivos Principales
- [ ] **Distribución Dual**: Mantener instalador tradicional (NSIS) y paquete oficial (MSIX).
- [ ] **Automatización Total**: Empaquetado y firma de código automática mediante GitHub Actions.
- [ ] **Certificación MS Store**: Cumplir con todas las normativas técnicas y de seguridad de Microsoft.

---

## 🛠️ Fase 1: Arquitectura y Distribución Condicional
- [ ] **Lógica de Build Target**: Implementar detección en tiempo de ejecución para saber si la app corre como MSIX (Store) o NSIS (Standalone).
- [ ] **Sincronización de Actualizaciones**:
    - Si es **MS Store**: Ocultar el banner de actualización manual (delegar a Windows Store).
    - Si es **Standalone**: Mantener el sistema de banner inteligente y `tauri-plugin-updater`.
- [ ] **Auditoría de i18n**: Asegurar que el 100% de las cadenas estén traducidas (ES/EN) para la certificación.

## 🔒 Fase 2: Seguridad y Sandboxing
- [ ] **Cumplimiento de Permisos**: Verificar accesos a archivos para cumplir con el sandboxing de Windows Apps.
- [ ] **Secretos de Firma**: Configurar certificados PFX en los secretos de GitHub para la firma de binarios.

## 🚀 Fase 3: CI/CD con GitHub Actions
- [ ] **Workflow de Release**:
    - Disparador: Creación de un Tag (ej: `v0.1.11`).
    - Acción: Compilación simultánea de MSIX y NSIS.
    - Artefactos: Subida automática a GitHub Releases.
- [ ] **Automatización de Versionado**:
    - Script para inyectar la versión de 4 dígitos (`0.1.11.0`) en `tauri.conf.json` durante el build de la Store.

## 💎 Fase 4: Pulido de Identidad
- [ ] **Store Assets**: Integración de los iconos generados (Tile, Splash, StoreLogo) en la configuración final.
- [ ] **Store Listing**: Redacción de descripciones técnicas optimizadas para SEO en la Store.

---
*Documento creado el 10 de Mayo de 2026 para guiar la transición a la versión v0.1.11.*
