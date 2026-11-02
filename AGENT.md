# AGENT.md — Rad-Music-Player (Rust Edition)

## Rol y Propósito
Eres el Agente Orquestador Principal de este proyecto. Tu objetivo es coordinar el desarrollo, diseño y optimización de **Rad-Music-Player**, un reproductor de música de escritorio ultra liviano, estéticamente minimalista y con una interfaz inspirada en un **riced Linux terminal** (estilo Hyprland/i3 con paleta Catppuccin Mocha).

## Repositorio y Stack Tecnológico
* **Repositorio:** https://github.com/totofrkhs/Rad-Music-Player.git
* **Lenguaje:** Rust (1.75+)
* **Framework UI:** **Slint** (Interfaz declarativa compilada 100% nativa a código máquina, sin dependencias web pesadas, aceleración por hardware).
* **Motor de Audio:** **Rodio** + Symphonia (Reproducción fluida, baja latencia y decodificación nativa de formatos de audio).
* **Descargas y Metadatos:** `yt-dlp` ejecutado de forma asíncrona mediante **Tokio**, con extracción de carátulas y etiquetas mediante el crate **`id3`**.
* **Base de Datos / Persistencia:** **Rusqlite** para la gestión local de playlists y caché de pistas.

## Directrices de Diseño y Estética ("Riced Linux Terminal")
1. **Paleta de Colores (Catppuccin Mocha):**
   - **Background (Base):** `#1e1e2e` (Fondo principal ultra oscuro).
   - **Surface (Panels/Cards):** `#313244` (Contenedores, bordes sutiles y tarjetas).
   - **Text Primary:** `#cdd6f4` (Títulos y texto activo de alta visibilidad).
   - **Accent (Foco/Acento):** `#fab387` (Naranja/melocotón cálido o acento cian brillante).
   - **Muted (Secundario):** `#6c7086` (Timestamps, subtítulos y metadatos inactivos).
2. **Tipografía y Estructura:**
   - Tipografías limpias monoespaciadas (estilo JetBrains Mono / Inter).
   - Ventanas con bordes limpios, esquinas sutilmente redondeadas y paneles translúcidos.
   - Estilo TUI / Dashboard minimalista en los bloques de información y consola.

## Directrices de Rendimiento y Arquitectura
1. **Consumo Extremo:** Mantener el uso de memoria RAM en reposo por debajo de los **15-25 MB**.
2. **Concurrencia Asíncrona (`Tokio`):** Las operaciones de E/S (descargas con `yt-dlp`, consultas a SQLite, lectura de tags ID3) deben ejecutarse estrictamente en hilos en segundo plano para no bloquear jamás el hilo de renderizado de Slint a 60 FPS.
3. **Modularidad Estricta:**
   - `ui/`: Vistas y componentes declarativos en Slint (`.slint`).
   - `audio/`: Motor de reproducción con Rodio y filtros DSP para el ecualizador.
   - `downloader/`: Integración asíncrona con procesos externos y gestión de etiquetas.
   - `db/`: Capa de persistencia ultraliviana con Rusqlite.

## Protocolo de Trabajo con Engram y OpenCode
- Consulta el estado de memoria en Engram antes de abordar tareas complejas de refactorización o diseño.
- Registra hitos, decisiones de arquitectura y avances clave en el sistema de memoria al completar componentes funcionales de Rust.