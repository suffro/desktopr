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

- [ ] Linux
- [ ] Windows
- [ ] macOS
- [ ] unsigned build funzionante;
- [ ] `workflow_dispatch`;
- [ ] `workflow_call` se pratico.

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

---

# Fase 8 — Prima build standalone completa

## Obiettivo

Ottenere la prima versione realmente standalone prima di fare refactor estetici.

## Validare

- [ ] install dipendenze;
- [ ] TypeScript compile;
- [ ] Rust compile;
- [ ] SDK build;
- [ ] WASM integration;
- [ ] Linux build;
- [ ] Windows build;
- [ ] macOS build;
- [ ] unsigned build;
- [ ] signing opzionale.

## Regola

Non iniziare grandi cleanup prima di arrivare qui.

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
- `LICENSE`;
- `README.md`;
- SECURITY.md;
- eventualmente CONTRIBUTING.md.

## Criterio di completamento

La repository sembra un progetto OSS intenzionale, non una repo interna resa pubblica per caso.

---

# Fase 10 — Hardening

Da fare **dopo la migrazione funzionale**, ma **prima della release stabile**.

## Companion/window isolation

- [ ] non fidarsi di `windowLabel` fornito dal frontend;
- [ ] ricavare l'identità della finestra lato Rust;
- [ ] enforcement backend degli scope filesystem;
- [ ] separare capability main/companion.

## Remote capabilities

- [ ] ridurre permessi non necessari;
- [ ] rivedere remote origin;
- [ ] documentare il trust model: Desktopr esegue la web app dello sviluppatore.

## Networking

- [ ] valutare accesso localhost/LAN;
- [ ] limitarlo solo se contrario al modello desiderato.

## Actions

- [ ] `permissions:` minimali;
- [ ] signing solo via Secrets;
- [ ] valutare pinning SHA;
- [ ] ridurre `curl | bash` e installazioni non pinned quando possibile.

---

# Fase 11 — Test e CI

## Rust

- [ ] `cargo fmt --check`
- [ ] `cargo clippy`
- [ ] `cargo test`

## TypeScript

- [ ] `npm ci`
- [ ] typecheck
- [ ] bridge compile
- [ ] SDK build

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

- [ ] Linux smoke build
- [ ] Windows smoke build
- [ ] macOS smoke build

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
- senza servizio hosted;
- utile come playground/demo reale del runtime;
- eventualmente usato anche come example/dogfooding.

## Distribuzione

GitHub Actions deve poter produrre:

- Windows build;
- Linux build;
- macOS build.

Pubblicabili come:

- GitHub Actions artifacts;
- eventualmente GitHub Release.

## macOS

Non rendere obbligatorio Apple Developer Program.

Finché non si vuole pagare:

- build macOS unsigned/non-notarized;
- documentare il warning Gatekeeper;
- niente dipendenza da firma Apple per mantenere il progetto.

Non introdurre un servizio hosted per risolvere questo problema.

---

# Fase 13 — Secret scan finale

Prima di rendere pubblico:

- [ ] cercare PAT/token;
- [ ] cercare URL Git credentialed;
- [ ] cercare riferimenti a `suffro-lib`;
- [ ] cercare repository private;
- [ ] cercare endpoint Desktopr/Bubbledesk morti;
- [ ] verificare lockfile;
- [ ] verificare history della nuova superficie pubblica.

Preferibile mantenere il nuovo monorepo con history pulita/sanitizzata se la vecchia history contiene credenziali.

---

# Fase 14 — Rename e pubblicazione

Solo alla fine:

- [ ] rinominare `tauri-skeleton` in `desktopr` o nome definitivo;
- [ ] aggiornare package metadata;
- [ ] aggiornare badge/link;
- [ ] collegare le docs separate;
- [ ] rendere pubblico;
- [ ] creare prima release OSS.

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
