# Demo data: specifikace

Plán práce je v [plan.md](plan.md).

## Proč

Katalog na webu je skoro prázdný. Demo (prezentace, testování, screenshoty) tak vypadá chudě a každý si data
vyrábí ručně. Chceme jedním příkazem dostat **realistický, vždy stejný obsah**:

- uživatele;
- assety se skutečnými soubory;
- reporty;
- tagy;
- později recenze.

A stejně jednoduše ho zase odstranit.

## Co to je

- **Samostatný repozitář `assetStore-demo`** ve stejné organizaci a **CLI v Rustu**: jeden malý binární soubor
  (~2 MB), bez Node a bez Dockeru.
- **Na serveru nic neběží a nic se tam nestaví.** Nástroj se spouští z GitHub Actions tohoto repa (nebo lokálně)
  a s webem mluví přes jeho veřejné API.
- **Hlavní repo `assetStore` se kvůli demu nemění.** Žádné „seed“ endpointy, žádné změny v deployi.
- **AWS zůstává stejné.** Soubory demo assetů jdou do stejného S3 jako od běžných uživatelů, protože je tam ukládá
  catalog.

## Hlavní pravidla

1. **Přes veřejné API, kde to jde.** Registrace, přihlášení, nahrání assetu, report a jeho zamítnutí jdou přes
   stejné endpointy jako z webu. Data tak projdou validací a audit logem. Když se API v `assetStore` změní,
   nástroj to hned ukáže (`status` a `seed` selžou se srozumitelnou chybou).
2. **SQL jen na to, co API neumí:**
   - role `ADMIN`;
   - stav `SUSPENDED`;
   - zpětné `created_at`;
   - `download_count`.

   Nástroj tyto dotazy jen **vypíše jako SQL soubor** (`seed --sql-out demo.sql`). Workflow ho pak pošle přes SSM
   do kontejneru `db` (`docker compose exec -T db psql`). Nástroj sám k databázi nikdy nepřistupuje.
3. **Deterministický obsah.** Fixtury psané ručně v `src/fixtures/`, ne náhodný text. Dva běhy dají stejný výsledek.
4. **Idempotence.** Opakovaný `seed` nic nezdvojí:
   - existujícího uživatele jen přihlásí;
   - asset se stejným jménem u stejného vlastníka přeskočí.
5. **Demo data jdou poznat a smazat.**
   - Všichni demo uživatelé mají e-mail `@demo.assetstore.test`.
   - Demo asset = asset demo uživatele.
   - `wipe` maže jen tohle.
6. **Žádná tajemství v repu.** Heslo demo účtů je v `DEMO_PASSWORD`: lokálně v `.env`, v GitHubu jako secret.
7. **Ochrana produkce.** Proti čemukoli jinému než `localhost` / `127.0.0.1` vyžadují `seed` a `wipe` přepínač
   `--yes`.

## Příkazy

| Příkaz | Co udělá |
|---|---|
| `status` | Zkontroluje health služeb a vypíše demo uživatele a počet jejich assetů. |
| `check` | Jako `status` + ukázkové assety musí odpovědět (detail a náhled přes Range, takže se nic nestahuje). Jinak skončí chybou. Běží každé ráno (cron) a po každém `seed`. |
| `seed` | Vytvoří nebo doplní demo data. |
| `wipe` | Smaže demo assety přes API (catalog smaže i soubory v S3). Vypíše SQL na smazání demo účtů a jejich reportů. |

Přepínače:

- `--dry-run` jen vypíše, co by se stalo;
- `--yes` je povinný mimo localhost.

## Konfigurace

| Proměnná | Co |
|---|---|
| `API_URL` | API webu, např. `https://3-125-15-78.sslip.io/api`, lokálně `http://localhost:5173/api` (vite proxy). |
| `DEMO_PASSWORD` | Heslo všech demo účtů, aspoň 8 znaků. |
| `SITE_AUTH_USER`, `SITE_AUTH_PASSWORD` | Jen když web běží se `SITE_AUTH=on` (heslo Caddy na celý web). |

## Obsah demo dat (verze 1)

### Uživatelé (12)

| Kdo | Počet | Poznámka |
|---|---|---|
| Autoři assetů | 8 | Různě aktivní: jeden má 20+ assetů (stránkování v „My assets“), jeden jen 1. |
| Běžní uživatelé | 2 | Posílají reporty. |
| `demo_admin` | 1 | Role `ADMIN` přes SQL, moderuje reporty. |
| `demo_suspended` | 1 | Stav `SUSPENDED` přes SQL. |

### Assety (~60)

- Všech 5 typů, každý aspoň 8×. Ruční názvy a popisy.
- Okrajové případy:
  - dlouhý název;
  - diakritika;
  - prázdný popis;
  - popis s 2000 znaky.
- **Skutečné malé soubory, generované v kódu** (žádné stahování z internetu):

  | Formát | Obsah |
  |---|---|
  | PNG | vzor |
  | WAV | tón |
  | GLB | kostka, jde otočit v budoucím 3D náhledu |
  | ZIP | pár souborů |
  | TMX | malá mapa |

  Každý soubor do ~200 kB, celkem do **20 MB**. Přípony jen z whitelistu catalogu.
- `created_at` rozložené do posledních ~60 dní a různé `download_count` (SQL).

### Reporty (~10)

- Od běžných uživatelů, s důvody podle AM (`inappropriateContent`, `copyright`, `spam`, `brokenFile`, `other`).
- Část zůstane `open`, část je zamítnutá jménem `demo_admin` (SQL: `status = 'rejected'`, `decided_by`, `decided_at`).

### Později

Až to bude umět backend; každá oblast = nový modul + fixtura:

- recenze a hodnocení.

## Spouštění na serveru

Workflow **Demo data** v tomto repu, ručně (`workflow_dispatch`), volba `status` / `seed` / `wipe`:

1. Postaví nástroj na runneru GitHubu (cache přes `rust-cache`, nic na našem serveru).
2. Spustí ho proti `https://3-125-15-78.sslip.io/api` s `DEMO_PASSWORD` ze secrets.
3. Když nástroj vypsal SQL, pošle ho přes SSM do kontejneru `db` na serveru.

   K tomu je potřeba OIDC role v AWS, která věří tomuhle repu. Buď se rozšíří trust policy
   `assetstore-github-deploy`, nebo vznikne nová role jen s `ssm:SendCommand` na tuhle instanci.
   Udělá se skriptem v CloudShellu.

## Rizika

| Riziko | Opatření |
|---|---|
| `wipe` smaže skutečná data | Maže jen podle domény `@demo.assetstore.test`. Před mazáním vypíše počty. Mimo localhost chce `--yes`. |
| Někdo se přihlásí jako `demo_admin` | Silné `DEMO_PASSWORD` jen v secrets. |
| Změna API v `assetStore` rozbije nástroj | `status` po každém větším merge. Chyby vypisují endpoint a odpověď. |
| S3 / disk | Do 20 MB v S3. Na disk serveru nic. |
