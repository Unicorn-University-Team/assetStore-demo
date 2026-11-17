# assetStore-demo

Demo data pro [assetStore](https://github.com/Unicorn-University-Team/assetStore): jedním příkazem naplnit web
realistickými uživateli, assety a reporty a stejně jednoduše je zase smazat.

Specifikace a plán: [docs/spec.md](docs/spec.md), [docs/plan.md](docs/plan.md).

## Demo režim (tlačítko)

**[Actions → Demo data → Run workflow](https://github.com/Unicorn-University-Team/assetStore-demo/actions/workflows/demo-data.yml)**
a vybrat:

| Volba | Co udělá |
|---|---|
| `status` | Jen se podívá: běží služby, kolik je demo účtů a assetů. |
| `seed` | Vytvoří nebo doplní demo data: 12 účtů, 59 assetů se skutečnými soubory, 10 reportů. Opakované spuštění nic nezdvojí. |
| `wipe` | Smaže demo data (assety i soubory v S3, reporty, účty). Ničeho jiného se nedotkne. |

Výsledek je v **Summary** běhu:
- počty;
- odkaz na web;
- seznam demo účtů.

Heslo všech demo účtů je v secretu `DEMO_PASSWORD` (zná ho Yura). Tlačítko vidí jen členové týmu s právem zápisu.

| Účet | Na co |
|---|---|
| `pixel.forge@demo.assetstore.test` | Autor s 22 assety (2 stránky v „My assets“). |
| `demo.admin@demo.assetstore.test` | Admin: fronta reportů v `/admin/reports`. |
| `curious.eva@demo.assetstore.test` | Běžný uživatel bez assetů. |
| `demo.suspended@demo.assetstore.test` | Zablokovaný účet: přihlášení odmítne. |

Jednorázové nastavení (už jen jednou):
1. AWS CloudShell: `curl -fsSL https://raw.githubusercontent.com/Unicorn-University-Team/assetStore-demo/main/aws/setup-demo-role.sh | bash`.
   Vypsané ARN se uloží do proměnné repa `AWS_DEMO_ROLE_ARN`.
2. Secret `DEMO_PASSWORD`: `gh secret set DEMO_PASSWORD -R Unicorn-University-Team/assetStore-demo`.

## Rychlý start (lokálně)

Potřeba: Rust (`rustup`), běžící assetStore lokálně (služby + `npm run dev` ve `frontend`, vite proxy `/api`).

```bash
cp .env.example .env    # doplnit DEMO_PASSWORD
set -a; source .env; set +a
cargo run -- status     # health služeb a demo uživatelé
cargo run -- seed --sql-out demo.sql   # SQL pak: psql -v ON_ERROR_STOP=1 -f demo.sql
```

Mimo `localhost` vyžadují `seed` a `wipe` přepínač `--yes`.

## Vývoj

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
```

Každá změna = vlastní větev a PR (česky), commity anglicky, merguje Yura.
