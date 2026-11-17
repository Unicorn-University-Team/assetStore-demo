# assetStore-demo

Demo data pro [assetStore](https://github.com/Unicorn-University-Team/assetStore): jedním příkazem naplnit web
realistickými uživateli, assety a reporty a stejně jednoduše je zase smazat.

Specifikace a plán: [docs/spec.md](docs/spec.md), [docs/plan.md](docs/plan.md).

## Rychlý start (lokálně)

Potřeba: Rust (`rustup`), běžící assetStore lokálně (služby + `npm run dev` ve `frontend`, vite proxy `/api`).

```bash
cp .env.example .env    # doplnit DEMO_PASSWORD
set -a; source .env; set +a
cargo run -- status     # health služeb a demo uživatelé
cargo run -- seed       # (zatím jen kostra, viz docs/plan.md)
```

Mimo `localhost` vyžadují `seed` a `wipe` přepínač `--yes`.

## Vývoj

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
```

Každá změna = vlastní větev a PR (česky), commity anglicky, merguje Yura.
