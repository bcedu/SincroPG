# Disseny CliPG (ClientPartidesGuardades)

## 1. Arquitectura general del CliPG

### Videojoc (struct)

**Responsabilitat:** Representa un joc amb les seves partides locals i la carpeta on estan.

#### Atributs

| Atribut            | Tipus                  | Descripció                                         |
|--------------------|------------------------|----------------------------------------------------|
| `nom`              | `String`               | Nom del joc.                                       |
| `local_folder`     | `String`               | Carpeta local on estan les partides d’aquest joc.  |
| `partides_locals`  | `Vec<PartidaGuardada>` | Llista de partides locals.                         |
| `partides_remotes` | `Vec<PartidaGuardada>` | Partides que hi ha al servidor (per sincronitzar). |
| `partides_guardades` | `Vec<PartidaGuardadaConfig>` | Partides guardades al client desde la ultima sincornitzacio.                    |

#### Mètodes

| Fet   | Mètode                                                                  | Retorn / Paràmetres                                                   | Descripció                                                                   |
|-------|-------------------------------------------------------------------------|-----------------------------------------------------------------------| ---------------------------------------------------------------------------- |
| ✅     | `new(nom: &str, local_folder: &str) -> Self`                            | `Videojoc`                                                            | Constructor amb nom i carpeta local.                                         |
| ✅     | `from(videojoc: &Videojoc) -> Self`                 | `Videojoc`                                                            | Constructor amb nom i carpeta local.                                         |
| ✅     | `load_partides_locals()`                                                | `()`                                                                  | Llegeix les partides de disc i les posa a `partides_locals`.                        |
| ✅     | `fetch_partides_remotes(api: &PgAPI)`                                   | `()`                                                                  | Demana al servidor les partides d’aquest joc i les posa a `partides_remotes`. |
| ✅     | `sync(api: &PgAPI)`                                                     | `()`                                                                  | Sincronitza les partides locals amb les del servidor.                        |
| ✅ | `resoldre_conflicte(local: &PartidaGuardada, server: &PartidaGuardada)` | `()`                | Gestiona conflictes (p. ex. renombrar i guardar les dues).                   |

---

### PartidaGuardada (struct)

**Responsabilitat:** Representa una partida concreta amb metadades útils per sincronitzar.

#### Atributs

| Atribut     | Tipus    | Descripció                                      |
|-------------|----------|-------------------------------------------------|
| `videojoc`  | `String` | Nom del videojoc.                               |
| `nom`       | `String` | Nom de la partida.                              |
| `path`      | `String` | Ruta completa del fitxer local.                 |
| `timestamp` | `u32`    | Última modificació (per comparar amb servidor). |
| `hash`      | `String` | Hash de contingut per detectar canvis.          |
| `last_sync_hash`      | `String` | Hash del últim contingut que tenim sincronitzat amb el servidor.          |

#### Mètodes

| Fet   | Mètode                                                              | Retorn / Paràmetres | Descripció                                                  |
|-------|---------------------------------------------------------------------| ------------------- |-------------------------------------------------------------|
| ✅     | `new(path: &str) -> Self`                                           | `PartidaGuardada`   | Crea una instància llegint metadata (timestamp, hash).      |
| ✅     | `from_partida_guardada(partida_guardada: &PartidaGuardada) -> Self` | `PartidaGuardada`   | Crea una instància copiant les dades.                       |
| ✅     | `update_metadata()`                                                 | `()`                | Torna a calcular timestamp i hash si el fitxer ha canviat.  |
| ✅     | `pujar_partida_guardada(api: PgAPI)`                             | `()`                | Puja la partida guardada al servidor.                       |
| ✅     | `descarregar_partida_guardada(api: PgAPI)`                       | `()`                | Es descarrega la partida guardada del servidor i la guarda. |
| ✅ | `duplicar_fitxer(nou_nom: String)`                                  | `()`                | Duplica el fitxer de la partida local amb el nou nom.       |
| ✅     | `eliminar_partida_guardada()`                                     | `()`                | Elimina el fitxer de la partida guardada.                  
---

### PgAPI (struct)

**Responsabilitat:** Parlar amb la API del servidor per consultar, descarregar i pujar partides guardades.

#### Atributs

| Atribut    | Tipus           | Descripció        |
|------------|-----------------|-------------------|
| `url` | `String`        | URL del servidor. |
| `usuari` | `String`        | Usuari.           |
| `contrassenya` | `String`        | Contrasenya.      |

#### Mètodes

| Fet | Mètode                                                                 | Retorn / Paràmetres    | Descripció                                                                        |
|--|------------------------------------------------------------------------|------------------------|-----------------------------------------------------------------------------------|
| ✅ | `new(usuari: String, contrassenya: String) -> Self`                    | `PgAPI`             | Constructor.                                                                      |
| ✅ | `probar_connexio(&self) -> bool`                                       | `bool`                 | Proba de connectarse amb les credencials proporcionades.                          |
| ✅ | `get_videojocs(&self) -> Vec<Videojoc>`                                | `Vec<String>`          | Obté el llistat de videojocs del servidor.                                        |
| ✅ | `get_partides_guardades(nom_videojoc: String) -> Vec<PartidaGuardada>` | `Vec<PartidaGuardada>` | Obté les partides guardades del servidor per el videojoc que es digui `videojoc`. |
| ✅ | `post_partida_guardada(partida_guardada: &PartidaGuardada)`            | `()`                   | Puja la partida guardada al servidor.                                             |
| ✅ | `get_partida_guardada(partida_guardada: &PartidaGuardada) -> String`   | `String`               | Retorna el contingut del fitxer de la partida guardada que hi ha al servidor.     |
| ✅ | `delete_partida_guardada(partida_guardada: &PartidaGuardada)`          | `()`                   | Elimina la partida guardada del servidor.                                          |

#### Structs que representen respostes de la API:

`VideojocAPI`:
- `id`: String
- `nom`: String

`PartidaGuardadaAPI`:
- `nom`: String
- `hash`: String

`PartidaGuardadaContingutAPI`:
- `nom`: String
- `contingut`: String

---

### CliPG (struct)

**Responsabilitat:** Gestiona la llista de jocs i la sincronització global.

#### Atributs

| Atribut  | Tipus           | Descripció                               |
|----------| --------------- |------------------------------------------|
| `api`    | `PgAPI`     | Client per comunicar-se amb el servidor. |
| `vjocs`  | `Vec<Videojoc>` | Llista de jocs locals configurats.       |
| `config` | `CliPgConfig` | Dades gaurdades de la aplicacio.         |


#### Mètodes

| Fet | Mètode                                                     | Retorn / Paràmetres   | Descripció                                                                                                                                 |
|----|-----------------------------------------------------------|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------|
| ✅  | `default() -> Self`                                       | `CliPG`               | Constructor per defecte (pots cridar `get_credentials()`).                                                                                 |
| ✅  | `load_local_jocs()`                                       | `Vec<VideojocConfig>` | Carrega tots els jocs locals (crea instàncies `Videojoc` amb la seva carpeta). Retorna una llista amb els jocs que no s'han pogut carregar |
| ✅ | `sync_all()`                                              | `()`                  | Sincronitza tots els jocs.                                                                                                                 |
| ✅   | `sync_joc(joc: &mut Videojoc)`                            | `String`              | Sincronitza un joc concret amb el servidor.                                                                                                |
| ✅  | `get_config_path() -> PathBuf`                            | `()`                  | Retorna el path al fitxer de configuracio.                                                                                                 |
| ✅  | `save_config(config: CliPgConfig)`                        | `()`                  | Guarda al disc la configuracio proporcionada.                                                                                              |
| ✅  | `load_or_create_config() -> CliPgConfig`                  | `()`                  | Carrega al configuracio que hi hagi guardada actualemtnen disc                                                                             |
| ✅  | `afegir_joc(path: String) -> Result<(), String>`          | `()`                  | Afegeix un joc als jocs habilitats (`config.videojocs_habilitats`)                                                                         |
| ✅  | `eliminar_joc(videojoc_id: String) -> Result<(), String>` | `()`                  | Eliminar un joc als jocs habilitats (`config.videojocs_habilitats`)                                                                        |

#### Structs que representen les dades guardades de la aplicació:

`CliPgConfig`:
- `server`: ServerConfig
- `videojocs_habilitats`: Vec<VideojocConfig>

`ServerConfig`:
- `url`: String
- `usuari`: String
- `contrasenya`: String

`VideojocConfig`:
- `nom`: String
- `path`: String
- `partides_guardades`: Vec<PartidaGuardadaConfig>

`PartidaGuardadaConfig`:
- `path`: String
- `hash`: String

---

## 2. Flux de la interfície (UI)

Per la UI es farà servir `egui`.

### Dashboard — SincroPG

┌──────────────────────────────────────────────────────────────┐
│ SincroPG                                      [⚙ Configuració] │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  [+ Afegir joc]                         [⟳ Sincronitzar tots] │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  🎮 Videojocs habilitats                                    │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Nom del joc    │ Estat        │ Última sinc. │ Accions │  │
│  ├────────────────────────────────────────────────────────┤  │
│  │ Skyrim         │ 🟢 Actualitzat │ 29/03 18:20 │ [Sync] [✏] [🗑] │
│  │ Baldurs Gate   │ 🟡 Desfasat    │ 28/03 21:10 │ [Sync] [✏] [🗑] │
│  │ Elden Ring     │ 🔴 Error       │ 27/03 12:05 │ [Sync] [✏] [🗑] │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  (Si no hi ha jocs)                                          │
│      No hi ha videojocs configurats.                         │
│      Prem "+ Afegir joc" per començar.                       │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Estat servidor: 🟢 Connectat (192.168.1.10:8080)            │
│                                                              │
│  Activitat:                                                  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ ✔ Skyrim sincronitzat correctament                    │  │
│  │ ✖ Error a Elden Ring: No es pot accedir a la ruta    │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
### Afegir / Editar Joc

┌──────────────────────────────────────────────────────────────┐
│ ← Tornar                                                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Nom del joc:                                                │
│  [________________________________________________________]  │
│                                                              │
│  Ruta local de la partida:                                   │
│  [____________________________________________________] [📂] │
│                                                              │
│  Carpeta al servidor (opcional):                             │
│  [________________________________________________________]  │
│                                                              │
│  Opcions avançades:                                          │
│  ☐ Sincronitzar automàticament                               │
│  ☐ Només pujar fitxers (no baixar)                           │
│                                                              │
│                                                              │
│                    [Guardar]   [Cancel·lar]                   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
### Configuració

┌──────────────────────────────────────────────────────────────┐
│ ← Tornar                                                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  🌐 Configuració del Servidor                               │
│                                                              │
│  Host / IP:                                                  │
│  [__________________________________________]                │
│                                                              │
│  Port:                                                        │
│  [________]                                                  │
│                                                              │
│  Usuari:                                                      │
│  [__________________________________________]                │
│                                                              │
│  Contrasenya:                                                 │
│  [__________________________________________]                │
│                                                              │
│  [Provar connexió]                                            │
│                                                              │
│  Estat connexió:                                              │
│  🟢 Connexió correcta                                         │
│  🔴 Error de connexió                                         │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  🔄 Opcions generals                                          │
│                                                              │
│  ☐ Sincronitzar en iniciar l'aplicació                       │
│  ☐ Sincronitzar automàticament cada ___ minuts               │
│  ☐ Mostrar notificacions                                      │
│                                                              │
│                                                              │
│                    [Guardar canvis]                           │
│                                                              │
└──────────────────────────────────────────────────────────────┘
### Sincronització en curs

┌──────────────────────────────────────────────┐
│ Sincronitzant Baldurs Gate...               │
├──────────────────────────────────────────────┤
│                                              │
│  [██████████████░░░░░░░░░] 65%              │
│                                              │
│  ✔ Comparant fitxers                        │
│  ✔ Pujant 3 fitxers                         │
│  ⟳ Baixant 1 fitxer                         │
│                                              │
│                     [Cancel·lar]            │
│                                              │
└──────────────────────────────────────────────┘
---

## 3. Lògica de sincronització

#### Notes Generals

1. Cada `Videojoc` coneix la seva carpeta local → no cal atribut global `local_folder`.
2. Cada `Videojoc` té la llista de partides locals i del servidor → sincronització encapsulada.
3. `CliPG` només gestiona la llista de jocs i la crida a sincronització global.
4. Permet escalar fàcilment a més jocs i més metadades sense tocar el client.

### 3.1 Procés general

1. Llegir partides locals
2. Llegir partides del servidor
3. Comparar per ID, local.hash, remote.hash i last_sync_hash
4. Decidir acció per sincornitzar
5. Actualitzar el CliPgConfig guardat en local per tindre els ultims hash sincronitzatsde cada partida

### 3.2 Algoritme de sincronització

La sincronització es basa en una comparació de tres valors per cada partida:

- `local_hash` — hash del fitxer local
- `remote_hash` — hash del fitxer al servidor
- `last_sync_hash` — hash de l’última versió sincronitzada

Per cada partida present en `local ∪ remote` s'aplica l’algoritme següent:

| Estat | Condició | Acció |
|------|----------|------|
| Iguals | `local_hash == remote_hash` | No fer res |
| Modificat només al servidor | `local_hash == last_sync_hash && remote_hash != last_sync_hash` | Descarregar del servidor |
| Modificat només en local | `remote_hash == last_sync_hash && local_hash != last_sync_hash` | Pujar al servidor |
| Modificat en ambdós | `local_hash != last_sync_hash && remote_hash != last_sync_hash && local_hash != remote_hash` | Conflicte |
| Només existeix localment (fitxer nou) | `local_exists && !remote_exists && last_sync_hash != local_hash` | Pujar al servidor |
| Només existeix remotament (fitxer nou) | `!local_exists && remote_exists && last_sync_hash != remote_hash` | Descarregar del servidor |
| Eliminat al servidor | `local_exists && !remote_exists && last_sync_hash == local_hash` | Eliminar local |
| Eliminat en local | `!local_exists && remote_exists && last_sync_hash == remote_hash` | Eliminar al servidor |

### 3.3 Resolució de conflictes
No es sobreescriu mai informació. Fem copies:
  ```
  save/
  ├─ save.dat
  ├─ save_LOCAL.dat
  ├─ save_SERVER.dat
  ```
  


---

## 4. CLI per consola de comandes

```
Usage: CliPG [OPTIONS]

Options:

-l, --list                  Mostra tots els videojocs habilitats per sincornitzar-se
-a, --add <videojoc_path>   Afegeix un videojoc amb la ruta donada
-r, --remove <videojoc_id>  Elimina un videojoc pel seu ID
-s, --sync_all              Sincronitza tots els videojocs
-v, --sync <videojoc_id>    Sincronitza un videojoc pel seu ID
-h, --help                  Print help
-V, --version               Print version
```
