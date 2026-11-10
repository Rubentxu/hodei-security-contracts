# Matriz de migración HRN

**Contrato:** [`specification.md`](specification.md) · **Vectores:** [`vectors.json`](vectors.json)

Esta matriz decide qué hacer con las cadenas HRN que ya existen. No se reescribe ningún dato persistido en bloque y **no se adivina** el origen de una cadena.

## Layouts históricos

| Sistema | Orden de campos | Observaciones |
| --- | --- | --- |
| `hodei-artifacts` (v2) | `partition:service:region:account:type/path` | Referencia del contrato; región vacía = forma v1 (scope global), **no** alias de `global`; path anidado |
| `hodei-authz` (`hodei-hrn` 0.1) | `partition:service:region:tenant:type/id` | Misma posición que Artifacts, pero el cuarto campo es `tenant`; no valida vacíos |
| `hodei-audit-trail` | `partition:service:tenant:region:type/path` | **`tenant` y `region` están intercambiados** respecto al contrato; `global`/vacío se serializa como `global` |

## Matriz

| # | Entrada / origen | Sintaxis canónica | Clasificación | Acción de migración |
| --- | --- | --- | --- | --- |
| 1 | Artifacts v2 con cuenta y región: `hrn:hodei:artifacts:eu-west-1:acme:repository/releases` | válida | **canónica** | Ninguna; byte a byte |
| 2 | Artifacts v1 con región vacía: `hrn:aws:iam::123456789012:User/alice` | válida | **alias v1** | Conservar bytes; el contrato la acepta como el mismo identificador |
| 2b | Artifacts v1 con región no vacía | válida | **ambigua** | La forma v1 es solo región vacía; sin origen no se reinterpreta |
| 3 | Artifacts con `service` en mayúsculas: `...:Artifacts:...` | válida | **canonicalización** | `service` a minúsculas; conservar el original como dato si se audita |
| 4 | Artifacts con `region = global` | válida | **canónica (dato)** | `global` es una región literal; **no** equivale a región vacía sin decisión del servicio |
| 5 | Artifacts con path anidado | válida | **canónica** | Ninguna; descendencia por segmentos |
| 6 | `hodei-authz` 0.1 donde `tenant == account` | válida | **ambigua por defecto** | `classify` la rechaza; un adapter explícito puede devolver `Alias` solo tras confirmar que tenant y account son el mismo ámbito |
| 7 | `hodei-authz` 0.1 donde `tenant != account` | válida | **ambigua** | **No** convertir sin origen y decisión humana; `tenant` no es `account` |
| 8 | `hodei-authz` 0.1 con `region` vacía y `tenant == account` | válida | **ambigua por defecto** | `classify` la rechaza; un adapter explícito puede devolver `Alias` solo tras confirmar el ámbito |
| 8b | `hodei-authz` 0.1 con `region` vacía y `tenant != account` | válida | **ambigua** | No convertir sin origen y decisión humana |
| 9 | Audit Trail: `hrn:hodei:audit:tenant-a:eu-west-1:event/1` | válida | **ambigua** | El contrato leería `region=tenant-a`, `account=eu-west-1`: semántica incorrecta. Requiere adapter por origen |
| 10 | Audit Trail con `global` en la posición de región: `hrn:hodei:audit:tenant-a:global:event/1` | válida | **ambigua** | `global` significa “sin región” en Audit Trail; no se equipara en silencio. El vector correspondiente usa exactamente esta forma |
| 11 | Audit Trail sin `tenant` o con `region` en la posición de tenant | variable | **rechazo o ambigua** | No hay firma que lo distinga de un HRN canónico; exigir metadato de origen |
| 12 | `hrn:...:Action/<name>` con región y cuenta **vacías** | inválida como recurso | **acción** | Convertir con `ActionId::from_legacy_hrn`; nunca crear un `Hrn` |
| 12b | `hrn:...:Action/<name>` con región o cuenta no vacías | inválida como recurso | **rechazo** | No migrar perdiendo scope; requiere decisión de servicio |
| 13 | Prefijo distinto de `hrn:` | inválida | **rechazo** | Error de parseo; no normalizar |
| 14 | Campos vacíos (`partition`, `service`, `account`, `type`, `path`) | inválida | **rechazo** | Error; no rellenar defaults |
| 15 | Path con `:`, `//`, `.`, `..` o no-ASCII | inválida | **rechazo** | Error; no sanear silenciosamente |
| 16 | `global` en un sistema que define región vacía como global | válida | **ambigua** | Requiere decisión de servicio; no equivalencia automática |

## Regla de decisión

1. Si el origen es **Artifacts v2**, se aplica el contrato tal cual.
2. Si el origen es **Artifacts v1**, la región vacía es alias; se conserva la cadena.
3. Si el origen es **hodei-authz**, solo se acepta si `tenant` y `account` son el mismo concepto en ese servicio. Si no, es ambiguo.
4. Si el origen es **Audit Trail**, la cadena **no** se convierte sin un adapter que sepa que los campos 4 y 5 están intercambiados. El parser canónico no puede detectarlo.
5. Si el origen es desconocido y la cadena es sintácticamente válida, se marca **ambigua** por defecto: no se acepta como HRN canónico sin una decisión explícita de procedencia.
6. Si es una acción legacy con región y cuenta vacías, se migra a `ActionId`; si tiene scope, se rechaza. Nunca se almacena como recurso.
7. Cualquier campo inválido se rechaza; no se rellena con `hodei`, `global`, `default` ni `system`.

## Consecuencia para la publicación

`hodei-hrn` 0.2.0 implementa este contrato. El crate 0.1.0 de `hodei-authz` usaba `tenant_id` en la posición de cuenta y no validaba; su sustitución requiere un adapter y pruebas sobre fixtures, no un reemplazo masivo. Los datos persistidos se conservan hasta demostrar equivalencia.
