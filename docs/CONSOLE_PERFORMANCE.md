# Optimización y Escalamiento de la Consola

Este documento detalla las estrategias recomendadas para escalar el historial de la consola en AnvilCraft, priorizando el rendimiento en equipos de bajos recursos.

## Estado Actual (v0.1.x)
- **Líneas en Memoria:** 2,000 líneas (almacenadas en `appState`).
- **Líneas Renderizadas:** 500 líneas (usando un simple `{#each}` en `ConsoleView.svelte`).
- **Limitación:** El rendimiento del DOM se degrada linealmente con el número de elementos. Superar las 1,000 líneas visibles puede causar lag en el scroll o al redimensionar la ventana.

## Recomendaciones para Escalamiento Futuro

### 1. Virtual Scrolling (Prioridad Alta)
La mejora más importante para manejar historiales grandes (10,000+ líneas) sin costo de rendimiento.
- **Concepto:** Solo se crean elementos HTML para las líneas que son visibles en el área de scroll actual.
- **Implementación:** Usar `svelte-virtual-list` o una implementación personalizada que calcule el offset del scroll.
- **Beneficio:** El uso de CPU y RAM se mantiene constante independientemente de si hay 1,000 o 1,000,000 de líneas.

### 2. Batched Updates (Actualizaciones por Lotes)
Durante el inicio del servidor o procesos intensivos, los logs pueden llegar muy rápido (ráfagas).
- **Problema:** Cada `logs.push` dispara un ciclo de re-renderizado en Svelte.
- **Solución:** Implementar un buffer temporal. En lugar de actualizar el estado con cada línea, acumular las líneas y actualizar el estado cada 100ms o 200ms.
- **Beneficio:** Reduce drásticamente la carga de CPU durante momentos de alta actividad.

### 3. Lazy Formatting (Formateo Perezoso)
Actualmente, `formatLog` y la conversión ANSI a HTML se ejecutan al renderizar.
- **Solución:** Combinar con Virtual Scrolling para procesar el texto ANSI solo cuando la línea entra en el "viewport" del usuario.
- **Beneficio:** Ahorra ciclos de procesamiento en logs que el usuario quizás nunca llegue a ver.

### 4. Off-heap Storage (Streaming a Disco)
Mantener 100,000 líneas en el `appState` (RAM) eventualmente consumirá mucha memoria.
- **Solución:** Mantener un buffer circular de ~1,000 líneas en RAM para visualización rápida y leer el resto desde el archivo `.log` físico en el disco de forma asíncrona solo cuando el usuario haga scroll hacia arriba (Infinite Loading).

### 5. Web Workers para Filtrado
Si se añade una función de "Búsqueda" o "Filtro" en el historial:
- **Solución:** Realizar la búsqueda y el filtrado de strings pesados en un Web Worker para evitar congelar la interfaz principal (Main Thread).

---
*Documento generado para referencia técnica de AnvilCraft.*
