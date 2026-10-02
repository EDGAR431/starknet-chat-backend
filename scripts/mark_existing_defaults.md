# Fixup: proteger los sonidos por defecto ya creados

Los 6 sonidos por defecto (Ding, Pop, Chime, Buzz, Boop, Whoosh) que ya se
crearon en producción (mediante las pruebas en vivo + el script de backfill)
se crearon **antes** de que existiera el campo `is_default`, así que en la
base de datos real estos documentos tienen `is_default: false` (o no tienen
el campo). Esto significa que, aunque el código ya está desplegado,
**esos sonidos concretos todavía se pueden borrar** hasta que se ejecute
este comando una sola vez.

## Comando a ejecutar en el VPS

Conéctate al VPS por SSH y ejecuta (ajusta el nombre del contenedor de Mongo
si no es `stoat-database-1`; puedes verlo con `docker ps` y buscando el
servicio `database`):

```bash
docker exec -it stoat-database-1 mongosh revolt --eval '
db.soundboard_sounds.updateMany(
  { name: { $in: ["Ding","Pop","Chime","Buzz","Boop","Whoosh"] } },
  { $set: { is_default: true } }
)
'
```

Si tu imagen de Mongo es antigua y no tiene `mongosh` (shell nuevo), usa el
shell clásico `mongo` en su lugar:

```bash
docker exec -it stoat-database-1 mongo revolt --eval '
db.soundboard_sounds.updateMany(
  { name: { $in: ["Ding","Pop","Chime","Buzz","Boop","Whoosh"] } },
  { $set: { is_default: true } }
)
'
```

Si no conoces el nombre exacto del contenedor, averígualo primero con:

```bash
docker ps --format '{{.Names}}'
```

y sustituye `stoat-database-1` por el nombre real del contenedor que
ejecuta el servicio `database` (Mongo).

## Qué hace

Marca como `is_default: true` cualquier documento de la colección
`soundboard_sounds` cuyo `name` sea uno de los 6 nombres de los sonidos del
pack por defecto, en cualquier soundboard (de cualquier usuario o servidor).
A partir de ahí, la ruta de borrado (`DELETE /sound/<id>`) los rechaza
siempre, sin importar quién lo intente ni los permisos que tenga.

## Nota de seguridad

Este `updateMany` filtra solo por `name`, así que si algún usuario llegó a
crear (antes de este cambio) su propio sonido personalizado con exactamente
uno de esos 6 nombres, ese sonido también quedaría protegido como "default"
tras ejecutar este comando. Dado el bajo riesgo y que esos nombres son
genéricos, se considera un efecto secundario aceptable para este fixup
puntual; si se prefiere más precisión, se podría añadir también un filtro
por `creator_id` igual al bot de sistema configurado en
`system_bot_token`, pero requeriría conocer su user id de antemano.
