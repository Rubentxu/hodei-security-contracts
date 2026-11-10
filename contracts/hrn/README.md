# Contrato HRN

- [`specification.md`](specification.md): gramática, canonicalización, igualdad, descendencia, acciones y errores.
- [`migration-matrix.md`](migration-matrix.md): qué aceptar, rechazar o marcar ambiguo según el origen.
- [`vectors.json`](vectors.json): vectores de conformidad.

`vectors.json` es una copia del fixture que ejecuta el crate:

```text
crates/hodei-hrn/tests/fixtures/vectors.json
```

El fixture es la fuente de verdad para los tests. Tras editarlo, sincroniza la copia pública:

```bash
./scripts/sync-vectors.sh
./scripts/check-vectors.sh
```
