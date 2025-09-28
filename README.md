# Proyecto2-graficas

Este repositorio contiene el proyecto práctico de gráficos (Proyecto2) donde se usa Rust + Bevy
para construir escenas 3D a base de cubos texturizados y capas ASCII. El objetivo es que los
estudiantes apliquen conceptos de renderizado, PBR, iluminación y creación procedimental de
escenas.

## Contenido

- `Proyecto2/` - Código fuente del proyecto en Rust (Bevy). Contiene la aplicación principal,
  assets (texturas, capas ASCII) y los recursos necesarios para compilar y ejecutar.

## Requisitos

- Rust toolchain (rustc + cargo). Se recomienda instalar desde https://rustup.rs.
- GPU con drivers actualizados para pruebas en tiempo real (opcional pero recomendado).

## Compilar y ejecutar

Desde la raíz del repo puedes entrar a la carpeta del proyecto y compilar:

```powershell
cd Proyecto2
cargo build --release
cargo run --release
```

Para desarrollo rápido usa `cargo run` (perfil dev).

## Controles (por defecto)

- Rotar cámara: clic izquierdo y arrastrar.
- Zoom: rueda del ratón o PageUp/PageDown.
- Pan horizontal: mantener clic derecho y arrastrar.
- UI: hay un botón en la esquina superior derecha para alternar día/noche.

## Layouts y generación por capas

La casa y otros elementos de la escena se generan leyendo archivos ASCII por capas.
Los archivos se encuentran en `Proyecto2/assets/` y siguen el patrón `layer_0.txt`, `layer_1.txt`, ...
Cada archivo representa una 'altura' (Y) y cada carácter mapea a un material/cubo.

Consejos:
- Mantén las capas con el mismo número de columnas para que queden centradas correctamente.
- Los caracteres reconocidos (por convención) incluyen, por ejemplo: `g` (grass), `t` (tronco),
  `m` (madera), `w` (vidrio), `a` (agua), `f` (farol), etc. Revisa `Proyecto2/src/main.rs` para
  la tabla actual de mapeo.

## Enlace Video:

  https://youtu.be/oZTX7kd2n38

