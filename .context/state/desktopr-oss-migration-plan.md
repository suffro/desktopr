# Desktopr — Piano aggiornato per la conversione open source

## Obiettivo

Trasformare l'attuale codebase Desktopr in un **tool open source standalone**, senza alcun servizio hosted Desktopr.

Il cloud/SaaS è già stato dismesso manualmente e **non fa parte di questo piano**.

Il risultato finale deve essere un monorepo tecnico che permetta a uno sviluppatore di:

- usare il runtime Desktopr/Tauri;
- usare il bridge JS/TS e l'SDK;
- usare i moduli WASM/plugin;
- buildare per Linux, Windows e macOS;
- firmare opzionalmente le build usando i propri GitHub Secrets;
- ottenere artifact e release direttamente da GitHub.

Nessun account Desktopr, backend, CDN, R2, Stripe, Firebase, build credits o servizio di distribuzione.

---

# Repository coinvolte

## Monorepo target

- `Bubbledesk/tauri-skeleton`

Questa repository diventerà progressivamente il nuovo Desktopr monorepo.

**Non rinominarla subito.** Il rename va fatto solo alla fine, quando tutto è stabile.

## Da integrare nel monorepo

- `Bubbledesk/github-actions`
- `Bubbledesk/project-globals`
- `Bubbledesk/wasm-module-template`
- `Bubbledesk/wasm-modules`

## Da integrare per ultima

- `Bubbledesk/companion`

## Da lasciare separata

- `Bubbledesk/docs`

Le docs restano online e in repository separata.

---

# Fase 1 — Inventario e dependency map

## Obiettivo

Capire esattamente cosa dipende da cosa prima di spostare codice.

## Azioni

- [x] Analizzare `tauri-skeleton`.
- [x] Analizzare `github-actions`.
- [x] Analizzare `project-globals`.
- [x] Analizzare `wasm-module-template`.
- [x] Analizzare `wasm-modules`.
- [x] Ignorare `companion` per ora.
- [x] Ignorare `docs`.

Produrre una mappa di:

- dipendenze cross-repo;
- package privati;
- URL/endpoint legacy;
- riferimenti Bubbledesk;
- riferimenti a vecchia infrastruttura;
- codice realmente necessario;
- codice ormai morto.

## Criterio di completamento

È chiaro cosa va copiato, cosa va riscritto, cosa va eliminato e cosa va lasciato dov'è.

---

# Fase 2 — Rimozione di `suffro-lib` e altre dipendenze private

## Obiettivo

Desktopr OSS non deve dipendere da repository o package personali/private.

## Cercare

- `suffro-lib`
- `Suffro/suffro-lib`
- Git dependency private;
- URL GitHub con credenziali;
- `github_pat_`;
- package interni/personali.

## Azioni

Per ogni utilizzo:

- [x] identificare quali funzioni vengono realmente usate;
- [x] copiare solo il minimo codice necessario se Desktopr-specific;
- [x] sostituire utility generiche con dipendenze pubbliche quando opportuno;
- [x] riscrivere localmente utility banali;
- [x] rimuovere del tutto ciò che non serve.

## Regole

- Non rendere `suffro-lib` pubblica.
- Non importare codice personale non necessario.
- Non lasciare dipendenze private.
- Non lasciare URL con token.
- Non stampare secret.

## Criterio di completamento

`npm install` / `cargo` non richiedono alcuna repository privata personale.

---

# Fase 3 — Trasformazione di `tauri-skeleton` nel monorepo Desktopr

## Obiettivo

Usare `tauri-skeleton` come unica source of truth tecnica.

Struttura indicativa:

```text
tauri-skeleton/
├── packages/
│   ├── runtime/
│   └── sdk/
├── wasm/
│   ├── module-template/
│   └── modules/
├── apps/
│   └── companion/        # solo più avanti
├── examples/
├── scripts/
├── .github/
│   └── workflows/
├── README.md
├── LICENSE
├── SECURITY.md
└── .gitignore
```

Non forzare questa struttura se crea lavoro inutile.

La priorità è:

1. consolidare;
2. mantenere il runtime funzionante;
3. ridurre dipendenze esterne;
4. poi pulire.

## Da preservare

- Rust Tauri runtime;
- TypeScript bridge;
- SDK;
- filesystem API;
- file dialogs;
- windows;
- clipboard;
- notifications;
- menu/context menu;
- global shortcuts;
- autostart;
- deep links;
- diagnostics;
- networking;
- WASM runtime;
- plugin storage;
- updater come feature opzionale.

## Criterio di completamento

Il runtime e l'SDK vivono nello stesso monorepo e compilano senza dipendenze dalle vecchie repo.

---

# Fase 4 — Assorbimento di `project-globals`

## Obiettivo

Eliminare la dipendenza dalla repository `project-globals`.

## Estrarre solo ciò che serve

Per esempio:

- Tauri config types;
- schemi;
- costanti runtime;
- tipi condivisi;
- menu schema/config;
- helper tecnici realmente usati.

## Non migrare

- Firebase;
- tipi business;
- Stripe;
- configurazioni SaaS;
- costanti relative al vecchio servizio.

## Criterio di completamento

`project-globals` non è più una dipendenza runtime/build di Desktopr.

**Stato:** completata. È stato assorbito soltanto il contratto menu realmente
usato; tipi SaaS, `suffro-lib` e tipi Tauri obsoleti non sono stati importati.

---

# Fase 5 — Integrazione WASM

## Obiettivo

Portare nel monorepo:

- `wasm-module-template`
- `wasm-modules`

Struttura suggerita:

```text
wasm/
├── module-template/
└── modules/
```

## Azioni

- [x] preservare il template per sviluppare moduli;
- [x] preservare eventuali moduli/example utili;
- [x] verificare compatibilità con il runtime WASM esistente;
- [x] rimuovere riferimenti esterni non necessari.

## Criterio di completamento

Un developer può capire e creare un modulo WASM Desktopr usando solo il monorepo.

**Stato:** completata. Template e modulo math sono nel workspace `wasm/`,
compilano per `wasm32-wasip1` e sono stati eseguiti contro il protocollo del
runtime esistente. La fase 6 non è stata avviata.

---

# Fase 6 — Rimozione completa del vecchio Desktopr come servizio

## Obiettivo

Nel codice non deve restare alcuna dipendenza necessaria da infrastruttura Desktopr.

## Cercare e rimuovere

- `cdn.desktopr.app`;
- endpoint hosted Desktopr;
- URL Bubbledesk;
- R2;
- Firebase;
- Stripe;
- build credits;
- auth hosted;
- callbacks;
- hosted logging;
- status APIs;
- hosted updater;
- private wrapper repo;
- token per checkout della wrapper repo.

## Updater

Deve diventare:

- opzionale;
- configurato dallo sviluppatore;
- disattivato di default se non configurato.

## Regola fondamentale

Una build Desktopr base deve funzionare **offline rispetto a qualsiasi infrastruttura Desktopr**.

**Stato:** completata. Il profilo base usa contenuto locale incorporato, non
concede origini remote e non include l'updater. URL applicazione e updater sono
opt-in e configurati esclusivamente dallo sviluppatore. Actions hosted e
companion restano nei rispettivi step successivi.

---

# Fase 7 — Migrazione delle GitHub Actions

## Obiettivo

Portare `Bubbledesk/github-actions` dentro `.github/workflows/` del monorepo.

## Mantenere

- build;
- signing.

## Eliminare

- distribution hosted;
- R2;
- CDN;
- `dist.yml` come workflow di distribuzione Desktopr;
- upload verso infrastruttura Desktopr.

## Nuovo flusso

```text
source
→ GitHub Actions
→ Linux / Windows / macOS
→ optional signing
→ GitHub Actions artifacts
→ optional GitHub Release
```

## Build workflow

- [x] Linux
- [x] Windows
- [x] macOS
- [x] unsigned build funzionante;
- [x] `workflow_dispatch`;
- [x] `workflow_call` se pratico.

## Signing workflow

Deve usare esclusivamente secret dello sviluppatore/repository consumer.

Esempi:

- Windows certificate;
- certificate password;
- Apple certificate;
- Apple certificate password;
- Apple ID/app password;
- Team ID.

Nessuna credenziale Desktopr.

Nessuna credenziale sensibile come normale workflow input.

## Criterio di completamento

Un developer può usare GitHub per buildare Desktopr senza infrastruttura esterna.

**Stato:** completata per le build non firmate; la firma con certificati reali e
la creazione della GitHub Release non sono ancora state eseguite.
`.github/workflows/build.yml` sostituisce build, sign e dist legacy: build
matrix Linux/Windows/macOS, firma opzionale nativa Tauri solo da secret del
repository, MSIX opzionale per Windows, artifact GitHub e GitHub Release
opzionale. Nessun R2/CDN, wrapper
privata, logging hosted o credenziale come input. Validato con actionlint e
shellcheck. Il run GitHub 35125323826 (commit `7563fc9`) ha prodotto su tutte e
tre le piattaforme AppImage/deb/rpm, installer NSIS + MSIX e DMG ad-hoc, con
checksum dei pacchetti verificati. Vedi `decisions/github-actions-build.md`.

---

# Fase 8 — Prima build standalone completa

## Obiettivo

Ottenere la prima versione realmente standalone prima di fare refactor estetici.

## Validare

- [x] install dipendenze;
- [x] TypeScript compile;
- [x] Rust compile;
- [x] SDK build;
- [x] WASM integration;
- [x] Linux build;
- [x] Windows build;
- [x] macOS build;
- [x] unsigned build;
- [ ] signing opzionale (percorso senza secret verificato; firma con certificati reali non eseguita).

## Regola

Non iniziare grandi cleanup prima di arrivare qui.

**Stato:** completata tranne la firma con certificati reali. Da install pulita
passano `npm ci`, typecheck, bridge, SDK (output identico a quello committato),
`cargo check --locked` con e senza updater, check del repo e WASM. I moduli
math e template sono eseguiti dall'host reale del runtime con
`npm run test:wasm-runtime`. Il run GitHub 35168452799 (commit `b592858`) ha
prodotto e **avviato** l'app su Linux, Windows e macOS. La validazione ha
trovato e corretto due bug che impedivano l'avvio (entitlement macOS riservata,
registrazione deep link su Linux) e un panic hook che nascondeva i panic.

---

# Fase 9 — Cleanup OSS

Solo dopo una build standalone funzionante.

## Eliminare

- `.DS_Store`;
- `deprecated/`;
- generated output inutile;
- branding Bubbledesk non più necessario;
- configurazione SaaS morta;
- vecchi debug file;
- codice non più raggiungibile.

## Aggiungere

- `.gitignore`;
- `LICENSE` (Apache 2.0);
- `README.md`;
- SECURITY.md;
- CONTRIBUTING.md.

## Criterio di completamento

La repository sembra un progetto OSS intenzionale, non una repo interna resa pubblica per caso.

**Stato:** completata. Rimossi `.DS_Store`, `deprecated/`, output SDK
committato, vecchio worker WASM JavaScript, caricamento del menu da file
all'avvio (disattivato), comandi non registrati e import inutilizzati (warning
Rust da 56 a 19). Aggiunti LICENSE Apache 2.0, README, SECURITY.md,
CONTRIBUTING.md e i campi licenza nei manifest. Il branding Bubbledesk resta
solo nel `frontend/` companion, rimandato alla fase 12. Da decidere più avanti:
il setup commentato di `tauri-plugin-prevent-default`.

---

# Fase 10 — Hardening

Da fare **dopo la migrazione funzionale**, ma **prima della release stabile**.

## Companion/window isolation

- [x] non fidarsi di `windowLabel` fornito dal frontend;
- [x] ricavare l'identità della finestra lato Rust;
- [x] enforcement backend degli scope filesystem;
- [x] separare capability main/companion.

## Remote capabilities

- [x] ridurre permessi non necessari (companion ridotte, comandi debug solo in dev; i permessi di `main` restano perché `Desktopr.tauri` è API documentata);
- [x] rivedere remote origin (origine di `APP_URL` più sottodomini, invariata e documentata);
- [x] documentare il trust model: Desktopr esegue la web app dello sviluppatore.

## Networking

- [x] valutare accesso localhost/LAN (ammesso per scelta);
- [x] limitarlo solo se contrario al modello desiderato (solo http/https, timeout, dimensione e frequenza limitati).

## Actions

- [x] `permissions:` minimali;
- [x] signing solo via Secrets;
- [x] valutare pinning SHA (azioni a SHA + Dependabot);
- [x] ridurre `curl | bash` e installazioni non pinned quando possibile (strumenti AppImage fissati e verificati).

**Stato:** completata. Identità delle finestre ricavata lato Rust, capability per finestra a runtime
(`main` completa, companion ridotta), scope filesystem imposti dal backend, controllo
`check:bridge-permissions`, limiti network, deadline WASM che non lascia thread in attesa, azioni
fissate a SHA con Dependabot e strumenti AppImage verificati. Verificato con test unitari (anche
rotti apposta) e con un self-test runtime su build debug (24 chiamate consentite/negate). Vedi
`decisions/window-isolation-and-capabilities.md`.

---

# Fase 11 — Test e CI

Dare una sistematina alla test automation.

## Rust

- [x] `cargo fmt --check`
- [x] `cargo clippy` (`-D warnings`, su Linux, Windows e macOS)
- [x] `cargo test` (inclusi i test del runtime WASM)

## TypeScript

- [x] `npm ci`
- [x] typecheck
- [x] bridge compile
- [x] SDK build

## Test prioritari

- path traversal;
- filesystem scopes;
- plugin storage isolation;
- invalid WASM;
- WASM timeout;
- config generation;
- capability generation;
- bridge initialization;
- SDK fallback;
- companion isolation dopo il relativo hardening.

## Cross-platform

- [x] Linux smoke build
- [x] Windows smoke build
- [x] macOS smoke build

**Stato:** completata. `.github/workflows/ci.yml` gira a ogni push su `main` e a ogni pull
request: controlli del repository, TypeScript, test bridge/SDK e test di generazione della config su
Linux; rustfmt, clippy e test Rust su tutte e tre le piattaforme; fmt/clippy del workspace WASM.
Tutti i test prioritari sono coperti (Rust: scope, traversal, storage dei plugin, WASM non valido e
timeout, isolamento companion; `scripts/test-config-generation.mjs`; `scripts/test-bridge.mjs`), e
ogni nuovo test è stato visto fallire con la guardia rotta. Emersi e corretti: `isDesktoprAvailable()`
documentato ma non esportato dall'SDK, e un campo usato solo su macOS che clippy segnalava su
Linux/Windows. Le smoke build con avvio restano in `build.yml` (on demand, verificate nella fase 10).

---

# Fase 12 — Companion

## Priorità

Bassa rispetto al core.

Va affrontato solo dopo:

- monorepo stabile;
- build workflow stabile;
- signing opzionale funzionante;
- runtime standalone.

## Nuovo ruolo

`companion` diventa un'app opzionale/playground di Desktopr.

Possibile posizione:

```text
apps/companion/
```

Deve essere:

- standalone;
- senza servizio hosted o app firmate;
- utile come playground/demo reale del runtime;
- eventualmente usato anche come example/dogfooding.

## Pubblicazione

GitHub Actions deve poter produrre:

- Windows build;
- Linux build;
- macOS build.

Pubblicabili come:

- GitHub Release.

**Stato:** completata, prima release ancora da pubblicare. Il companion vive in `apps/companion/`
(senza storia né lockfile del repo originale, senza `suffro-lib`, landing o link ospitati) al posto
di `frontend/`. L'URL inserito si apre nella finestra principale con tutti i permessi. Build con
`APP_FRONTEND=companion` / `frontend: companion`; release con `companion-release.yml` (tag
`companion-v`, note di primo avvio, macOS firmato e notarizzato obbligatorio, MSIX per lo Store
`Desktopr.Desktopr`). Niente modalità dev per provarlo. Verificato: CI 35244079211 e build companion
35244107334 su Linux, Windows e macOS con smoke test di avvio, checksum e identità MSIX corretti.
La prima release sarà la 3.0.0 (lo Store è a 2.3.0.0) dopo il caricamento dei secret Apple. Vedi
`decisions/companion-app.md`.

---

# Fase 13 — Secret scan finale

Prima di rendere pubblico:

- [x] cercare PAT/token;
- [x] cercare URL Git credentialed;
- [x] cercare riferimenti a `suffro-lib`;
- [x] cercare repository private;
- [x] cercare endpoint Desktopr/Bubbledesk morti;
- [x] verificare lockfile;
- [x] verificare history della nuova superficie pubblica.

Preferibile mantenere il nuovo monorepo con history pulita/sanitizzata se la vecchia history contiene credenziali.

**Stato:** completata, ma **dopo** che la repo era già pubblica. La history conteneva il token
`suffro-lib` in 5 commit (`frontend/package.json`, lockfile e configurazioni, dal 2025-08-26 al
2026-04-24). Il token è stato revocato dall'utente, la history riscritta con `git filter-repo`
(unico valore sostituito con `REDACTED-REVOKED-TOKEN`, stesso albero e stessi 216 commit) e
forzata su tutti i branch e tag. Backup della history precedente in
`~/desktopr-history-backup-20260918-0344.git`. Verificato su un clone nuovo dal remoto: zero token,
nemmeno nei ref delle PR. I vecchi oggetti possono restare raggiungibili su GitHub per SHA finché
non fa pulizia; serve il supporto GitHub per forzarla.

---

# Fase 14 — Rename e pubblicazione

Solo alla fine:

- [x] rinominare `tauri-skeleton` in `desktopr` (spostata anche l'owner: `suffro/desktopr`);
- [x] aggiornare package metadata;
- [x] aggiornare badge/link;
- [x] collegare le docs separate (assorbite in `docs/`, non più separate);
- [x] rendere pubblico;
- [ ] creare prima release OSS del runtime (pubblicata solo `companion-v3.0.0`).

**Stato:** quasi completata. La repo è pubblica come `suffro/desktopr`. Le docs sono state assorbite
in `docs/` e riscritte per il progetto open source: rimosse le pagine dei servizi (prezzi, crediti,
Edge, terms, privacy), il tracker ShareThis, i componenti YouTube, il video in home e i badge; le
pagine su avvio, architettura, companion, firma e configurazione descrivono il workflow aperto e i
secret del repository. `npm run docs:build` è nella CI e fallisce sui link morti. Il sito viene
caricato su Cloudflare dall'utente. Le chiavi di cache Cargo ora includono il nome della repo,
perché Tauri scrive percorsi assoluti negli artefatti e la cache di `tauri-skeleton` rompeva la
build dopo lo spostamento. Manca solo la prima release OSS del runtime.

## Gate finale

Prima del pubblico devono essere veri tutti:

- nessuna dipendenza privata;
- nessuna dipendenza SaaS Desktopr;
- build cross-platform funzionante;
- unsigned build funzionante;
- signing opzionale;
- distribution hosted rimossa;
- Actions autonome;
- CI verde;
- LICENSE presente;
- README sufficiente;
- secret scan pulita.

---

# Ordine operativo sintetico

```text
1. Inventory
2. Remove suffro-lib/private deps
3. Consolidate tauri-skeleton
4. Absorb project-globals
5. Absorb WASM repos
6. Remove hosted-service coupling
7. Migrate build/sign Actions
8. Delete hosted distribution workflow
9. Get standalone cross-platform build working
10. Cleanup OSS
11. Hardening
12. Tests/CI
13. Companion
14. Secret scan
15. Rename
16. Publish
```

## Regola principale

**Prima migrare e rendere standalone. Poi ripulire e hardenizzare. Companion per ultimo.**
