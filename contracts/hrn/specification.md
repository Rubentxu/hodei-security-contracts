# Hodei Resource Name (HRN) — contrato canónico v2

**Estado:** normativo para el MVP de seguridad. **Versión del contrato:** `hrn:v2`.
**Implementación de referencia:** `crates/hodei-hrn`. **Vectores:** [`vectors.json`](vectors.json).
**Migración:** [`migration-matrix.md`](migration-matrix.md).

Las palabras **DEBE**, **NO DEBE**, **PUEDE** se interpretan como en RFC 2119.

## 1. Alcance

Un HRN identifica un **recurso** de forma estable, comparable y sin colisiones entre cuentas, regiones, servicios y particiones. El HRN es una identidad, **no una autorización**: conocer o acertar un HRN no concede ningún permiso.

Quedan fuera de este contrato:

- la resolución remota del recurso;
- la evaluación de políticas Cedar;
- la persistencia y su esquema;
- las **acciones**, que tienen una identidad distinta (§7);
- la normalización Unicode: el contrato es ASCII y no reescribe datos.

## 2. Gramática

```abnf
hrn            = "hrn:" partition ":" service ":" region ":" account ":" resource
resource       = resource-type "/" resource-path
partition      = name
service        = service-start *service-char
region         = "" / name
account        = account-id
resource-type  = type-name
resource-path  = segment *( "/" segment )

name           = name-start *name-char
name-start     = %x61-7A / DIGIT          ; a-z 0-9
name-char      = name-start / "-"

service-start  = ALPHA / DIGIT
service-char   = ALPHA / DIGIT / "-"

account-id     = account-start *account-char
account-start  = ALPHA / DIGIT
account-char   = account-start / "_" / "-"

type-name      = ALPHA *type-char
type-char      = ALPHA / DIGIT / "_" / "-"

segment        = 1*( ALPHA / DIGIT / "." / "_" / "~" / "-" )
```

Restricciones adicionales:

- `hrn:` es literal y en minúsculas. `HRN:` se rechaza.
- El HRN completo es ASCII y como máximo de 1481 bytes (suma exacta de los límites por campo, el prefijo y los separadores).
- `service` admite mayúsculas en la entrada y se canonicaliza a minúsculas; `partition` y `region` solo admiten minúsculas.
- `service` puede empezar por dígito como identidad de recurso; para `ActionId` se exige que empiece por letra minúscula (§7).
- `partition`, `service` y `region` no pueden empezar ni terminar con `-`.
- `account` empieza por letra o dígito y es sensible a mayúsculas.
- `resource-type` empieza por letra y es sensible a mayúsculas.
- `resource-path` tiene al menos un segmento; no admite segmentos vacíos, `.` ni `..`.
- Ningún campo admite `:`. El path usa `/` como separador y puede anidarse.
- No se admite percent-encoding, backslash, caracteres de control ni normalización de mayúsculas en el path.

Límites por campo (en bytes):

| Campo | Máximo |
| --- | --- |
| `partition` | 64 |
| `service` | 64 |
| `region` | 64 |
| `account` | 128 |
| `resource-type` | 128 |
| `resource-path` | 1024 |
| **HRN completo** | **1481** |

## 3. Componentes

| Campo | Obligatorio | Canonicalización | Igualdad |
| --- | --- | --- | --- |
| `partition` | sí | ninguna | exacta |
| `service` | sí | **minúsculas** | tras canonicalizar |
| `region` | no (vacío) | ninguna | exacta; vacío ≠ `global` |
| `account` | sí | ninguna | exacta, sensible a mayúsculas |
| `resource-type` | sí | ninguna | exacta |
| `resource-path` | sí | ninguna | exacta, segmento a segmento |

`region` vacía representa el scope global y, además, es la forma byte a byte de los HRN v1 de **Artifacts**. **No se reescribe** a `global`: son valores distintos y el contrato no los equipara. `hodei-authz` 0.1 usaba `global` por defecto; esa forma se trata como ambigua en la matriz de migración, no como alias automático.

`global` es un valor de región válido como dato, pero su equivalencia con “sin región” es una decisión de cada servicio y **no** una regla del contrato. Audit Trail, que serializa `global` para `None`, requiere migración explícita (§8).

## 4. Canonicalización

1. Validar el prefijo, el número de componentes y cada campo.
2. Convertir `service` a minúsculas ASCII.
3. No modificar ningún otro campo.
4. Serializar exactamente como:

```text
hrn:{partition}:{service}:{region}:{account}:{resource-type}/{resource-path}
```

Con `region` vacía, la representación conserva los dos puntos consecutivos: `hrn:aws:iam::123456789012:User/alice`.

La serialización es idempotente y estable: `parse(display(x)) == x`, y un HRN ya canónico se serializa byte a byte igual.

> **Nota:** la canonicalización de `service` es la única reescritura permitida. Un HRN con `Artifacts` y otro con `artifacts` son el mismo recurso; un HRN con `Acme` y otro con `acme` en `account` **no** lo son.

## 5. Igualdad

Dos HRN son iguales si y solo si coinciden `partition`, `service` (ya canonicalizado), `region`, `account`, `resource-type` y `resource-path`, comparando cada campo por bytes.

Consecuencias obligatorias:

- `account-a` y `account-b` con el mismo `resource-type/resource-path` **no** son iguales. Es el aislamiento entre cuentas.
- `eu-west-1` y `us-east-1` con el mismo path **no** son iguales.
- `repository/releases` y `repository/Releases` **no** son iguales.
- La región vacía y `global` **no** son iguales por contrato.

## 6. Jerarquía y descendencia

Un recurso puede anidar su path. La relación padre-hijo se define **por segmentos completos**:

- `hrn:...:repository/releases` es ancestro de `hrn:...:repository/releases/v2`.
- `hrn:...:repository/releases` **no** es ancestro de `hrn:...:repository/releases-old`.
- `parent()` elimina el último segmento; un path de un solo segmento no tiene padre.
- La descendencia exige el mismo `partition`, `service`, `region`, `account` y `resource-type`.

Nunca se implementa con `starts_with` sobre el path completo. El byte siguiente al prefijo DEBE ser `/`.

## 7. Acciones

Las acciones son verbos, no recursos:

- La reserva de `Action` es **case-insensitive**: `Action`, `action` y `ACTION` **NO DEBEN** aceptarse como HRN de recurso; el parser devuelve `HrnError::ActionIsNotResource` antes de validar la cuenta.
- La identidad canónica de una acción es `ActionId`:

```text
<ServiceNamespace>::Action::"<name>"
```

- `ServiceNamespace` se deriva del servicio canónico (minúsculas) de forma **inyectiva**: se sustituye cada `-` por `_` y se pone la primera letra en mayúscula. El servicio DEBE empezar por letra minúscula. Ejemplos: `iam` → `Iam`; `audit-trail` → `Audit_trail`; `a-1` → `A_1` (distinto de `a1` → `A1`).
- `name` es la acción, con `[A-Za-z0-9][A-Za-z0-9_-]*`, máximo 128 bytes.
- Las acciones son **globales por servicio**: la partición no forma parte del UID Cedar. La plataforma DEBE garantizar que un nombre de servicio es único entre particiones.
- El formato legacy `hrn:<partition>:<service>:::Action/<name>` se acepta **solo** con región y cuenta vacías, en el adaptador de migración (`ActionId::from_legacy_hrn`). Una acción con scope se rechaza: no se migra perdiendo el scope.
- `ActionId` se serializa como su UID Cedar y puede volver a parsearse (`FromStr`/`from_cedar_uid`). El parser de UID solo acepta la **forma canónica**: `from_cedar_uid("IAM::Action::\"read\"")` se rechaza; `Iam::Action::"read"` se acepta.
- Un `ActionId` y un `Hrn` son tipos distintos y no se comparan entre sí.

## 8. Alias, ambigüedad y migración

El contrato distingue tres resultados al leer datos históricos:

1. **Canónico:** se parsea sin cambios.
2. **Alias / canonicalización:** se acepta y se obtiene una forma canónica, pero el valor original se conserva como dato hasta una migración explícita.
3. **Ambigüedad:** la cadena es sintácticamente válida pero su significado depende del sistema de origen (por ejemplo, Audit Trail coloca `tenant` donde el contrato espera `region`). **NO DEBE** convertirse por adivinación; requiere origen, contexto o una decisión humana.

La matriz completa está en [`migration-matrix.md`](migration-matrix.md). Regla de oro: **ante la duda, rechazar o marcar ambiguo; nunca reinterpretar en silencio**.

`Hrn::parse` es **solo sintáctico**: no conoce el origen y, por tanto, acepta como literales cadenas que pueden ser ambiguas (por ejemplo, Audit Trail con tenant y región intercambiados). Quien lee datos persistidos DEBE usar `hodei_hrn::classify(origin, input)`, que devuelve `HrnError::AmbiguousOrigin` para Authz v1, Audit Trail y origen desconocido. El tipo `Hrn` **no conserva el string original**: la persistencia debe guardar el raw y su origen por separado hasta completar la migración explícita.

## 9. Errores

El parser distingue, como mínimo:

- `NonAscii`, `TooLong`, `InvalidPrefix`, `InvalidComponentCount`;
- `EmptyField` para `partition`, `service`, `account`, `resource_type` y `resource_path`;
- `InvalidField` para cada campo con valor inválido, incluido `resource` cuando falta el separador `/`; el valor se escapa para no inyectar controles en logs;
- `ActionIsNotResource`;
- `AmbiguousOrigin` cuando `classify` no puede resolver el layout sin procedencia.

Un error de parseo no produce un HRN parcial ni un valor por defecto.

## 10. Invariantes de seguridad

1. El HRN no confiere permisos.
2. `account` es parte del scope de identidad; nunca se infiere de una cabecera de cliente.
3. Dos cuentas pueden tener homónimos; el HRN canónico los separa.
4. La descendencia no se calcula por prefijo de cadena.
5. Acción y recurso no comparten representación.
6. La canonicalización no reescribe datos persistidos; los alias se migran explícitamente.
7. El contrato no hace IO, no consulta red y no depende de Cedar.

## 11. Conformidad

Un parser es conforme si supera todos los vectores de [`vectors.json`](vectors.json) y las relaciones de igualdad, descendencia y paternidad allí descritas. Los vectores cubren:

- HRN v2 canónicos, región vacía y paths anidados;
- canonicalización de `service`;
- rechazos de prefijo, componentes, campos vacíos, path inválido, percent-encoding, backslash, controles, no-ASCII y `Action` (incluidas minúsculas);
- límites de longitud y frontera `TooLong`;
- acciones legacy;
- homónimos entre cuentas y regiones;
- trampa de prefijo en descendencia.
