# Glosario de seguridad Hodei

Términos compartidos por Contracts, Authz y Artifacts. Las definiciones son normativas para el MVP; el contrato de HRN manda sobre la sintaxis.

| Término | Definición |
| --- | --- |
| **HRN** | *Hodei Resource Name*. Identidad estable y comparable de un recurso. No es una autorización. |
| **Canónico** | Representación única que produce el parser: `service` en minúsculas y el resto de campos sin reescribir. |
| **Alias** | Forma histórica aceptada que se lee como el mismo identificador, pero que no se reescribe en los datos persistidos. Ejemplo: HRN v1 con región vacía. |
| **Ambigüedad** | Cadena sintácticamente válida cuyo significado depende del sistema de origen. No se convierte sin metadato de origen y decisión explícita. |
| **Partition** | Espacio de nombres superior (`hodei`, `aws`). No se infiere. |
| **Service** | Servicio lógico que posee el recurso (`artifacts`, `iam`). Se canonicaliza a minúsculas. |
| **Region** | Ámbito geográfico o lógico. Vacía representa el scope global/v1 y se conserva. `global` es un valor literal distinto. |
| **Account** | Frontera de aislamiento del recurso. Es sensible a mayúsculas. Un HRN de otra cuenta no es igual aunque el path coincida. |
| **Tenant** | Término histórico de `hodei-authz`; solo equivale a `account` cuando el servicio lo confirma. |
| **Resource type** | Tipo del recurso (`repository`, `User`, `physical-artifact`). Empieza por letra. |
| **Resource path** | Uno o más segmentos separados por `/`. Puede anidarse. No admite `.`, `..`, segmentos vacíos ni `:`. |
| **Descendencia** | Relación padre-hijo por segmentos completos del path, con el mismo scope. No es un prefijo de cadena. |
| **Acción** | Verbo autorizable. Tiene identidad propia (`ActionId`), nunca se representa como HRN de recurso. |
| **ActionId** | Identidad Cedar de una acción: `<Service>::Action::"<name>"`. |
| **Scope** | `partition` + `service` + `region` + `account`. Dos recursos con el mismo path y distinto scope no son iguales. |
| **Deny by default** | Sin una decisión explícita de autorización, el acceso se deniega. Un HRN válido no concede permiso. |
