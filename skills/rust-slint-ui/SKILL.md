---
name: rust-slint-ui
description: "Trigger: slint, ui, declarative interface, bidirectional bindings, native widgets. Directrices para interfaces declarativas en Slint, enlaces de propiedades bidireccionales y optimización de widgets nativos sin dependencias web."
license: Apache-2.0
metadata:
  author: gentle-ai-orchestrator
  version: "1.0"
---

## Activation Contract

Activa esta skill cuando se escriba o modifique UI declarativa con Slint en el reproductor de música Rust (Slint + Rodio). Cubre: archivos `.slint`, propiedades de estado, bindings bidireccionales, callbacks, optimización de widgets nativos y evita cualquier dependencia web (no HTML/CSS/JS en la UI).

## Hard Rules

- Toda la UI vive en archivos `.slint`; no se permiten componentes web embebidos, webviews ni dependencias de navegador para la interfaz principal.
- Los bindings bidireccionales se declaran con `<->` en Slint; nunca se sincroniza estado manualmente entre Rust y Slint salvo que un binding explícito lo impida.
- Los widgets nativos de Slint (Button, Slider, VerticalBox, HorizontalBox, LineEdit, Image, ListView) se usan sin envolver en contenedores web.
- Los colores, tipografía y espaciado se definen en Slint con su sistema de theming nativo, no con CSS.
- Cada cambio de UI debe compilar con `cargo check` y no introducir dependencias nuevas de crates web.

## Decision Gates

| Necesidad | Acción |
|-----------|--------|
| Pantalla o componente nuevo | Crear archivo `.slint` propio y registrarlo en `main.rs` o el módulo de UI correspondiente. |
| Estado compartido UI ↔ Rust | Usar propiedad Slint con `<->` binding o callback `=>` para eventos. |
| Estilos visuales | Usar theming nativo de Slint; no crear archivos CSS. |
| Optimizar renderizado | Evitar reconstruir árboles enteros; usar componentes reutilizables y bindings locales. |

## Execution Steps

1. Definir el componente en `skills/rust-slint-ui/assets/template.slint` o directamente en el módulo de UI del proyecto.
2. Declarar propiedades con `<->` para estado bidireccional (por ejemplo, volumen, posición de reproducción).
3. Conectar callbacks `=>` desde Slint a funciones Rust para acciones (play, pause, siguiente, anterior).
4. Verificar que no hay imports de crates web en `Cargo.toml` relacionados con la UI.
5. Ejecutar `cargo check` para validar compilación.

## Output Contract

- Archivos `.slint` creados o modificados.
- Bindings `<->` y callbacks `=>` conectados.
- `Cargo.toml` sin dependencias web para UI.
- Resultado de `cargo check` limpio.

## References

- `AGENT.md` — reglas del orquestador y estilo de respuesta.
