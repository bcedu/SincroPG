# SincroPG

Formada per dos aplicacions: Client i Servidor.

- [CliPG](https://github.com/bcedu/SincroPG/tree/master/CliPG): Client, amb interficie gràfica.
- [SerPG](https://github.com/bcedu/SincroPG/tree/master/SerPG):: Servidor, per linia de comandes.

---

# Guia d’instal·lació

## CliPG


### Debian (Ubuntu i derivats)

Hi ha dos mètodes d’instal·lació:
1. GitHub Releases
2. PPA (APT repository)


#### 1. Instal·lació des de GitHub Releases (.deb)

Descarrega el paquet `.deb` des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Instal·lació:

```bash
sudo apt install ./clipg_<versió>_amd64.deb
```

#### 2. Instal·lació des de PPA (APT repository)

Afegir la clau i el repositori:

```bash
curl -fsSL https://ppa.bcclean.pw/bcedu.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/bcedu.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/bcedu.gpg] https://ppa.bcclean.pw stable main" | sudo tee /etc/apt/sources.list.d/bcedu.list
```

Actualitzar i instal·lar:
```bash
sudo apt update
sudo apt install clipg
```

### Fedora (openSUSE , Red Hat i derivats)

#### 1. Instal·lació des de GitHub Releases (.rpm)

Descarrega el paquet `.rpm` des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Instal·lació:
```bash
sudo dnf install clipg_<versió>.rpm
```

### Arch Linux (Manjaro , EndeavourOS i derivats)

#### 1. Instal·lació des de GitHub Releases (.pkg.tar.zst)

Descarrega el paquet `.pkg.tar.zst` des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Instal·lació:
```bash
sudo pacman install -U ./clipg_<versió>.pkg.tar.zst
```

### Linux (altres)

Si la teva distribució Linux no és compatible amb cap dels paquets disponibles a les releases (.deb, .rpm o .pkg.tar.zst), pots utilitzar el binari precompilat.

Descarrega el fitxer corresponent des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Execució:
```bash
chmod +x CliPG
./CliPG
```

> ℹ️ En aquest cas l’aplicació no s’instal·la al sistema. Simplement s’executa com a binari autònom.
> Si vols utilitzar-la com una aplicació instal·lada (icona, menú del sistema, etc.), hauràs d’utilitzar un dels paquets suportats (.deb, .rpm o .pkg.tar.zst) o crear una entrada manual al teu sistema.

### Windows

No disponible actualment.


### macOS

No disponible actualment ni mai.


## SerPG

### Debian (Ubuntu i derivats)

Hi ha dos mètodes d’instal·lació:
1. GitHub Releases
2. PPA (APT repository)


#### 1. Instal·lació des de GitHub Releases (.deb)

Descarrega el paquet `.deb` des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Instal·lació:

```bash
sudo apt install ./serpg_<versió>_amd64.deb
```

#### 2. Instal·lació des de PPA (APT repository)

Afegir la clau i el repositori:

```bash
curl -fsSL https://ppa.bcclean.pw/bcedu.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/bcedu.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/bcedu.gpg] https://ppa.bcclean.pw stable main" | sudo tee /etc/apt/sources.list.d/bcedu.list
```

Actualitzar i instal·lar:
```bash
sudo apt update
sudo apt install serpg
```


### Linux (altres)

Si la teva distribució Linux no és compatible amb cap dels paquets disponibles a les releases (.deb), pots utilitzar el binari precompilat.

Descarrega el fitxer corresponent des de la pàgina de releases:

https://github.com/bcedu/SincroPG/releases

Execució:
```bash
chmod +x SerPG
./SerPG
```

> ℹ️ En aquest cas l’aplicació no s’instal·la al sistema. Simplement s’executa com a binari autònom.
> Si vols utilitzar-la com una aplicació instal·lada (icona, menú del sistema, etc.), hauràs d’utilitzar un dels paquets suportats (.deb, .rpm o .pkg.tar.zst) o crear una entrada manual al teu sistema.


### Windows

No disponible actualment.


## macOS

No disponible actualment ni mai.
