# Desplegar el backend en el VPS (starknet-chat.tech)

Servidor: `root@45.133.178.187`, backend en `~/stoat` (docker compose).
El compose usa **imagenes propias** (no las oficiales de stoatchat):

| Servicio | Imagen | Se construye con |
|---|---|---|
| `api` | `starknet-chat-backend-delta:v0.16.2-merge` | `crates/delta/Dockerfile` |
| `events` | `starknet-chat-backend-events:v0.16.2-merge` | `crates/bonfire/Dockerfile` |

El resto (autumn, january, pushd, crond, voice-ingress...) sigue en imagenes oficiales v0.15.5.
Si un cambio solo toca la API (rutas, MFA, limites), basta reconstruir **api**.

## Metodo habitual: binario precompilado (el VPS NO compila)

El VPS no tiene potencia para compilar Rust. El binario `revolt-delta` se compila aparte
(`LIBZ_SYS_STATIC=1 cargo build --release -p revolt-delta`, ~15 min en 2 nucleos; el
`LIBZ_SYS_STATIC=1` evita enlazar `libz` dinamicamente, que la imagen distroless no trae)
y se entrega como `starknet-chat-backend-delta-XXXX.zip` con `target/release/revolt-delta` +
`Dockerfile.package-delta`. Comprobar con `ldd` que solo depende de libssl, libcrypto,
libgcc_s, libm y libc.

```powershell
scp -i $env:USERPROFILE\.ssh\id_ed25519_stoat starknet-chat-backend-delta-XXXX.zip root@45.133.178.187:/root/
```

```bash
cd ~ && rm -rf delta-deploy && mkdir delta-deploy && unzip -q starknet-chat-backend-delta-XXXX.zip -d delta-deploy
docker tag starknet-chat-backend-delta:v0.16.2-merge starknet-chat-backend-delta:antes-$(date +%Y%m%d-%H%M)
cd delta-deploy && docker build -t starknet-chat-backend-delta:v0.16.2-merge -f Dockerfile.package-delta .
cd ~/stoat && docker compose up -d --force-recreate api && docker compose logs --tail 30 api
```

Las secciones siguientes describen compilar dentro del VPS (lento; solo como alternativa).
Para volver atras, ver la seccion 4.

## 1. Subir el codigo (desde el PC)

```powershell
scp -i $env:USERPROFILE\.ssh\id_ed25519_stoat starknet-chat-backend-XXXX.zip root@45.133.178.187:/root/
```

## 2. Construir y desplegar (en el VPS)

La primera compilacion de Rust tarda mucho (30-90 min en un VPS pequeno).
Las siguientes reutilizan la cache de dependencias. Lanzarlo dentro de `tmux`
para que no se corte si se cierra la sesion SSH: `tmux new -s build`.

```bash
# 2.1 punto de retorno (imagen actual)
docker tag starknet-chat-backend-delta:v0.16.2-merge starknet-chat-backend-delta:antes-$(date +%Y%m%d-%H%M)

# 2.2 codigo limpio en carpeta nueva (no mezclar con versiones anteriores)
rm -rf ~/backend-src && mkdir ~/backend-src
unzip -q ~/starknet-chat-backend-XXXX.zip -d ~/backend-src
cd ~/backend-src

# 2.3 imagen base (compila todo; usar 2 jobs por la RAM)
docker build --build-arg CARGO_BUILD_JOBS=2 -t ghcr.io/stoatchat/base:latest -f Dockerfile.useCurrentArch .

# 2.4 imagen final de la API (con el MISMO tag que usa compose.yml)
docker build -t starknet-chat-backend-delta:v0.16.2-merge - < crates/delta/Dockerfile

# 2.5 recrear solo la API
cd ~/stoat && docker compose up -d --force-recreate api
docker compose logs --tail 30 api
```

Si tambien cambio `bonfire` (eventos/websocket), repetir 2.4 y 2.5 con:
`docker build -t starknet-chat-backend-events:v0.16.2-merge - < crates/bonfire/Dockerfile` y `... --force-recreate events`.

## 3. Comprobar

- `docker compose ps` -> `api` en `Up`.
- `curl -s https://starknet-chat.tech/api/ | head -c 300` responde JSON.
- Probar el cambio en la web (ver el commit/PR correspondiente).

## 4. Volver atras

```bash
docker images starknet-chat-backend-delta   # elegir el tag "antes-..."
docker tag starknet-chat-backend-delta:antes-YYYYMMDD-HHMM starknet-chat-backend-delta:v0.16.2-merge
cd ~/stoat && docker compose up -d --force-recreate api
```

## Alternativa: construir fuera del VPS y cargar la imagen

Asi se desplegaron las versiones anteriores (`*.tar.gz` en `~`):

```bash
# en una maquina con Docker
docker save starknet-chat-backend-delta:v0.16.2-merge | gzip > delta.tar.gz
scp delta.tar.gz root@45.133.178.187:/root/
# en el VPS
gunzip -c ~/delta.tar.gz | docker load
cd ~/stoat && docker compose up -d --force-recreate api
```

> Nota: la receta de la seccion 2 sale de los Dockerfile del repo; si algo difiere de lo que
> corre en produccion, actualizar este archivo en el mismo cambio.
