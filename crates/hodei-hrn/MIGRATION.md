# Migración de hodei-hrn 0.1 → 0.2

`hodei-hrn` 0.2.0 implementa el contrato HRN v2. La versión 0.1.0 publicada no se renombra ni se sobrescribe: 0.2.0 es una versión nueva y la titularidad de `hodei-hrn` en crates.io se verifica en HSP-3 antes de publicar.

## Qué se conserva

| 0.1 | 0.2 |
| --- | --- |
| `hodei_hrn::api::Hrn` | El módulo `api` sigue existiendo y re-exporta `Hrn`, `HrnBuilder` y los errores |
| `HrnBuilder` con `service`, `tenant_id`, `resource`, `build` | Igual, más `partition`, `region`, `account`, `resource_type`, `resource_path` |
| Defaults del builder `hodei`/`global` | Igual en `HrnBuilder::new()` |
| `HrnBuilder::default()` con todos los campos vacíos | Igual: `build()` falla hasta que se rellenan |
| `Display` y `FromStr` | Igual, con parser estricto |
| `tenant_id`/`resource_id` como conceptos | Getters `tenant_id()`/`resource_id()` como aliases de `account()`/`resource_path()` |

> **`tenant_id` no es `account` en general.** El alias existe para el layout de Authz v1, donde el cuarto campo era el tenant. En un HRN canónico, Artifacts o AWS el cuarto campo es la cuenta; no interpretes un tenant como cuenta sin que el servicio lo confirme (matriz de migración, filas 6–8).

## Rupturas intencionadas

| 0.1 | 0.2 | Motivo |
| --- | --- | --- |
| Campos públicos `partition`, `service`, `region`, `tenant_id`, `resource_type`, `resource_id` | Getters `partition()`, `service()`, `region()`, `account()`, `resource_type()`, `resource_path()` | Los campos privados impiden construir HRN inválidos |
| Serde como objeto de campos | Serde como **string canónico** | La forma serializada es la identidad; `Deserialize` valida |
| Feature `sqlx` | Eliminada | El contrato no depende de PostgreSQL; la persistencia pertenece a los servicios |
| No había constructor validado | `Hrn::new(...) -> Result<Hrn, HrnError>` | Construcción programática segura |
| No había `from_string` en 0.1 | `Hrn::from_string(&str) -> Option<Hrn>` | Helper para el kernel de Artifacts; **no** es API 0.1 y oculta el error. No lo uses para datos persistidos sin `classify` |

`from_string` es solo sintáctico: para Artifacts v1, Authz v1, Audit Trail u origen desconocido usa `classify(origin, input)`. La procedencia por sí sola **no** resuelve Authz v1 ni Audit Trail: hace falta un adapter explícito que confirme la semántica de los campos.

## Errores

| 0.1 | 0.2 |
| --- | --- |
| `InvalidFormat` | `InvalidComponentCount`, `InvalidPrefix` o `InvalidField`, según el caso |
| `InvalidPrefix` | `InvalidPrefix` |
| `InvalidResourcePart` | `InvalidResourcePart` en el builder; `InvalidField { field: "resource" }` en el parser |
| `MissingPart` | `MissingPart` en el builder |
| — | `EmptyField`, `InvalidField`, `ActionIsNotResource`, `AmbiguousOrigin`, `TooLong`, `NonAscii` |

`InvalidFormat` se conserva como variante de compatibilidad para brazos de `match` antiguos, pero el parser ya no la produce. `HrnError` es `#[non_exhaustive]`: añadir variantes no rompe a los consumidores que tengan un brazo comodín.

## Serde antiguo (objeto de campos)

0.2 serializa `Hrn` como string. Para leer el JSON 0.1, usa `LegacyHrnFields` y convierte explícitamente:

```rust
use hodei_hrn::{Hrn, LegacyHrnFields};

let fields: LegacyHrnFields = serde_json::from_str(old_json)?;
let hrn = Hrn::try_from(fields)?; // valida y canonicaliza
```

`LegacyHrnFields::from(&hrn)` produce la forma antigua para exportadores.

## SQLx

La feature `sqlx` desaparece. Guarda `hrn.to_string()` en una columna `TEXT` y reconstruye con `Hrn::parse(&text)` (o `Hrn::from_string` si aceptas perder el diagnóstico). No hay `Type`/`Encode`/`Decode` implícitos en 0.2.

## Consumidores actuales

- `hodei-authz` sigue usando `hodei-hrn 0.1.0` por path y `hodei_hrn::api::Hrn` con campos públicos. No puede cambiar a 0.2 hasta que 0.2 esté publicado (HSP-3); el cambio de consumidor y la retirada del duplicado son HSP-18.
- `hodei-artifacts` mantiene su `Hrn` en `kernel`; su adaptación es HSP-12/HSP-18.
- El módulo `api` y `LegacyHrnFields` existen para que ese cambio sea mecánico, no para declarar que ya está hecho.

## Licencia y metadatos

Se restauran la licencia dual `MIT OR Apache-2.0`, `authors`, `documentation`, `homepage`, `keywords` y `categories` de 0.1. La feature `sqlx` sigue eliminada por diseño.

## Publicación

- El nombre `hodei-hrn` no se renombra.
- 0.2.0 debe publicarse con el mismo nombre y verificar la titularidad con `cargo owner --list hodei-hrn` antes de HSP-3.
- La prueba de consumo desde un proyecto externo sin dependencias `path` es HSP-3.
