# hodei-security-contracts

Contratos de seguridad Hodei: HRN canónico, gramática, vectores de compatibilidad y vocabulario compartido para el MVP de seguridad.

## Contenido

| Ruta | Qué es |
| --- | --- |
| [`contracts/hrn/specification.md`](contracts/hrn/specification.md) | Contrato normativo HRN v2: gramática, canonicalización, igualdad, descendencia y acciones |
| [`contracts/hrn/migration-matrix.md`](contracts/hrn/migration-matrix.md) | Qué aceptar, rechazar o marcar ambiguo en Artifacts, Authz y Audit Trail |
| [`contracts/hrn/vectors.json`](contracts/hrn/vectors.json) | Vectores ejecutables de conformidad, igualdad, descendencia y acciones |
| [`language/security-glossary.md`](language/security-glossary.md) | Vocabulario compartido entre repositorios |
| [`crates/hodei-hrn`](crates/hodei-hrn) | Implementación de referencia del contrato (sin IO, sin Cedar, sin Tokio) |
| [`crates/hodei-hrn/MIGRATION.md`](crates/hodei-hrn/MIGRATION.md) | Ruptura y compatibilidad 0.1 → 0.2 |

## Verificación

```bash
cargo test
```

Los tests recorren `vectors.json`: parseo, canonicalización, rechazos, límites, acciones legacy, homónimos entre cuentas/regiones y descendencia por segmentos completos. La copia pública del contrato se sincroniza con `./scripts/sync-vectors.sh` y CI verifica que no diverja.

## Estado

- `hodei-hrn` 0.2.0 es la referencia del contrato. La consolidación de la API publicada y la publicación en crates.io corresponden a HSP-2/HSP-3.
- Los datos persistidos no se reescriben: los alias y la ambigüedad se resuelven con adapters y decisiones explícitas.
- Este repositorio no contiene servicios, persistencia ni evaluación de políticas.
