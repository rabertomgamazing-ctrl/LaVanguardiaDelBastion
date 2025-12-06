# LaVanguardiaDelBastion

Motor base en Rust para un cliente Tauri scrimdark (inspirado en Darkest Dungeon) que combina los protocolos del Códice Bastión, un bucle de juego con turnos de amenaza cada 24 párrafos y un coordinador de IA híbrida (Ollama, Gemini y APIs externas).

## Módulos principales
- `protocol`: carga los modos `ENGINE FULL/LITE` y `PARTIDA_ESTÁNDAR/RELATO_LIBRE` desde `BastionLAN_Core.yaml`, aplicando el guard que exige solicitud explícita para el modo libre.
- `session`: gestiona párrafos, relojes (8 pasos) y marcas (24 segmentos) y dispara Turno de Amenaza cada 24 párrafos.
- `dice`: tiradas d4–d20 y chequeos con modificador para visualizar dados físicos más adelante.
- `events`: mapea eventos del motor a presets audiovisuales (FX y audio) que el frontend puede consumir.
- `events::EventSink`: interfaz para emitir eventos hacia el frontend (Tauri u otro) con `MemoryEventSink` para pruebas.
- `ai`: coordinador de proveedores (`Ollama`, `Gemini`, `ExternalApi`) con roles A/B/C del Arquitecto IA.
- `persistence`: serializa/deserializa campañas en JSON para slots locales.
- `world`: catálogo de Marcas/Facciones (27–32) con los ejércitos listos para desplegar.

## Próximos pasos
- Integrar comandos Tauri que expongan los eventos y estados al frontend.
- Añadir shaders/FX reales y bancos de sonido reaccionando a los eventos ya definidos.
- Conectar llamadas reales a proveedores IA (Ollama local, Gemini remoto y servicios de FX/TTS).

Para validar la base actual ejecuta:

```bash
cargo test
```

## Hoja de ruta de mejoras seleccionables
1. **Comandos Tauri para eventos**: exponer `events::EngineEvent` hacia el frontend con `tauri::AppHandle` y canales de emisión.
2. **Shaders scrimdark**: integrar un canvas WebGL con fragment shaders para viñeta, niebla y pulsos rúnicos ligados a cambios de estado.
3. **Banco de sonido reactivo**: asociar presets de audio (campanas, golpes, drones) a cada evento y permitir mezcla dinámica desde Rust.
4. **Animación de dados físicos**: renderizar tiradas 3D/2D en frontend al usar `dice::roll` y reproducir audio de impacto.
5. **UI de modo activo**: banner permanente que muestre `PARTIDA_ESTÁNDAR/RELATO_LIBRE` y `ENGINE FULL/LITE` con tooltips diegéticos.
6. **Guard rails de modo**: añadir middleware que bloquee comandos de `Relato Libre` salvo petición explícita registrada en logs.
7. **Integración Ollama**: llamada real a modelo local afinado scrimdark para rol A (Narrador) con streaming de texto.
8. **Integración Gemini**: usar Gemini para planeación/validación (rol B) y combinar su salida con el narrador local.
9. **API externa de FX/TTS**: consumir un servicio REST para generar efectos o voz procesada con latencia mínima.
10. **Caché de lore**: embeddings locales del YAML central y Tomos, con recuperación semántica previa a cada turno narrativo.
11. **Gestor de campañas**: UI para slots, backups y migraciones versionadas usando `persistence::CampaignState`.
12. **Visor de tablas**: panel interactivo con Hoja 1, Hoja 7, Misiones 5 PM, Relojes y Marcas con búsqueda.
13. **Panel de viaje rúnico**: mapa por Coronas de Distancia con opción de salto si hay nodo activado y coste de PC/Favor Enano.
14. **Relojes y marcas visuales**: HUD con segmentos llenables (8 pasos/24 marcas) y alertas cromáticas al avanzar.
15. **Turno de Amenaza**: pantalla/overlay especial que solo se dispara cada 24 párrafos si hay una Marca del Cronista activa en ese tramo; incluir un mensaje reactivo por cada Marca que avanza peligrosamente en el reloj.
16. **Sala de calibración audiovisual**: espacio de prueba para volumen, contraste y accesibilidad (reducir parpadeos/ruido).
17. **Logs diegéticos**: bitácora en-world (voz del Cronista) que registre cambios de modo, tiradas y activaciones de nodos.
18. **Modo desarrollador**: toggle oculto para simular fallos de proveedores IA y forzar `ENGINE LITE` o eventos especiales.
19. **CLI Lore Linter**: herramienta Rust para validar que los textos generados no inventen Casas/Marcas y respeten la regla de 24 párrafos.
20. **Pruebas automatizadas (Modo C)**: suites en Modo C que simulan sesiones completas y verifican adherencia a protocolos y conteo de párrafos/amenazas.

## Marcas (facciones) y ejércitos disponibles

Los identificadores 27–32 del YAML principal se exponen en `world::core_faction_marks()` con sus ejércitos listos para usar en prompts o UI:
- **Tormentas Elementales (27, FACCION.ELEMENTAL)**: Guardia del Trueno (infantería pesada que canaliza rayos) y Cohorte de Relámpagos (zapadores rúnicos y artillería de bobinas).
- **Campamento de Bandidos (28, FACCION.BANDIDOS)**: Hermanas del Camino Polvoriento (hostigamiento montado) y Mercenarios del Estandarte Roto (veteranos deshonrados que rehacen líneas).
- **Guarida de Monstruo (29, FACCION.MONSTRUO)**: Cacería de Lanzas Negras (caza mayor) y Custodios del Foso (contención defensiva).
- **Bosque Encantado/Oscuro (30, FACCION.BOSQUE)**: Guarda de Corteza (control de terreno feérico) y Ronda de Linternas Marchitas (desorientar y aislar patrullas).
- **Plaga Verde (31, FACCION.PLAGA)**: Monjes Pálidos (contención sanitaria) y Brigada de Cal viva (sellar focos con fuego/cal).
- **Escuela del Silencio / Tinta (32, FACCION.TINTA)**: Alumnas del Silencio (operaciones encubiertas) y Custodios del Bastión Negro (guardia élite de nodos rúnicos).

## Factores del modo A (Narrador) ya aplicados

- **Intro scrimdark** y **firma del Bastión** se entregan al Narrador a través de `ModeAOnboarding::default().as_prompts()` para anclar el tono y recordar que Kaelen mantiene el mando.
- **Cuenta de 24 párrafos narrativos**: se inyecta en el prompt de modo A con `paragraph_limit` usando `PARAGRAPHS_PER_THREAT` para respetar el Turno de Amenaza.
- **Misión de tutorial**: el prompt de arranque obliga a ejecutar una escolta de convoy con tiradas visibles y consecuencias en Relojes antes de avanzar a misiones libres.

## Integración reactiva por eventos

- Usa `BastionEngine::with_event_sink` para conectar un emisor (por ejemplo, un comando Tauri) y recibir `EventEnvelope` en cada párrafo, cambio de modo o avance de marcas/relojes.
- `MemoryEventSink` facilita pruebas o prototipos de UI al almacenar la secuencia completa de eventos generados.
- `SessionManager::apply_mark` devuelve el estado actualizado de las Marcas para disparar FX/sonido asociados a las facciones activas.
