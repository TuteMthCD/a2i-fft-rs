# Audio Processor

Herramienta CLI en Rust que convierte un archivo de audio en una imagen PNG estilo espectrograma.

## Requisitos
- Rust y Cargo (edición 2024 o superior)
- ffmpeg disponible en el `PATH`

## Uso rápido
```bash
cargo run --release -- -i ruta/al/audio.mp3 -o salida.png
```

El comando lee el archivo de entrada (`-i`) y genera una imagen PNG con el mapa de frecuencias en la ruta indicada por `-o`. Si no se especifica `-o`, el resultado se guarda como `./a.png`.

## Parámetros disponibles
- `-i`, `--input` (obligatorio): ruta del audio a procesar.
- `-o`, `--output`: ruta del PNG de salida. Por defecto `./a.png`.
- `-r`, `--sample-rate`: frecuencia de muestreo objetivo en Hz. Por defecto `44100`.
- `-d`, `--downsample`: factor para reducir la resolución horizontal del espectrograma. Por defecto `128`.
- `-j`, `--jobs`: cantidad de hilos que ffmpeg utilizará al decodificar. Por defecto `16`.

## Cómo funciona
1. `ffmpeg` convierte el audio de origen a una señal mono en formato `f32le` con la frecuencia indicada.
2. El programa agrupa los samples en bloques de un segundo y ejecuta una FFT sobre cada uno para obtener el espectro de frecuencias.
3. Las magnitudes se normalizan, se aplican ajustes de gamma y se promedian en bloques de tamaño `--downsample`.
4. Con esos valores se arma un buffer RGB y se guarda como PNG utilizando la crate `image`.

## Desarrollo
- Ejecutá `cargo test` para validar los módulos auxiliares.
- Durante el desarrollo podés invocar `cargo run -- -i <archivo>` para generar imágenes con distintos parámetros y verificar el resultado.

## Notas
- Si `ffmpeg` no está instalado, el programa fallará al intentar extraer los samples.

### TODO!
- hacer eje x en escala log
