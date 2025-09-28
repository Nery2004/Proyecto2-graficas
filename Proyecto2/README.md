# Proyecto2 (dentro de Proyecto2/)

Este README contiene instrucciones concretas para compilar, ejecutar y personalizar la escena
(arquitectura por capas) del proyecto.

## Requisitos

- Rust (rustup) y cargo.
- Windows: Powershell recomendado para los comandos de ejemplo.

## Compilar y ejecutar

```powershell
# Desde la carpeta Proyecto2
cargo build
cargo run
```

Para una versión optimizada:

```powershell
cargo build --release
cargo run --release
```

## Estructura relevante

- `src/main.rs` - código principal: carga texturas, crea materiales, parsea `assets/layer_*.txt` y
  spawnea cubos y faroles, además de controlar la cámara y la UI.
- `assets/` - texturas (.png) y archivos `layer_0.txt`, `layer_1.txt`, ... que definen la casa.

## Mapas ASCII (capas)

Cada `layer_X.txt` representa una capa en Y. Las filas y columnas del archivo corresponden a
las coordenadas Z y X en el mundo, respectivamente. Mantén las capas con el mismo ancho para
que la estructura quede centrada.

Caracteres comunes (ejemplo):
- `g` - grass (césped)
- `t` - tronco (madera vertical)
- `m` - madera (paredes)
- `w` - vidrio (transparente)
- `a` - agua
- `f` - farol (lantern)

## Añadir el enlace de YouTube (tu demo)

Si quieres que el README muestre un enlace o mini-preview de YouTube, reemplaza `VIDEO_ID`
por el identificador del vídeo.

Enlace directo (pegado tal cual):

https://youtu.be/VIDEO_ID

Enlace markdown:

[Demo en YouTube](https://youtu.be/VIDEO_ID)

Mini-preview usando la miniatura de YouTube (thumbnail):

[![Demo](https://img.youtube.com/vi/VIDEO_ID/0.jpg)](https://youtu.be/VIDEO_ID)

Sustituye `VIDEO_ID` por el valor correcto (parte después de `v=` en la URL o el segmento final
en una URL `youtu.be`).

---

Si quieres, puedo:
- Insertar tu `VIDEO_ID` ahora mismo y añadir la miniatura al README.
- Añadir un pequeño script (PowerShell) que abra el enlace en el navegador desde la carpeta del proyecto.
- Agregar badges o un gif corto demostrativo.
