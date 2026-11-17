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

## Krok 2: uživatelé (~2 h)

- [ ] `seed`: `POST /api/auth/register`, když účet neexistuje, jinak jen přihlášení.
- [ ] SQL výstup (`--sql-out`): `demo_admin` → `ADMIN`, `demo_suspended` → `SUSPENDED` (podle e-mailu, ne id).

**Hotovo, když:** dva běhy `seed` za sebou dají pořád 12 uživatelů a jde se za ně přihlásit na webu.

## Krok 3: soubory a assety (~5 h)

- [ ] Generátory bez externích crate:

  | Formát | Obsah |
  |---|---|
  | PNG | `deflate` stored bloky, CRC a Adler ručně |
  | WAV | tón |
  | GLB | kostka |
  | ZIP | stored |
  | TMX | malá mapa |
- [ ] Fixtura ~60 assetů a okrajové případy.
- [ ] `POST /api/assets` multipart jako vlastník. Existující (jméno + vlastník) se přeskočí.
- [ ] SQL výstup: `created_at`, `download_count`.
- [ ] Testy:
  - typy a přípony odpovídají catalogu;
  - název do 100 znaků, popis do 2000;
  - celkem do 20 MB;
  - GLB a PNG mají platnou hlavičku.

**Hotovo, když:** lokálně je v katalogu ~60 assetů, stažení vrací skutečný soubor a nejaktivnější autor má
v „My assets“ 2 stránky.

## Krok 4: reporty (~2 h)

- [ ] `POST /api/assets/:assetId/reports` jako běžný uživatel. Část zamítne `demo_admin`
      (`POST /api/reports/:id/reject`).
- [ ] Idempotence: stejný autor + asset se nehlásí znovu.

**Hotovo, když:** v `/admin/reports` je fronta otevřených reportů a pár zamítnutých.

## Krok 5: `wipe` (~2 h)

- [ ] Smazat demo assety přes `DELETE /api/assets/:id` jako vlastník (catalog smaže i S3).
- [ ] SQL výstup: reporty na demo assety a od demo uživatelů, pak demo účty (podle domény e-mailu).
- [ ] Před mazáním vypsat počty.

**Hotovo, když:** `seed` → `wipe` → `status` dá nuly a v S3 nezůstane nic po demo assetech.

## Krok 6: workflow na server (~3 h)

- [ ] AWS (skript pro CloudShell): OIDC role pro tohle repo s `ssm:SendCommand` na instanci
      `i-0af9c67b1566f5c8d`.
- [ ] Secrets v repu: `DEMO_PASSWORD`, `AWS_ROLE_ARN`.
- [ ] `.github/workflows/demo-data.yml`:
  1. `workflow_dispatch` s volbou `status` / `seed` / `wipe`;
  2. `cargo build --release`;
  3. spustit nástroj proti produkci s `--yes`;
  4. SQL soubor přes SSM do `docker compose exec -T db psql`.
- [ ] README: jak spustit lokálně a z GitHubu.

**Hotovo, když:** workflow `seed` naplní produkci a `wipe` ji vrátí do původního stavu.

## Později

Až to bude umět backend `assetStore`:

- [ ] tagy;
- [ ] preview obrázky;
- [ ] recenze a hodnocení.
