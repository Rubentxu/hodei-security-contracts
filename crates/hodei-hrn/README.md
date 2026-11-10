# hodei-hrn

Identidad canónica de recursos Hodei (HRN v2): parsing estricto, canonicalización, comparación por scope y descendencia por segmentos completos.

## Uso canónico

```rust
use hodei_hrn::Hrn;

let hrn = Hrn::parse("hrn:hodei:artifacts:eu-west-1:acme:repository/releases")?;
assert_eq!(hrn.account(), "acme");
assert_eq!(hrn.resource_path(), "releases");
```

La región puede ser vacía (forma v1/global). **No** es equivalente a `global`: `global` es una región literal.

```rust
let v1 = Hrn::parse("hrn:aws:iam::123456789012:User/alice")?;
assert_eq!(v1.region(), "");
```

## Compatibilidad 0.1

El builder conserva los nombres y defaults de 0.1 (`hodei`/`global`), pero valida cada campo:

```rust
use hodei_hrn::Hrn;

let hrn = Hrn::builder()
    .service("documents-api")
    .tenant_id("tenant-a")
    .resource("document/doc-1")?
    .build()?;
assert_eq!(hrn.to_string(), "hrn:hodei:documents-api:global:tenant-a:document/doc-1");
```

- `HrnBuilder::default()` no pone defaults (como 0.1); `HrnBuilder::new()` usa `hodei`/`global`.
- `tenant_id()` es alias de `account()` y `resource_id()` de `resource_path()`; el alias es solo para el layout Authz v1, no una equivalencia semántica general.
- `hodei_hrn::api::Hrn` sigue existiendo.
- `LegacyHrnFields::into_canonical(MigrationContext)` convierte el JSON de campos de 0.1 solo con un origen explícito; orígenes ambiguos se rechazan.
- Los campos públicos ya no son mutables: usa `Hrn::new`, el builder o el parser.

## Migración y aliases

- `Hrn::parse` es solo sintáctico. Para datos persistidos usa `hodei_hrn::classify(origin, input)`; Audit Trail, Authz v1 y origen desconocido se rechazan como ambiguos.
- La persistencia debe conservar el raw y el origen.
- Consulta [MIGRATION.md](MIGRATION.md) para la ruptura 0.1 → 0.2, errores, serde y SQLx.

## Verificación

```bash
cargo test
```

Los vectores de conformidad están en `tests/fixtures/vectors.json` y cubren límites, errores, homónimos, descendencia y acciones. La suite de compatibilidad cubre builder, aliases, JSON 0.1 y path anidado.
