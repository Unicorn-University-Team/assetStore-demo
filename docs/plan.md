# Demo data: plán práce

Podle [spec.md](spec.md). Každý krok = vlastní větev a PR (česky), commity anglicky (conventional commits),
merguje Yura.

## Krok 1: kostra ✅

- [x] Rust CLI: `status`, `seed`, `wipe`, `--dry-run`, `--yes` (ochrana mimo localhost).
- [x] Konfigurace z prostředí (`API_URL`, `DEMO_PASSWORD`, volitelně `SITE_AUTH_*`).
- [x] Session přes API s cookies (přihlášení, `auth/me`), health služeb.
- [x] `status`: demo uživatelé a počet jejich assetů.
- [x] Fixtura 12 uživatelů + testy.
- [x] CI: `cargo fmt`, `clippy -D warnings`, `cargo test`.

## Krok 2: uživatelé ✅

- [x] `seed`: `POST /api/auth/register`, když účet neexistuje, jinak jen přihlášení.
- [x] SQL výstup (`--sql-out`): `demo_admin` → `ADMIN`, `demo_suspended` → `SUSPENDED` (podle e-mailu, ne id).

**Hotovo, když:** dva běhy `seed` za sebou dají pořád 12 uživatelů a jde se za ně přihlásit na webu.

## Krok 3: soubory a assety ✅

- [x] Generátory bez externích crate:

  | Formát | Obsah |
  |---|---|
  | PNG | `deflate` stored bloky, CRC a Adler ručně |
  | WAV | tón |
  | GLB | kostka |
  | ZIP | stored |
  | TMX | malá mapa |
- [x] Fixtura ~60 assetů a okrajové případy.
- [x] `POST /api/assets` multipart jako vlastník. Existující (jméno + vlastník) se přeskočí.
- [x] SQL výstup: `created_at`, `download_count`.
- [x] Testy:
  - typy a přípony odpovídají catalogu;
  - název do 100 znaků, popis do 2000;
  - celkem do 20 MB;
  - GLB a PNG mají platnou hlavičku.

**Hotovo, když:** lokálně je v katalogu ~60 assetů, stažení vrací skutečný soubor a nejaktivnější autor má
v „My assets“ 2 stránky.

## Krok 4: reporty ✅

- [x] `POST /api/assets/:assetId/reports` jako běžný uživatel. Část je zamítnutá jménem `demo_admin`
      (SQL: `status = 'rejected'`, `decided_by`, `decided_at`).
- [x] Idempotence: stejný autor + asset se nehlásí znovu.

**Hotovo, když:** v `/admin/reports` je fronta otevřených reportů a pár zamítnutých.

## Krok 5: `wipe` ✅

- [x] Smazat demo assety přes `DELETE /api/assets/:id` jako vlastník (catalog smaže i S3).
- [x] SQL výstup: reporty na demo assety a od demo uživatelů, pak demo účty (podle domény e-mailu).
- [x] Před mazáním vypsat počty.

**Hotovo, když:** `seed` → `wipe` → `status` dá nuly a v S3 nezůstane nic po demo assetech.

## Krok 6: workflow na server ✅

- [x] AWS skript pro CloudShell (`aws/setup-demo-role.sh`), spuštěný: OIDC role pro tohle repo s `ssm:SendCommand` na instanci
      `i-0af9c67b1566f5c8d`.
- [x] Secret `DEMO_PASSWORD` a proměnná `AWS_DEMO_ROLE_ARN` v repu.
- [x] `.github/workflows/demo-data.yml`:
  1. `workflow_dispatch` s volbou `status` / `seed` / `wipe`;
  2. `cargo build --release`;
  3. spustit nástroj proti produkci s `--yes`;
  4. SQL soubor přes SSM do `docker compose exec -T db psql`.
- [x] README: jak spustit lokálně a z GitHubu.

**Hotovo, když:** workflow `seed` naplní produkci a `wipe` ji vrátí do původního stavu.

## Krok 7: ovládací panel a údržba ✅

- [x] `check`: ukázkové assety odpovídají (detail + náhled), jinak běh selže.
- [x] Workflow:
  - `reset` (wipe + seed);
  - potvrzení `SMAZAT` pro mazání;
  - `dry_run`;
  - denní kontrola (cron 6:00 UTC);
  - Summary s odkazy na 3D, zvuk, obrázek a mapu.
- [x] CI summary (testy, velikost binárky).
- [x] Release workflow: tag `v*` → binárky pro Linux a macOS.
- [x] Dependabot.
- [x] README se screenshoty, diagramem a FAQ.

## Později

Až to bude umět backend `assetStore`:

- [ ] tagy;
- [ ] preview obrázky;
- [ ] recenze a hodnocení.
