# Desktopr — Piano di dismissione del SaaS e conversione open source

## Obiettivo

Dismettere **Desktopr come servizio online** senza perdere codice, dati o possibilità di migrazione, quindi trasformare il progetto in un **tool open source standalone** per costruire applicazioni desktop Tauri a partire da web app.

Principio guida:

> **Spegnere prima il servizio, preservare tutto ciò che può servire, migrare dopo. Cancellare solo quando la nuova versione OSS è stabile.**

---

# Fase 1 — Shutdown pubblico del servizio

## Obiettivo
Impedire nuovi utilizzi del SaaS mentre l'infrastruttura viene progressivamente dismessa.

## Azioni

- [ ] Disabilitare nuove registrazioni.
- [ ] Disabilitare la creazione di nuovi progetti.
- [ ] Disabilitare nuove build.
- [ ] Disabilitare acquisti, crediti e pagamenti.
- [ ] Disabilitare eventuali trial.
- [ ] Disabilitare operazioni che generano nuovi costi infrastrutturali.
- [ ] Sostituire la landing/app pubblica con una pagina semplice di shutdown.
- [ ] Indicare chiaramente che il servizio hosted non è più disponibile.
- [ ] Evitare di promettere continuità del servizio SaaS.

## Da verificare prima dello spegnimento

- [ ] Esistono utenti attivi?
- [ ] Esistono build distribuite che usano ancora `cdn.desktopr.app`?
- [ ] Esistono updater Desktopr attivi in applicazioni già distribuite?
- [ ] Esistono webhook Stripe o callback esterne ancora attive?
- [ ] Esistono domini o API utilizzati da software già distribuito?

## Criterio di completamento

Desktopr non accetta più nuovi utenti, build, pagamenti o operazioni SaaS.

---

# Fase 2 — Preservation e sicurezza

## Obiettivo
Congelare il sistema senza distruggere materiale necessario alla migrazione.

## Azioni

- [ ] Fare backup dei dati importanti.
- [ ] Esportare/configurare backup di Firebase/Firestore se contiene dati da conservare.
- [ ] Preservare temporaneamente R2.
- [ ] Preservare configurazioni di Cloudflare.
- [ ] Preservare le vecchie repository Bubbledesk private.
- [ ] Salvare variabili d'ambiente e configurazioni necessarie alla comprensione del sistema.
- [ ] Documentare quali servizi sono collegati tra loro.
- [ ] Revocare/ruotare il PAT GitHub trovato hardcoded nei repository.
- [ ] Ruotare altri secret che potrebbero essere stati committati o condivisi.
- [ ] Eseguire successivamente una scansione completa dei secret prima di pubblicare codice.

## NON cancellare ancora

Non eliminare in questa fase:

- vecchie repository Bubbledesk;
- bucket R2;
- Firebase;
- dati utenti;
- vecchi artefatti di build;
- configurazioni Cloudflare;
- CDN Desktopr;
- signing material necessario per comprendere la pipeline;
- vecchi workflow GitHub Actions.

L'obiettivo iniziale è **spegnere**, non **distruggere**.

---

# Fase 3 — Spegnimento dell'infrastruttura SaaS

## Obiettivo
Ridurre progressivamente l'infrastruttura e i costi del vecchio servizio.

## Componenti da dismettere

### SaaS

- [ ] `app`
- [ ] `auth`
- [ ] `marketing`
- [ ] `firebase-functions`

### Cloudflare Workers

- [ ] Stripe worker
- [ ] email worker
- [ ] GitHub proxy
- [ ] R2/S3 proxy
- [ ] GitHub Actions logs proxy
- [ ] domain masking worker
- [ ] status workers
- [ ] altri worker esclusivamente SaaS

## Stripe

- [ ] Bloccare nuovi acquisti.
- [ ] Verificare eventuali subscription ancora attive.
- [ ] Disabilitare webhook non più necessari.
- [ ] Conservare i dati amministrativi/fiscali richiesti.

## Firebase

- [ ] Rendere non operative le funzioni SaaS.
- [ ] Conservare temporaneamente i dati.
- [ ] Evitare cancellazioni irreversibili fino alla fine della migrazione.

## Cloudflare

- [ ] Disabilitare gradualmente i Worker non più necessari.
- [ ] Conservare DNS e domini finché non è chiaro cosa servirà al progetto OSS.
- [ ] Ridurre le superfici pubbliche esposte.

## Criterio di completamento

Il vecchio Desktopr hosted non offre più funzionalità SaaS e genera costi infrastrutturali minimi.

---

# Fase 4 — Inventario tecnico prima della migrazione

## Obiettivo
Separare chiaramente il prodotto tecnico dal vecchio servizio hosted.

## Repository principali

### Da preservare e migrare

- `Bubbledesk/tauri-skeleton`
- `Bubbledesk/github-actions`

### Da valutare

- `companion`
- `docs`
- `project-globals`
- `wasm-module-template`
- `wasm-modules`

### Principalmente SaaS / da archiviare

- `app`
- `auth`
- `marketing`
- `firebase-functions`
- Cloudflare Workers

## Output atteso

Creare una tabella:

| Componente | Migrare | Riscrivere | Archiviare | Eliminare dopo |
|---|---:|---:|---:|---:|
| Tauri runtime | ✓ | | | |
| Desktopr bridge | ✓ | | | |
| SDK | ✓ | | | |
| GitHub build workflow | ✓ | parziale | | |
| Signing workflow | ✓ | parziale | | |
| R2 distribution | | | ✓ | ✓ |
| Firebase backend | | | ✓ | eventuale |
| Stripe | | | ✓ | eventuale |

---

# Fase 5 — Creazione della nuova repository OSS

## Obiettivo
Non rendere pubblico direttamente il vecchio repository interno.

Creare una repository nuova e pulita, ad esempio:

```text
desktopr/
├── src-tauri/
├── src-ts/
├── sdk/
├── config/
├── scripts/
├── examples/
├── docs/
├── .github/
│   └── workflows/
├── README.md
├── LICENSE
├── SECURITY.md
└── .gitignore
```

## Perché repository nuova

- history pulita;
- nessun vecchio secret;
- niente rumore SaaS;
- niente commit interni irrilevanti;
- naming coerente;
- storia GitHub più professionale.

## Criterio di completamento

La nuova repo contiene il core Desktopr ma non dipende dalla storia del SaaS.

---

# Fase 6 — Migrazione del core Desktopr

## Obiettivo
Portare `tauri-skeleton` quasi invariato prima di fare refactor importanti.

## Migrare

- [ ] bridge Rust;
- [ ] bridge TypeScript;
- [ ] SDK;
- [ ] filesystem;
- [ ] window APIs;
- [ ] clipboard;
- [ ] notification;
- [ ] file dialogs;
- [ ] menus/context menus;
- [ ] global shortcuts;
- [ ] autostart;
- [ ] deep links;
- [ ] diagnostics;
- [ ] networking;
- [ ] WASM runtime;
- [ ] plugin storage;
- [ ] configurazione Tauri;
- [ ] updater come feature opzionale.

## Regola

**Non riscrivere il core durante la migrazione.**

Prima ottenere una versione standalone funzionante; poi migliorare.

---

# Fase 7 — Eliminazione delle dipendenze SaaS

## Rimuovere

- Firebase;
- Stripe;
- Desktopr authentication;
- build credits;
- hosted orchestration;
- Desktopr R2;
- Desktopr CDN come requisito;
- proxy Cloudflare;
- logging backend Desktopr;
- callback SaaS;
- token di accesso alla vecchia wrapper repo.

## Updater

L'updater deve diventare:

- opzionale;
- configurabile dallo sviluppatore;
- non legato a `cdn.desktopr.app`.

---

# Fase 8 — Nuova pipeline GitHub Actions

## Obiettivo

Passare da:

```text
build
→ R2
→ signing
→ R2
→ dist
→ cdn.desktopr.app
```

a:

```text
source
→ GitHub Actions
→ Linux / Windows / macOS
→ optional signing
→ GitHub Artifacts
→ optional GitHub Release
```

## Workflow

### Build

- [ ] Linux
- [ ] Windows
- [ ] macOS
- [ ] `workflow_dispatch`
- [ ] `workflow_call`

### Signing

Signing opzionale tramite GitHub Secrets.

### Distribution

Default:

- GitHub Actions artifacts;
- GitHub Releases.

Il vecchio `dist.yml` probabilmente può essere eliminato o fortemente ridotto.

---

# Fase 9 — Cleanup OSS

Solo dopo che la migrazione funziona.

## Eliminare

- [ ] `.DS_Store`
- [ ] directory `deprecated/`
- [ ] generated output non necessario
- [ ] branding Bubbledesk residuo
- [ ] commenti/debug interni inutili
- [ ] configurazioni SaaS morte

## Aggiungere

- [ ] `LICENSE`
- [ ] `.gitignore`
- [ ] `README.md`
- [ ] `SECURITY.md`
- [ ] eventualmente `CONTRIBUTING.md`

---

# Fase 10 — Hardening

Da fare **dopo la migrazione**, ma **prima della release pubblica stabile**.

## Priorità

### Companion isolation

- [ ] Derivare lato Rust l'identità della finestra chiamante.
- [ ] Non fidarsi di `windowLabel` fornito dal frontend.
- [ ] Enforcement dell'accesso persistent/cache nel backend Rust.

### Tauri capabilities

- [ ] Separare capability main e companion.
- [ ] Ridurre i permessi remoti non necessari.
- [ ] Verificare i domini autorizzati.

### Networking

- [ ] Valutare accesso a localhost e LAN.
- [ ] Documentare o limitare i comportamenti sensibili.

### GitHub Actions

- [ ] Signing credentials solo tramite GitHub Secrets.
- [ ] `permissions:` espliciti e minimali.
- [ ] Valutare pinning delle Actions a commit SHA.
- [ ] Eliminare installazioni remote non necessarie tramite `curl | bash`.

---

# Fase 11 — Test e CI

## Rust

- [ ] `cargo fmt --check`
- [ ] `cargo clippy`
- [ ] `cargo test`

## TypeScript

- [ ] install pulito con `npm ci`
- [ ] typecheck
- [ ] bridge compilation
- [ ] SDK build

## Test prioritari

- path traversal;
- filesystem scope;
- companion isolation;
- plugin storage isolation;
- WASM timeout;
- invalid WASM;
- config generation;
- remote capabilities;
- SDK fallback;
- bridge initialization.

## Smoke build

- [ ] Linux
- [ ] Windows
- [ ] macOS

---

# Fase 12 — Pubblicazione open source

Prima di rendere pubblico:

- [ ] secret scan completa;
- [ ] nessun PAT/token nella repository;
- [ ] CI verde;
- [ ] build cross-platform funzionante;
- [ ] README sufficiente per partire;
- [ ] esempio minimale;
- [ ] LICENSE presente;
- [ ] security boundary documentati;
- [ ] nessuna dipendenza dal vecchio SaaS.

Solo a questo punto rendere Desktopr pubblico.

---

# Fase 13 — Cancellazione definitiva della vecchia infrastruttura

Da fare solo quando:

- la versione OSS è funzionante;
- nessuna app distribuita dipende più dai vecchi endpoint;
- non servono più dati per migrazione/debug;
- sono stati conservati eventuali backup obbligatori.

A quel punto:

- [ ] eliminare bucket R2 inutili;
- [ ] eliminare Worker Cloudflare morti;
- [ ] eliminare Firebase non più necessario;
- [ ] eliminare vecchi secret;
- [ ] archiviare definitivamente le repository Bubbledesk;
- [ ] rimuovere DNS/API obsolete;
- [ ] eventualmente mantenere `desktopr.com` come landing del progetto OSS.

---

# Ordine raccomandato finale

```text
1. Bloccare il SaaS
2. Fare backup e ruotare i secret
3. Spegnere l'infrastruttura pubblica
4. Conservare temporaneamente dati e vecchie repo
5. Fare l'inventario del codice
6. Creare la nuova repo OSS
7. Migrare il core quasi invariato
8. Sostituire R2/CDN con GitHub
9. Ripulire la repo
10. Fare hardening
11. Aggiungere test e CI
12. Pubblicare
13. Cancellare definitivamente il vecchio backend
```

## Regola più importante

**Non cancellare ciò che può servire alla migrazione finché Desktopr OSS non è stabile.**

Shutdown e deletion sono due operazioni diverse.
