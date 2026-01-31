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

#### Mètodes

| Fet   | Mètode                                                                  | Retorn / Paràmetres                                                   | Descripció                                                                   |
|-------|-------------------------------------------------------------------------|-----------------------------------------------------------------------| ---------------------------------------------------------------------------- |
| ✅     | `new(nom: &str, local_folder: &str) -> Self`                            | `Videojoc`                                                            | Constructor amb nom i carpeta local.                                         |
| ✅     | `load_partides_locals()`                                                | `()`                                                                  | Llegeix les partides de disc i les posa a `partides_locals`.                        |
| ✅     | `fetch_partides_remotes(api: &SerPGAPI)`                                | `()`                                                                  | Demana al servidor les partides d’aquest joc i les posa a `partides_remotes`. |
| ✅     | `sync(api: &SerPGAPI)`                                                  | `()`                                                                  | Sincronitza les partides locals amb les del servidor.                        |
| DOING | `resoldre_conflicte(local: &PartidaGuardada, server: &PartidaGuardada)` | `()`                | Gestiona conflictes (p. ex. renombrar i guardar les dues).                   |

---

### PartidaGuardada (struct)

**Responsabilitat:** Representa una partida concreta amb metadades útils per sincronitzar.

#### Atributs

| Atribut     | Tipus    | Descripció                                      |
| ----------- |----------| ----------------------------------------------- |
| `nom`       | `String` | Nom de la partida.                              |
| `path`      | `String` | Ruta completa del fitxer local.                 |
| `timestamp` | `u32`    | Última modificació (per comparar amb servidor). |
| `hash`      | `String` | Hash de contingut per detectar canvis.          |

#### Mètodes

| Fet   | Mètode                                                              | Retorn / Paràmetres | Descripció                                                  |
|-------|---------------------------------------------------------------------| ------------------- |-------------------------------------------------------------|
| ✅     | `new(path: &str) -> Self`                                           | `PartidaGuardada`   | Crea una instància llegint metadata (timestamp, hash).      |
| ✅     | `from_partida_guardada(partida_guardada: &PartidaGuardada) -> Self` | `PartidaGuardada`   | Crea una instància copiant les dades.                       |
| ✅     | `update_metadata()`                                                 | `()`                | Torna a calcular timestamp i hash si el fitxer ha canviat.  |
| ✅     | `pujar_partida_guardada(api: SerPGAPI)`                             | `()`                | Puja la partida guardada al servidor.                       |
| ✅     | `descarregar_partida_guardada(api: SerPGAPI)`                       | `()`                | Es descarrega la partida guardada del servidor i la guarda. |
| ✅ | `duplicar_fitxer(nou_nom: String)`                                  | `()`                | Duplica el fitxer de la partida local amb el nou nom.       |

---

### CliPG (struct)

**Responsabilitat:** Gestiona la llista de jocs i la sincronització global.

#### Atributs

| Atribut | Tipus           | Descripció                               |
| ------- | --------------- | ---------------------------------------- |
| `api`   | `SerPGAPI`     | Client per comunicar-se amb el servidor. |
| `vjocs` | `Vec<Videojoc>` | Llista de jocs locals configurats.       |

#### Mètodes

| Fet | Mètode                         | Retorn / Paràmetres | Descripció                                                                     |
|-----|--------------------------------| ------------------- | ------------------------------------------------------------------------------ |
| x   | `new(api: SerPGAPI) -> Self`   | `CliPG`               | Constructor amb l’API.                                                         |
| x   | `default() -> Self`            | `CliPG`               | Constructor per defecte (pots cridar `get_credentials()`).                     |
| x   | `load_local_jocs()`            | `()`                | Carrega tots els jocs locals (crea instàncies `Videojoc` amb la seva carpeta). |
| x   | `sync_all()`                   | `()`                | Sincronitza tots els jocs.                                                     |
| x   | `sync_joc(joc: &mut Videojoc)` | `()`           | Sincronitza un joc concret amb el servidor.                                    |
| x   | `show_status()`                | `()`                           | Mostra estat global de sincronització.                                         |

### SerPGAPI (struct)

**Responsabilitat:** Parlar amb la API del servidor per consultar, descarregar i pujar partides guardades.

#### Atributs

| Atribut    | Tipus           | Descripció        |
|------------|-----------------|-------------------|
| `url` | `String`        | URL del servidor. |
| `usuari` | `String`        | Usuari.           |
| `contrassenya` | `String`        | Contrasenya.      |

#### Mètodes

| Fet | Mètode                                                                 | Retorn / Paràmetres    | Descripció                                                                        |
|-----|------------------------------------------------------------------------|------------------------|-----------------------------------------------------------------------------------|
| x   | `new(usuari: String, contrassenya: String) -> Self`                    | `SerPGAPI`             | Constructor.                                                                      |
| x   | `get_partides_guardades(nom_videojoc: String) -> Vec<PartidaGuardada>` | `Vec<PartidaGuardada>` | Obté les partides guardades del servidor per el videojoc que es digui `videojoc`. |
| x   | `post_partida_guardada(partida_guardada: &PartidaGuardada)`            | `()`                   | Puja la partida guardada al servidor.                                             |
| x   | `get_partida_guardada(partida_guardada: &PartidaGuardada) -> String`         | `String`               | Retorna el contingut del fitxer de la partida guardada que hi ha al servidor.     |


### Notes Generals
1. Cada `Videojoc` coneix la seva carpeta local → no cal atribut global `local_folder`.
2. Cada `Videojoc` té la llista de partides locals i del servidor → sincronització encapsulada.
3. `CliPG` només gestiona la llista de jocs i la crida a sincronització global.
4. Permet escalar fàcilment a més jocs i més metadades sense tocar el client.


---

## 2. Flux de la interfície (UI)

Per la UI es farà servir `egui`.

### 2.1 Inici de l’aplicació

```
CliPG s'inicia
 ↓
S'obre MainWindow
 ↓
Intent automàtic de connexió amb SPG
```

### 2.2 Sense connexió amb el servidor

- Missatge:
    - ❌ No es pot connectar amb el servidor
- Botó:
    - Configurar servidor

```
[ MainWindow ]
┌──────────────────────────────┐
│ ❌ Sense connexió amb SPG     │
│                              │
│ [ Configurar servidor ]      │
└──────────────────────────────┘
```

---

### 2.3 Finestra de configuració del servidor

Dades configurables:
- URL del servidor SPG
- Usuari
- Contrasenya

```
[ ServerConfigWindow ]
┌──────────────────────────────┐
│ Servidor:  https://...       │
│ Usuari:    __________       │
│ Password:  *********        │
│                              │
│ [ Provar ]   [ Desar ]       │
└──────────────────────────────┘
```

---

### 2.4 Connexió correcta

```
Connexió OK
 ↓
Obtenir jocs habilitats
 ↓
Mostrar jocs
 ↓
Iniciar sincronització
```

```
[ MainWindow ]
┌────────────────────────────────────────┐
│ Connexió: ✅                            │
│                                        │
│ Joc              Estat                 │
│ ------------------------------------- │
│ Skyrim           🔄 Sincronitzant...   │
│ Baldur's Gate    ✅ OK                 │
│ Witcher 3        ⚠ Conflicte           │
│                                        │
│ [ Forçar sync ]  [ Config joc ]        │
└────────────────────────────────────────┘
```

---

## 3. Lògica de sincronització

### 3.1 Procés general

1. Llegir partides locals
2. Llegir partides del servidor
3. Comparar per ID, hash i timestamp
4. Decidir acció

### 3.2 Casos possibles

| Estat | Acció |
|-----|------|
| Només local | ⬆️ Pujar al servidor |
| Només servidor | ⬇️ Descarregar |
| Iguals | ✔️ No fer res |
| Diferents | ⚠ Conflicte |

### 3.3 Resolució de conflictes

Quan una partida ha canviat a local i servidor:

```
save/
 ├─ save.dat
 ├─ save_LOCAL.dat
 ├─ save_SERVER.dat
```

No es sobreescriu mai informació.


---
