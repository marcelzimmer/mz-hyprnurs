# MZ-HyprNurs - Entwicklerdokumentation

**Autor:** Marcel Zimmer<br>
**Web:** [www.marcelzimmer.de](https://www.marcelzimmer.de)<br>
**X:** [@marcelzimmer](https://x.com/marcelzimmer)<br>
**GitHub:** [@marcelzimmer](https://github.com/marcelzimmer)<br>
**Version:** 1.0.0<br>
**Sprache:** Rust<br>
**App-Sprache:** Deutsch<br>
**Plattform:** Primär für **Omarchy Linux** (eine Arch-Linux-Distribution) mit **Hyprland** entwickelt und optimiert - läuft selbstverständlich auch unter Windows, macOS und jedem anderen Linux<br>
**Lizenz:** MIT

---

## Wichtige Hinweise

### Zweck und Grenzen der App

MZ-HyprNurs ist ausschließlich ein **Hilfsmittel für die mündliche Übergabe beim Schichtwechsel**. Alle eingetragenen Informationen sind **persönliche Kurznotizen der Pflegekraft**, keine offiziellen medizinischen Dokumente.

Die App ersetzt **nicht** das klinische Informationssystem des Krankenhauses und will das auch ausdrücklich nicht. Sie ist kein KIS, kein Dokumentationssystem und keine Medikationsverwaltung. Verbindliche und vollständige Patientendaten - insbesondere Diagnosen, Medikamentenpläne und Pflegedokumentation - gehören ausschließlich in das dafür vorgesehene offizielle System der Einrichtung und müssen dort korrekt und vollständig gepflegt werden. Was in MZ-HyprNurs steht, ist das Wichtigste auf einen Blick - für eine schnelle, strukturierte Übergabe am Stationstresen. Nicht mehr, nicht weniger. Es gibt daher **bewusst keine Schnittstellen** zu anderen Systemen. Keine API, kein Datenbankexport, keine Integration. Das hier sind Notizen. Keine Doku.

### MIT-Lizenz - Nutzung, Rechte und Pflichten

MZ-HyprNurs steht unter der **MIT-Lizenz**. Der vollständige Lizenztext befindet sich in der Datei `LICENSE`.

**Was die MIT-Lizenz erlaubt:**

- **Nutzung** - Die Software darf frei genutzt werden, auch **kommerziell**.
- **Modifikation** - Der Quellcode darf verändert und angepasst werden.
- **Weitergabe** - Die Software darf weitergegeben und weiterverteilt werden, auch in veränderter Form.

**Was ausdrücklich ausgeschlossen ist:**

- **Haftung ist ausgeschlossen.** Der Autor haftet nicht für Schäden, Datenverlust, Fehlfunktionen oder sonstige Folgen, die durch die Nutzung dieser Software entstehen - weder direkt noch indirekt.
- **Keine Gewährleistung.** Die Software wird **„wie sie ist"** bereitgestellt, ohne jegliche Garantie auf Funktionsfähigkeit, Eignung für einen bestimmten Zweck oder Fehlerfreiheit.
- **Keine Support-Pflicht.** Es besteht keinerlei Verpflichtung, Fehler zu beheben, Fragen zu beantworten, Updates bereitzustellen oder irgendeine Form von Wartung oder Support zu leisten.

**Pflichten bei der Nutzung:**

- **Eigenverantwortliche Code-Prüfung:** Wer diese Software in einem produktiven Umfeld einsetzt - insbesondere in sicherheitskritischen oder medizinischen Bereichen - ist **selbst dafür verantwortlich**, den Quellcode zu lesen, zu verstehen, zu prüfen und ggf. anzupassen. Eine Nutzung ohne vorherige Prüfung erfolgt auf eigenes Risiko.

### Genehmigungspflicht vor dem Einsatz

Die App sollte **nicht ohne Genehmigung** auf Krankenhausrechnern installiert werden. Bevor MZ-HyprNurs auf einer Station eingesetzt wird, sollten folgende Stellen einbezogen werden:

- **Geschäftsführung** - strategische und haftungsrechtliche Freigabe
- **Datenschutzbeauftragter** - Prüfung auf DSGVO-Konformität, insbesondere beim Speichern von Patientendaten auf Netzlaufwerken
- **Pflegeleitung / Stationsleitung** - fachliche Freigabe und Einweisung
- **IT-Abteilung** - Sicherheitsprüfung, Netzwerkfreigabe, Betrieb

Ohne diese Freigaben entsteht **Schatten-IT**, die rechtliche, haftungsrechtliche und datenschutzrechtliche Risiken birgt.

### ODF- und PDF-Export - offene Standards

MZ-HyprNurs exportiert Dokumente in **PDF** und **ODF** [Open Document Format]. Das ist kein Zufall: Der **IT-Planungsrat** hat am 19. März 2026 im Rahmen des Deutschland-Stacks verbindlich festgelegt, dass die gesamte öffentliche Verwaltung in Deutschland [Bund, Länder, Kommunen] beim digitalen Dokumentenaustausch ausschließlich **ODF** [für bearbeitbare Dokumente] und **PDF** [für nicht veränderliche Dokumente] verwenden darf. Microsoft OOXML ist nicht vorgesehen. Die Umsetzungsfrist ist 2028. MZ-HyprNurs folgt diesem Standard bereits heute.

### Keine Passwortverwaltung in der App

MZ-HyprNurs enthält **keine eigene Benutzerverwaltung und keine Passwörter**. Das ist eine bewusste Entscheidung: Die App läuft ausschließlich auf Rechnern, die durch das Betriebssystem und die IT-Infrastruktur des Krankenhauses bereits passwortgeschützt sind. Die Zugangskontrolle liegt damit dort, wo sie hingehört - auf Betriebssystem- und Netzwerkebene, verwaltet von der IT-Abteilung.

### Empfehlung: App lokal, Daten im Netzwerk

Die **App selbst** [die Binary bzw. das `.app`-Bundle] sollte auf einem **lokalen Laufwerk** des jeweiligen Rechners installiert werden. Lokale Ausführung ist schneller, funktioniert auch ohne Netzwerkverbindung und erzeugt keine unnötige Last auf Netzlaufwerken.

Die **Datendateien** [`.md`, `.backup`, PDF- und ODT-Exporte] hingegen empfiehlt es sich, auf einem **zentralen Netzwerklaufwerk** abzulegen. Das ermöglicht:

- **Kollaboratives Arbeiten:** Mehrere Pflegekräfte können dieselbe Datei öffnen und die Übergabe gemeinsam nutzen.
- **Zentrale Datensicherung:** Die IT-Abteilung sichert das Netzwerklaufwerk im Rahmen ihres regulären Backup-Konzepts.
- **Zugriff von mehreren Rechnern:** Die Daten der Station sind von jedem Stationsrechner aus erreichbar.

Die konkrete Einrichtung des Netzwerklaufwerks und der Zugriffsberechtigungen sollte **mit der IT-Abteilung abgesprochen** werden.

### Datensicherung durch die IT-Abteilung erforderlich

Die eingebaute `.backup`-Datei [siehe unten] ist eine **Soforthilfe auf der Station** - sie erlaubt es, einen versehentlich überschriebenen Stand schnell wiederherzustellen, ohne die IT anrufen zu müssen. Sie ist **kein Ersatz für eine professionelle Datensicherung**.

Für eine verlässliche, langfristige Sicherung muss die IT-Abteilung den Speicherpfad der `.md`-Datei in ihr reguläres **Backup-Konzept [z. B. Bandsicherung]** aufnehmen. Die Abstimmung des Speicherorts mit der IT liegt beim Nutzer bzw. der verantwortlichen Stelle.

### Automatische Backup-Datei bei jedem Speichern

Bei **jedem Speichervorgang** - ob manuell oder durch das Auto-Speichern - erstellt MZ-HyprNurs automatisch eine Sicherungskopie der vorherigen Version. Die Backup-Datei hat denselben Namen wie die `.md`-Arbeitsdatei, jedoch mit der Endung `.backup` [z. B. `MZ-HYPRNURS_Station5C.backup`]. Sie liegt immer im selben Verzeichnis wie die Arbeitsdatei.

Die `.backup`-Datei wird bei jedem Speichern überschrieben - sie enthält immer genau den Stand **vor dem letzten Speichern**. Das genügt, um einen einzelnen Fehler auf der Station schnell zu korrigieren. Für mehr als einen Schritt zurück ist die IT-seitige Sicherung zuständig.

### Exportdateien: Datum und Uhrzeit im Dateinamen

Jede exportierte PDF- und ODT-Datei trägt automatisch **Datum und Uhrzeit im Dateinamen** [z. B. `MZ-HYPRNURS_Station5C_2026-03-21_14-30.pdf`]. Das ermöglicht es, alle Exporte in einem einzigen Ordner zu sammeln, ohne dass Dateien sich gegenseitig überschreiben. Wer Exportdateien aufbewahren möchte oder muss, legt sich einmalig einen Archivordner an und speichert alle Exporte dort hinein.

**Hinweis zur DSGVO:** Personenbezogene Daten müssen auf Anfrage schnell und vollständig auffindbar sein. Es empfiehlt sich daher, die Speicherorte - sowohl die `.md`-Arbeitsdatei als auch den Exportordner für PDF/ODT - von Beginn an zu dokumentieren und dem Datenschutzbeauftragten mitzuteilen. So lassen sich alle Stellen, an denen Patientendaten gespeichert werden, jederzeit benennen.

---

## Anleitung für Anwender

### Was ist MZ-HyprNurs?

MZ-HyprNurs ersetzt den Word-Zettel an der Bettenstation. Statt einer Tabelle in einem Textprogramm sehen Sie alle Betten Ihrer Station auf einen Blick - übersichtlich, schnell und ohne Scrollen durch lange Dokumente.

Die App läuft lokal auf dem Rechner, braucht keine Internetverbindung und speichert alles in einer einzigen lesbaren Datei. Die Daten können auf einem Netzwerklaufwerk liegen, sodass alle Kolleginnen und Kollegen an der Station dieselbe Datei nutzen.

### Erste Schritte

**App starten** - Starten Sie MZ-HyprNurs wie jedes andere Programm. Beim ersten Start sehen Sie eine leere Bettenübersicht mit 12 Zimmern.

**Stationsname festlegen** - Klicken Sie oben links auf das Feld **„STATIONSNAME"**. Dort können Sie den Namen Ihrer Station eingeben, zum Beispiel `Station 5C`. Dieser Name erscheint in allen Exporten.

**Neue Datei anlegen** - Beim allerersten Start ist noch keine Datei geöffnet. Wählen Sie **Speichern** [`Strg+S`] und legen Sie die Datei an einem geeigneten Ort ab - idealerweise auf dem Netzwerklaufwerk Ihrer Station. Der Dateiname wird automatisch aus dem Stationsnamen gebildet, zum Beispiel `MZ-HYPRNURS_Station5C.md`. Ab jetzt speichert die App automatisch alle 10 Minuten, sofern Änderungen vorhanden sind.

**Zimmeranzahl anpassen** - Die App startet mit einer Standardanzahl an Zimmern. Um die Anzahl an Ihre Station anzupassen, öffnen Sie die **Einstellungen** [`Strg+Shift+E`] und stellen dort die gewünschte Zimmeranzahl ein [1 bis 50 Zimmer].

> **Achtung:** Wenn Sie Zimmer entfernen, gehen alle Patientendaten in diesen Zimmern unwiderruflich verloren. Die App weist Sie im Dialog darauf hin.

### Die Bettenübersicht

Nach dem Start sehen Sie alle Zimmer Ihrer Station nebeneinander. Jedes Zimmer zeigt die darin enthaltenen Betten als Karte.

```
╭─────────────────────────────────────────╮
│ ┌────┬───┐  Hr. Hamster [*73, m]         │
│ │101 │ A │  Isolation                    │
│ └────┴───┘                               │
│  HDIA  │ Pneumonie links                 │
│  NDIA  │ Diabetes Typ 2                  │
│  NURS  │ mit Hilfe mobil, Sturzrisiko    │
│  INFO  │ Labor 14 Uhr                    │
│  TODO  │ Röntgen-Termin klären           │
╰─────────────────────────────────────────╯
```

| Bereich       | Inhalt                                              |
|---------------|-----------------------------------------------------|
| Kopfzeile     | Zimmernummer, Bettnummer, Name, Alter, Geschlecht   |
| Besonderheiten| Isolation, Allergien oder sonstige Kurzhinweise     |
| HDIA          | Hauptdiagnose [oder PDIA im Psychiatrie-Modus]      |
| NDIA          | Nebendiagnose [oder SDIA im Psychiatrie-Modus]      |
| NURS          | Pflegerische Hinweise                               |
| INFO          | Sonstige Informationen                              |
| TODO          | Offene Aufgaben für die Folgeschicht                |

Ein Bett ohne Patient wird als grauer Block dargestellt. Klicken Sie auf diesen Block, um das Bett zu belegen.

**Navigation per Tastatur** - Mit den Pfeiltasten `←` `→` `↑` `↓` können Sie zwischen den Karten wechseln. `Enter` öffnet das Detailfenster des markierten Bettes.

### Patientendaten eingeben

Klicken Sie auf eine beliebige Karte. Es öffnet sich das **Detailfenster** für dieses Bett.

| Feld                        | Inhalt                                                         |
|-----------------------------|----------------------------------------------------------------|
| Anrede                      | `Hr.` / `Fr.` / `-` [rechtliches Geschlecht / Verwaltung]    |
| Biologisches Geschlecht     | `m` / `w` / `d` [medizinisch relevant für Labor, Dosierung]  |
| Nachname / Vorname          | Patientenname                                                  |
| Geburtsdatum                | Format `TT.MM.JJJJ` - das Alter wird automatisch berechnet   |
| Besonderheiten              | Einzeiliger Freitext [Isolation, Allergie, Sturzrisiko …]     |
| HDIA                        | Hauptdiagnose [max 3 Zeilen]                                   |
| NDIA                        | Nebendiagnose [max 3 Zeilen]                                   |
| NURS                        | Pflegerische Informationen [max 3 Zeilen]                      |
| INFO                        | Sonstige Informationen [max 3 Zeilen]                          |
| TODO                        | Offene Aufgaben für die Folgeschicht [max 3 Zeilen]            |

Die sechs Felder [Patient, HDIA, NDIA, NURS, INFO, TODO] werden **einzeln** angezeigt - mit `←`/`→` oder den Navigations-Punkten am unteren Rand wechselt man zwischen ihnen. Wenn kein Textfeld fokussiert ist, funktionieren die Pfeiltasten direkt; tippen Sie in ein Feld, navigieren die Pfeiltasten wie gewohnt innerhalb des Textes.

**Detailfenster schließen** - Klicken Sie auf **×** oder drücken Sie `ESC`. Die Änderungen sind sofort in der Übersicht sichtbar.

**Patientendaten löschen** - Über das Symbol **🗑** im Kopfbereich des Detailfensters können Sie alle Daten dieses Bettes löschen. Die App fragt vorher zur Sicherheit nach.

**Zimmernummer und Bettbezeichnung ändern** - Im oberen Bereich des Detailfensters können Zimmernummer und Bettnummer direkt bearbeitet werden. Die App prüft dabei auf Duplikate - ist die Kombination bereits vergeben, erscheint ein roter Warnbalken und das Fenster kann erst geschlossen werden, wenn der Konflikt behoben ist.

### Speichern und Öffnen

| Aktion               | Tastenkombination | Beschreibung                                                                  |
|----------------------|-------------------|-------------------------------------------------------------------------------|
| Speichern            | `Strg+S`          | Speichert die aktuelle Datei. Ohne Pfad öffnet sich ein Dateidialog.          |
| Speichern unter      | `Strg+Shift+S`    | Öffnet immer den Dateidialog - nützlich für Kopien.                           |
| Datei öffnen         | `Strg+O`          | Lädt eine vorhandene `.md`-Datei.                                             |
| Neue Station         | `Strg+N`          | Löscht alle Daten und startet eine leere Station [mit Sicherheitsabfrage].    |

**Auto-Speichern** - Die App speichert automatisch spätestens nach **10 Minuten**, sofern Änderungen vorhanden sind und ein Speicherpfad gesetzt ist.

**Automatische Sicherungskopie** - Bei jedem Speichern wird eine Sicherungskopie des vorherigen Stands angelegt [z. B. `MZ-HYPRNURS_Station5C.backup`]. Sie liegt im selben Ordner und enthält immer genau den Stand vor dem letzten Speichern - als schnelle Soforthilfe auf der Station, kein Ersatz für die IT-seitige Datensicherung.

### Übergabe drucken

**PDF-Export** [`Strg+P`] - Erzeugt ein druckfertiges DIN-A4-Dokument mit allen Betten der Station. Kompaktes Layout mit 7-pt-Schrift, ideal für die schnelle Übersicht am Klemmbrett. Der Dateiname enthält automatisch Datum und Uhrzeit, zum Beispiel: `MZ-HYPRNURS_Station5C_2026-03-25_06-30.pdf`

**ODT-Export** [`Strg+L`] - Erzeugt ein bearbeitbares Dokument im Open-Document-Format [ODF] mit 11-pt-Schrift. Ideal, wenn die Schrift im PDF zu klein ist: Öffnen Sie die Datei in LibreOffice Writer und passen Sie Schriftgröße, Ränder oder Farben nach Wunsch an. Das ODF-Format entspricht bereits heute dem ab 2028 verbindlichen Standard für die deutsche öffentliche Verwaltung.

> **Tipp:** Da jeder Export eine neue Datei erzeugt, empfiehlt es sich, einmalig einen Archivordner anzulegen und alle Druckexporte dort zu sammeln.

### Mehrere Nutzer gleichzeitig

Liegt die `.md`-Datei auf einem gemeinsamen Netzwerklaufwerk, können mehrere Pflegekräfte gleichzeitig damit arbeiten. Die App erkennt dabei, wenn jemand anderes zwischenzeitlich Änderungen gespeichert hat.

Wenn beim Speichern festgestellt wird, dass die Datei verändert wurde, erscheint für jedes betroffene Bett ein **Konflikt-Popup**. Es zeigt links **Meine Änderungen** und rechts **Andere Änderungen**. Sie wählen pro Bett, welche Version behalten werden soll.

### Tastenkombinationen

Vollständige Übersicht: siehe [Tastenkombinationen](#tastenkombinationen) in der Entwicklerdokumentation.

### Häufige Fragen

**Das Detailfenster lässt sich nicht schließen.** - Es gibt einen Eingabefehler: Entweder ist die Zimmernummer leer, die Bettbezeichnung leer, oder die Zimmer+Bett-Kombination existiert bereits. Der rote Warnbalken erklärt, was zu korrigieren ist.

**Die App zeigt einen Konflikt-Dialog beim Speichern.** - Eine Kollegin oder ein Kollege hat dieselbe Datei zwischenzeitlich geändert. Wählen Sie pro betroffenem Bett, welche Version behalten werden soll.

**Der Stationsname erscheint nicht im Dateinamen.** - Der Dateiname wird beim ersten Speichern festgelegt. Nutzen Sie **Speichern unter** [`Strg+Shift+S`], um die Datei mit dem neuen Namen neu anzulegen.

**Wo liegt die Backup-Datei?** - Im selben Ordner wie die Arbeitsdatei, mit der Endung `.backup` statt `.md`. Sie enthält den Stand vor dem letzten Speichern und wird bei jedem Speichern überschrieben. Für ältere Stände ist die IT-seitige Datensicherung zuständig.

**Die App speichert nicht automatisch.** - Auto-Speichern funktioniert nur, wenn bereits ein Speicherpfad gesetzt ist. Bitte einmalig `Strg+S` drücken und einen Ort wählen.

**Kann ich MZ-HyprNurs auf mehreren Rechnern gleichzeitig nutzen?** - Ja - legen Sie die `.md`-Datei auf einem gemeinsamen Netzwerklaufwerk ab. Die App erkennt beim Speichern, wenn die Datei zwischenzeitlich verändert wurde, und zeigt einen Konflikt-Dialog.

---

## Inhaltsverzeichnis [Entwicklerdokumentation]

1. [Überblick](#überblick)
2. [Abhängigkeiten](#abhängigkeiten)
3. [Projektstruktur](#projektstruktur)
4. [Datenmodell](#datenmodell)
5. [Architektur und Programmfluss](#architektur-und-programmfluss)
6. [UI-Schicht](#ui-schicht)
7. [HyprGross-Modus](#hyprgross-modus)
8. [Screensaver](#screensaver)
9. [Schriftarten-Laden](#schriftarten-laden)
10. [Dateiformat .md](#dateiformat-md)
11. [PDF- und ODT-Export](#pdf--und-odt-export)
12. [Theme-System](#theme-system)
13. [Psychiatrie-Modus](#psychiatrie-modus)
14. [Datei-Dialoge und Thread-Kommunikation](#datei-dialoge-und-thread-kommunikation)
15. [Einstellungen-Dialog](#einstellungen-dialog)
16. [Validierung im Detailfenster](#validierung-im-detailfenster)
17. [Tastenkombinationen](#tastenkombinationen-entwickler)
18. [Build und Installation](#build-und-installation)

---

## Überblick

### Zur Namensgebung

Der Name **MZ-HyprNurs** folgt einer Konvention, die in der Community rund um **Omarchy Linux** und den Fenstermanager **Hyprland** verbreitet ist. Hyprland läuft auf Wayland und hat eine eigene Programmiererkultur: Namen bestehen üblicherweise aus vier Buchstaben - und der Präfix `Hypr` ist dabei keine Falschschreibung von *Hyper*, sondern ein bewusstes Stilmittel dieser Welt.

Dasselbe gilt für `Nurs` - kein Tippfehler, sondern die konsequente Weiterführung dieser Namenslogik. `MZ` steht für **Marcel Zimmer**, den Entwickler der App.

**MZ-HyprNurs** ist also wörtlich zu lesen als: eine App von Marcel Zimmer, entstanden in der Hyprland-Welt, für die Pflege [*Nursing*].

Dass die wenigsten Krankenhäuser und Kliniken heute **Linux** einsetzen, ist ihm bewusst - aber er ist überzeugt, dass das noch kommt. In der Zwischenzeit läuft MZ-HyprNurs selbstverständlich auch unter **Windows und macOS**. Kein Kompromiss, keine abgespeckte Version. Genau dafür wurde **Rust** als Programmiersprache gewählt: plattformübergreifend, schnell, ohne externe Laufzeitumgebung - eine einzige Binary, die überall funktioniert.

### App-Beschreibung

MZ-HyprNurs ist eine Desktop-App für **Linux, Windows und macOS** für die pflegerische Übergabe im Krankenhaus. Sie ersetzt den bisherigen Word-Workflow durch eine strukturierte Desktop-Anwendung, die alle Patientendaten einer Station übersichtlich auf einen Blick zeigt.

Die Oberfläche wird mit **egui/eframe** [Rust-GUI-Framework] gerendert. Eine Station umfasst eine konfigurierbare Anzahl Zimmer [1-50] mit je 1-2 Betten. Jedes Bett wird als Karte in der Übersicht dargestellt. Die Bearbeitung erfolgt ausschließlich im Detailfenster. Als Exportformate stehen **PDF** [druckfertig, DIN A4] und **ODT** [Open Document Text] zur Verfügung.

Die gesamte Anwendungslogik befindet sich in einer einzigen Quelldatei: `src/main.rs`.

### Philosophie: Klein, schnell, sparsam

MZ-HyprNurs ist bewusst so gebaut, dass sie **so wenig Ressourcen wie möglich verbraucht** - wenig Speicher, wenig CPU, kein unnötiger Netzwerkverkehr. Die App erzeugt keinen Hintergrunddatenverkehr, spricht keine externen Dienste an und lädt nichts nach. Was beim Start da ist, bleibt da.

Das ist keine technische Zufälligkeit, sondern eine Haltung. Der Entwickler hat das Programmieren in den **1980er Jahren** gelernt - in einer Zeit, in der Arbeitsspeicher in Kilobyte gemessen wurde, jedes Byte eine Entscheidung war und Effizienz kein Buzzword, sondern eine schlichte Notwendigkeit. Diese Schule hat geprägt: Code, der das tut was er soll, ohne Aufwand der nicht gebraucht wird.

Das Ergebnis ist eine App, die sich auch auf älterer Hardware und auf schwachen Stationsrechnern flüssig anfühlt - und die in Umgebungen mit eingeschränkter Netzwerkbandbreite oder strikten Firewall-Regeln problemlos funktioniert.

---

## Abhängigkeiten

| Crate         | Version | Verwendungszweck                                          |
|---------------|---------|-----------------------------------------------------------|
| `eframe`      | 0.29    | Anwendungsrahmen und Ereignisschleife [egui-Backend]      |
| `egui`        | -       | Immediate-Mode-GUI [Teil von eframe]                      |
| `rfd`         | 0.15    | Datei-Öffnen/Speichern-Dialoge [plattformnativ]           |
| `printpdf`    | 0.3     | PDF-Dokument-Generierung                                  |
| `image`       | 0.25    | PNG-Icon einlesen                                         |
| `zip`         | 2       | ODT-Export [ZIP-Container]                                |
| `chrono`      | 0.4     | Lokale Datum- und Uhrzeitermittlung [Zeitzone-korrekt]    |
| `winresource` | 0.1     | Windows: Icon in .exe einbetten [build-dependency]        |

---

## Projektstruktur

```
mz-hyprnurs/
├── src/
│   └── main.rs              - gesamte Anwendungslogik [Datenmodell, UI, Export]
├── assets/
│   ├── icon.png             - App-Icon 256×256 [Linux, in Binary eingebettet]
│   ├── icon_macos.png       - App-Icon 512×512 [macOS, Retina]
│   ├── icon.ico             - App-Icon für Windows-Binary [16-256px gebündelt]
│   ├── icon.icns            - App-Icon für macOS-Bundle [16-1024px]
│   ├── icon_16.png          - hicolor-Icon 16×16 [Linux Desktop]
│   ├── icon_24.png          - hicolor-Icon 24×24
│   ├── icon_32.png          - hicolor-Icon 32×32
│   ├── icon_48.png          - hicolor-Icon 48×48
│   ├── icon_64.png          - hicolor-Icon 64×64
│   ├── icon_128.png         - hicolor-Icon 128×128
│   ├── icon_256.png         - hicolor-Icon 256×256
│   ├── icon_512.png         - hicolor-Icon 512×512
│   ├── hintergrund.png      - Screensaver-Hintergrundbild
│   └── Info.plist           - macOS Bundle-Metadaten
├── .github/workflows/
│   └── release.yml          - CI/CD: Release-Builds für Linux, macOS, Windows
├── build.rs                 - bettet icon.ico unter Windows in die .exe ein
├── install.sh               - Installations-Skript [Linux/Omarchy]
├── PKGBUILD                 - AUR-Paketdefinition [Arch Linux]
├── Cargo.toml               - Paketdefinition und Abhängigkeiten
├── Cargo.lock               - reproduzierbare Builds
├── .gitignore
├── LICENSE                  - MIT-Lizenz
└── README.md                - diese Datei
```

---

## Datenmodell

### `Patient` [Patientendaten]

```rust
struct Patient {
    nachname:        String,   // Nachname
    vorname:         String,   // Vorname
    geburtsdatum:    String,   // Format: TT.MM.JJJJ
    anrede:          String,   // Rechtliches Geschlecht: "Hr." / "Fr." / "-"
    bio_geschlecht:  String,   // Biologisches Geschlecht: "m" / "w" / "d"
    besonderheiten:  String,   // Kurzer Freitext [Isolation, Allergien …]
    hdia:            String,   // Hauptdiagnose [3 Zeilen]
    ndia:            String,   // Nebendiagnose [3 Zeilen]
    info:            String,   // Informationen [3 Zeilen]
    pflege:          String,   // Pflegerische Informationen [3 Zeilen]
    todo:            String,   // Offene Aufgaben für die Folgeschicht [3 Zeilen]
}
```

Das Alter wird zur Laufzeit aus `geburtsdatum` berechnet [Funktion `alter_aus_geburtsdatum`].

**Namensanzeige je nach Kontext:**
- Bettenübersicht und PDF: `Hr. Löwe [*73, m]` - ohne Vorname, wenn Anrede gesetzt
- Detailfenster-Header und HyprGross: `Hr. Löwe, Leon [*73, m]` - immer mit Vorname
- Kein Anrede gesetzt: `Löwe, Leon [*73, m]` - klassisches Format

Die zwei Felder `anrede` und `bio_geschlecht` trennen bewusst **rechtliches** [Anrede, Verwaltung] von **biologischem** Geschlecht [medizinisch relevant für Laborwerte und Dosierungen].

### `Bett`

```rust
struct Bett {
    buchstabe: String,         // „A" oder „B"
    patient:   Option<Patient> // None = Bett leer / gesperrt
}
```

Ein `patient: None` wird in der Übersicht als gesperrte [graue] Karte dargestellt.

### `Zimmer`

```rust
struct Zimmer {
    nummer: String,    // frei eingegebene Zimmernummer [z. B. „101"]
    betten: Vec<Bett>, // 1 oder 2 Betten
}
```

### `Station` [Hauptzustand]

```rust
struct Station {
    zimmer:      Vec<Zimmer>, // dynamisch konfigurierbar [1-50 Zimmer]
    dienst_info: String,      // Freitext für stationsweite Übergabeinformation
}
```

### `MzHyprNursApp` [App-Struct]

Zentrale Struct, die `eframe::App` implementiert und den vollständigen Anwendungszustand hält.

| Feld                          | Typ                                | Bedeutung                                          |
|-------------------------------|------------------------------------|----------------------------------------------------|
| `station`                     | `Station`                          | Aktuelle Stationsdaten                             |
| `station_name`                | `String`                           | Name der Station [Titelzeile]                      |
| `station_hyprinfo`            | `String`                           | Stationsweite Übergabenotiz                        |
| `bearbeitung`                 | `Option<[usize, usize]>`           | Geöffnetes Detailfenster [Zimmer-/Bett-Index]      |
| `bearbeitungsfeld`            | `usize`                            | Aktuell aktives Eingabefeld im Detailfenster [0-5] |
| `ausgewaehlte_karte`          | `Option<[usize, usize]>`           | Tastaturauswahl in der Bettenübersicht             |
| `speicher_pfad`               | `Option<PathBuf>`                  | Zuletzt genutzter Speicherpfad                     |
| `letzte_aenderung_am`         | `Option<String>`                   | Zeitstempel der letzten Speicherung [lokale Zeit]  |
| `letzte_aenderung_von`        | `Option<String>`                   | Benutzer @ Hostname der letzten Speicherung        |
| `geaendert`                   | `bool`                             | Ungespeicherte lokale Änderungen vorhanden         |
| `theme`                       | `Theme`                            | Aktives Farbschema                                 |
| `hat_omarchy`                 | `bool`                             | Omarchy-Konfigurationsdatei gefunden               |
| `hyprgross_aktiv`             | `bool`                             | HyprGross-Vollbildmodus aktiv                      |
| `hyprgross_ansicht`           | `HyprGrossAnsicht`                 | Aktiver Tab im HyprGross-Modus                     |
| `matrix_modus`                | `bool`                             | Matrix-Regen-Screensaver ein [true] / aus [false]  |
| `einstellungen_offen`         | `bool`                             | Einstellungen-Dialog geöffnet                      |
| `einstellungen_zimmer_anzahl` | `usize`                            | Temporäre Zimmeranzahl im Einstellungen-Dialog     |
| `dialog_rx`                   | `Option<Receiver<DialogErgebnis>>` | Laufender Dateidialog [mpsc]                       |

### `HyprGrossAnsicht`

```rust
enum HyprGrossAnsicht {
    Feld,       // Alle Felder eines Bettes in Vollbild-Ansicht
    HyprInfo,   // Großansicht des INFO-Feldes
    DienstInfo, // Großansicht der stationsweiten Dienstinformation
}
```

### `DialogErgebnis`

Kommunikationstyp zwischen Datei-Dialog-Threads und dem Haupt-Thread:

```rust
enum DialogErgebnis {
    Speichern(PathBuf),     // gewählter Speicherpfad
    Laden(PathBuf, String), // Pfad + Dateiinhalt
    PdfExport(PathBuf),     // gewählter PDF-Speicherpfad
    OdtExport(PathBuf),     // gewählter ODT-Speicherpfad
}
```

---

## Architektur und Programmfluss

MZ-HyprNurs folgt dem **Immediate-Mode-GUI-Muster** von egui:

```
┌────────────────────────────────────────┐
│  eframe-Ereignisschleife               │
│  [läuft ~60 Hz oder bei Ereignis]      │
│                                        │
│  MzHyprNursApp::update()                 │
│  ┌──────────────────────────────────┐  │
│  │ 1. Tastenkombinationen prüfen    │  │
│  │ 2. Dialog-Ergebnisse verarbeiten │  │
│  │ 3. Auto-Speichern prüfen         │  │
│  │ 4. Konflikt-Popups anzeigen      │  │
│  │ 5. Theme anwenden                │  │
│  │ 6. UI rendern [deklarativ]       │  │
│  │    a] Bettenübersicht            │  │
│  │    b] Detailfenster [optional]   │  │
│  │    c] HyprGross [optional]       │  │
│  │    d] Screensaver [optional]     │  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘
         │ Nutzeraktion [Klick/Eingabe]
         ▼
   Zustandsänderung in MzHyprNursApp
         │
         ▼
   nächster Frame → neu rendern
```

### Auto-Speichern

Änderungen werden über einen Vergleich mit `inhalt_beim_speichern` erkannt. Die App prüft alle **30 Sekunden**, ob sich der Inhalt geändert hat. Liegt eine Änderung vor, wird spätestens nach **10 Minuten** automatisch gespeichert [sofern ein `speicher_pfad` gesetzt ist]. Dabei werden auch `letzte_aenderung_am` und `letzte_aenderung_von` aktualisiert und eine `.backup`-Datei angelegt. Auch beim Auto-Speichern wird vorher auf Konflikte geprüft.

### Kollaboratives Arbeiten - Konfliktlösung beim Speichern

Ein automatisches Nachladen der Datei im Hintergrund gibt es bewusst nicht - das würde laufende Eingaben unbemerkt überschreiben. Stattdessen prüft die App **beim Speichern** [manuell und Auto-Save], ob die Datei auf dem Datenträger seit dem letzten Laden verändert wurde.

Gibt es Unterschiede bei einzelnen Betten, erscheint für jedes betroffene Bett ein **Konflikt-Popup**. Es zeigt beide Versionen nebeneinander:

- Links: **„Meine Änderungen"** - was lokal im Speicher steht
- Rechts: **„Andere Änderungen"** - was aktuell auf dem Laufwerk liegt

Der Nutzer wählt pro Bett, welche Version behalten werden soll. Die Popups erscheinen nacheinander. Erst wenn alle Konflikte aufgelöst sind, wird tatsächlich gespeichert.

**Kein Konflikt** entsteht, wenn ein Bett lokal neu hinzugefügt wurde [noch nicht auf Disk vorhanden] oder wenn die Daten auf beiden Seiten identisch sind.

---

## UI-Schicht

### Aufbau der Oberfläche

```
┌────────────────────────────────────────────────────────────────────┐
│  MZ-HyprNurs  [Stationsname]    [Neu][Öffnen][Speichern][PDF][…]  │
├────────────────────────────────────────────────────────────────────┤
│  ScrollArea: Bettenübersicht                                        │
│                                                                    │
│  Zi. 101                        Zi. 102                            │
│  ╭────────────────────╮  ╭────────────────────╮  ╭─────────────╮  │
│  │ 101│A  Hamster, H  │  │ 101│B  Fuchs, F    │  │ 102│A  leer │  │
│  │    │ HDIA: …       │  │    │ HDIA: …       │  │    │        │  │
│  │    │ NDIA: …       │  │    │ NDIA: …       │  │    │        │  │
│  │    │ PFLEGE: …     │  │    │ PFLEGE: …     │  │    │        │  │
│  │    │ INFO: …       │  │    │ INFO: …       │  │    │        │  │
│  ╰────────────────────╯  ╰────────────────────╯  ╰─────────────╯  │
└────────────────────────────────────────────────────────────────────┘
```

### Bett-Karte

Jede Karte besteht aus einem farbig hinterlegten Kopfblock [Zimmer-/Bettnummer in Akzentfarbe] und einer Tabelle der Pflegefelder. Alle Karten haben gleiche Höhe [je 3 Zeilen pro Feld]. Leere und gesperrte Betten werden als ausgegraueter Block dargestellt.

```
╭─────────────────────────────────────────────────────╮
│ ┌────┬────┐  Hr. Hamster [*73, m]   Isolation       │  ← Kopfzeile
│ │101 │ A  │                                          │
│ └────┴────┘                                          │
│      │  HDIA   │ Pneumonie links                     │
│      │  NDIA   │ Diabetes Typ 2                      │
│      │  NURS   │ mit Hilfe mobil, Sturzrisiko        │
│      │  INFO   │ Labor 14 Uhr                        │
│      │  TODO   │ Röntgen-Termin klären               │
╰─────────────────────────────────────────────────────╯
         ↑ Klick → Detailfenster öffnet sich
```

### Detailfenster

Klick auf eine Karte öffnet ein separates `egui::Window` mit großen Eingabefeldern. Hier findet die gesamte Dateneingabe statt - das ist der einzige Ort mit editierbaren Feldern.

```
╔══════════════════════════════════════════════════════╗
║  Zi. 101 · Bett A  Hr. Hamster, Hans [*73, m]       ║
╠══════════════════════════════════════════════════════╣
║  ‹ Patient ›   · · · · · ·                          ║
║                                                      ║
║  ANREDE / RECHTLICHES GESCHLECHT  │ BIOLOGISCHES     ║
║  [Hr.] [Fr.] [-]                  │ [m]  [w]  [d]    ║
║  NACHNAME: [                    ]                    ║
║  VORNAME:  [                    ]                    ║
║  ALTER (tt.mm.jjjj): [          ]                    ║
╚══════════════════════════════════════════════════════╝
```


---

## HyprGross-Modus

Der HyprGross-Modus [`Strg+G`] zeigt ein einzelnes Feld eines belegten Bettes bildschirmfüllend in großer roter Schrift. Er ist für die Nutzung auf großen Bildschirmen oder Monitoren an der Bettenstation optimiert. Mit `←`/`→` wechselt man zwischen Feldern und Betten, mit `↑`/`↓` springt man zum nächsten/vorherigen Patienten.

```
┌──────────────────────────────────────────────────────────┐
│  Hauptdiagnose         Hr. Hamster, Hans [*73, m]  [Bild]│
│                                                          │
│                                                          │
│                 Pneumonie links                           │
│                                                          │
│                                                          │
│  Zimmer 101 | Bett A       Isolation     29.03.2026 @14:30│
└──────────────────────────────────────────────────────────┘
```

Oben rechts wird das Hintergrundbild [`assets/hintergrund.png`] klein eingeblendet [ca. 100 px Höhe].

Die Navigation beginnt bei HyprInfo und endet bei DienstInfo:

| Ansicht       | Inhalt                                                    |
|---------------|-----------------------------------------------------------|
| `HyprInfo`    | Stationsname und HyprInfo-Text groß                       |
| `Feld`        | Einzelnes Feld [Patient, HDIA, NDIA, NURS, INFO, TODO] bildschirmfüllend |
| `DienstInfo`  | Stationsweite Dienstinformation groß                      |

---

## Screensaver

Nach 60 Sekunden ohne Eingabe aktiviert sich automatisch der Screensaver. Er zeigt das Hintergrundbild [`assets/hintergrund.png`] zentriert auf schwarzem Hintergrund. Jede Mausbewegung, jeder Tastendruck oder Mausklick beendet den Screensaver sofort.

### Matrix-Regen-Modus [`Strg+M`]

Über `Strg+M` oder den Menüeintrag **„Matrix: einschalten / Matrix: ausschalten"** im Hamburger-Menü lässt sich ein optionaler Matrix-Regen-Effekt aktivieren. Im aktivierten Zustand fällt roter Zeichenregen [Ziffern und die Buchstaben `MZHYPRNURS` sowie Sonderzeichen] über das Hintergrundbild.

Der Zustand wird in der `.md`-Datei gespeichert [`**Matrix:** true/false`], sofern eine Datei geöffnet ist. Ohne geladene Datei gilt bei jedem Programmstart der Standard: Matrix aus.

---

## Schriftarten-Laden

egui benötigt für fetten Text eine separate Font-Family „Bold". Die Anwendung liest Systemschriften zur Laufzeit - es werden keine Schriften eingebettet.

**Windows:** Arial Bold, Verdana Bold, Calibri Bold [`C:\Windows\Fonts\`]

**macOS:** Arial Bold, Verdana Bold, Georgia Bold [`/System/Library/Fonts/Supplemental/`]

**Linux:** Liberation Sans Bold [Arch, Fedora, Debian, Ubuntu], DejaVu Sans Bold, Noto Sans Bold [Fallback]

Wird keine Schrift gefunden, verwendet egui seine eingebettete Fallback-Schrift [ohne fette Variante].

---

## Dateiformat .md

Die Stationsdaten werden als **Markdown-Datei** [`.md`] gespeichert. Das Format ist menschenlesbar, versionierbar und kann auf einem Netzwerklaufwerk für mehrere gleichzeitig arbeitende Nutzer abgelegt werden.

```markdown
# MZ-HyprNurs

**Station:** Station 5C
**HyprInfo:** Übergabe 06:00 - Nachtdienst ruhig
**Letzte Aenderung am:** 22.03.2026 @ 06:14:37 Uhr
**Letzte Aenderung von:** pflege01 @ stationsrechner
**Matrix:** true

## Dienstinfo

Bitte Blutdruckkontrolle Zi. 103 nicht vergessen.

## Zimmer 101

### Bett A

**Nachname:** Hamster
**Vorname:** Hans
**Geburtsdatum:** 01.01.1952
**Anrede:** Hr.
**Biologisches Geschlecht:** m
**Besonderheiten:** Isolation
**Hauptdiagnose:** Pneumonie links
**Nebendiagnose:** Diabetes mellitus Typ 2
**Pflege:** mit Hilfe mobil, Normalkost, Katheter, Sturzrisiko
**Info:** Labor 14 Uhr, Röntgen ausstehend
**ToDo:** Röntgen-Termin klären

### Bett B

[leer]

## Zimmer 102

### Bett A

[leer]
```

Ein Bett ohne Patient wird als `[leer]` gespeichert. Der Parser überspringt diese Zeile beim Einlesen.

Die optionale Zeile `**Matrix:** true` im Header speichert die Screensaver-Einstellung. Ist sie nicht vorhanden oder auf `false` gesetzt, startet die App mit dem Standard-Screensaver [Logo]. Beim nächsten Speichern wird der aktuelle Zustand wieder geschrieben.

---

## PDF- und ODT-Export

### PDF [`Strg+P`]

Der PDF-Export erzeugt ein DIN-A4-Dokument mit allen Betten der Station. Zwei Betten eines Zimmers stehen nebeneinander auf einer Seite. Jedes Bett wird als Karte mit schwarzem Kopf [Zimmernummer, Bett, Patientenname] und 5 Feldern [HDIA, NDIA, NURS, INFO, TODO] dargestellt. Die Seitenzahl ist **dynamisch**: pro Seite werden 6 Zimmer dargestellt, die Gesamtseitenzahl ergibt sich aus `ceil(Zimmeranzahl / 6)`.

```
┌────────────────────────────────┬───────────────────────────────┐
│ 101 A  Hr. Hamster [*73, m]    │ 101 B  Fr. Fuchs [*61, w]     │
│ HDIA: Pneumonie links          │ HDIA: Herzinsuffizienz        │
│ NDIA: Diabetes Typ 2           │ NDIA: …                       │
│ NURS: …                        │ NURS: …                       │
│ INFO: …                        │ INFO: …                       │
│ TODO: …                        │ TODO: …                       │
└────────────────────────────────┴───────────────────────────────┘
```

- Ränder: 10 mm links und rechts
- Schriftgröße 7 pt, Feldlabels 7 pt [fett], bis zu 3 Zeilen pro Feld
- Patientenname im PDF ohne Vornamen: `Hr. Hamster [*73, m]`
- Umgesetzt mit `printpdf`

### ODT [`Strg+L`]

Der ODT-Export erzeugt ein bearbeitbares Dokument im **Open Document Format** [ODF], das sich direkt in LibreOffice Writer, Google Docs oder Microsoft Word öffnen lässt. Er ist als **Ergänzung zum PDF** gedacht: Wem die Schrift im kompakten PDF-Ausdruck zu klein ist, kann das ODT nach Belieben formatieren - Schriftgröße, Seitenränder, Spaltenbreiten und Farben lassen sich frei anpassen.

Das ODT-Dokument stellt jedes Bett als eigene Tabelle dar:

- **Schwarze Kopfzeile** mit weißer Schrift: Zimmernummer, Bett und Patientenname
- **5 Zeilen** darunter mit den Feldern HDIA, NDIA, NURS, INFO, TODO
- Standardschriftgröße **11 pt**, Seitenränder **10 mm**
- Am Ende: Dienstinfo-Block im gleichen Stil
- Fußzeile mit MZ-HyprNurs-Kennung und Druckdatum

Umgesetzt mit dem `zip`-Crate als reiner ODF-ZIP-Container ohne externe Abhängigkeiten.

### Warum ODF? Gesetzliche Grundlage und Zukunftssicherheit

Der **IT-Planungsrat** hat am 19. März 2026 im Rahmen des Deutschland-Stacks verbindlich festgelegt, dass die gesamte öffentliche Verwaltung in Deutschland [Bund, Länder, Kommunen] beim digitalen Dokumentenaustausch ausschließlich **ODF** [für bearbeitbare Dokumente] und **PDF** [für nicht veränderliche Dokumente] verwenden darf. Die Umsetzungsfrist ist **2028**. MZ-HyprNurs folgt diesem Standard bereits heute und ist damit für den Einsatz in öffentlichen Krankenhäusern und Einrichtungen vorbereitet.

### Zukunftssicheres Datenformat

MZ-HyprNurs speichert alle Stationsdaten als **Markdown** [`.md`] - ein offenes, menschenlesbares Textformat. Das bringt entscheidende Vorteile:

- **Kein Vendor-Lock-in:** Die Daten sind nicht an MZ-HyprNurs gebunden. Jeder Texteditor kann die `.md`-Dateien öffnen und lesen.
- **Import in andere Systeme:** Markdown wird von nahezu allen modernen Anwendungen unterstützt - Wikis, Dokumentationssysteme, KIS-Schnittstellen oder eigene Skripte können die Daten direkt weiterverarbeiten.
- **Langzeitlesbarkeit:** Auch in 10 oder 20 Jahren wird eine `.md`-Datei problemlos lesbar sein - ohne spezielle Software, ohne Lizenzen, ohne Konvertierung.
- **Versionierung mit Git:** Da Markdown reiner Text ist, lässt es sich mit Git versionieren. Änderungen sind zeilengenau nachvollziehbar.

---

## Theme-System

### Varianten

Sechs Themes, umschaltbar mit `Strg+T`:

| Theme      | Hintergrund                | Schrift/Akzent           | Besonderheit                           |
|------------|----------------------------|--------------------------|----------------------------------------|
| `Hell`     | egui Standard [hell]       | Schwarz / Pink           | -                                      |
| `Dunkel`   | Reines Schwarz             | Grau / Rot               | -                                      |
| `CPCgruen`    | Dunkelgrün [`#001800`]     | Phosphorgrün [`#33FF33`] | CRT-Scanline-Optik, GT65-Grünmonitor   |
| `CPCrot`      | Dunkelrot [`#180000`]      | Leuchtendes Rot [`#FF3333`] | CRT-Scanline-Optik, Rotmonitor-Variante |
| `CPCblaugelb` | Blau [`#000080`]           | Gelb [`#FFFF00`]         | CTM644-Farbmonitor-Optik               |
| `Omarchy`  | Aus `colors.toml`          | Aus `colors.toml`        | Nur wenn Konfigurationsdatei vorhanden |

### Omarchy-Integration

Die Funktion `omarchy_farben_laden` liest TOML-Zeilen der Form `key = "#rrggbb"` aus
`~/.config/omarchy/current/theme/colors.toml` ein.

| TOML-Schlüssel | Verwendung in der App                    |
|----------------|------------------------------------------|
| `background`   | Fensterhintergrund und Kartenhintergrund |
| `foreground`   | Texte in Eingabefeldern                  |
| `accent`       | Kopfblock, Buttons, Hover-Effekte        |
| `color3`       | Beschriftungen [Labels]                  |
| `color8`       | Trennlinien, Hover-Hintergrund           |

Das Omarchy-Theme wird nur im Zyklus angeboten, wenn die Konfigurationsdatei gefunden wurde [`hat_omarchy = true`]. Beim Start wählt die App automatisch Omarchy, wenn verfügbar - das ist der Standard auf **Omarchy Linux**. Ist keine Omarchy-Konfiguration vorhanden, startet die App mit dem **schwarz-roten Theme** als Standard.

---

## Psychiatrie-Modus

Der Psychiatrie-Modus lässt sich ausschließlich über **Einstellungen** [`Strg+Shift+E`] aktivieren - bewusst kein Menüeintrag und keine eigene Tastenkombination, damit er nicht versehentlich ausgelöst wird. Er passt die Feldbeschriftungen an die psychiatrische Fachsprache an:

| Standard        | Psychiatrie-Modus          | Langform                   |
|-----------------|----------------------------|----------------------------|
| `HDIA`          | `PDIA`                     | Psychiatrische Diagnose    |
| `NDIA`          | `SDIA`                     | Somatische Diagnose        |

Die Umbenennung gilt überall: in der Bettenübersicht, im Detailfenster, im HyprGross-Modus, im PDF-Export und im ODT-Export. In der gespeicherten `.md`-Datei werden die Felder als `**PDIA:**` und `**SDIA:**` abgelegt statt als `**Hauptdiagnose:**` und `**Nebendiagnose:**`.

**Auto-Erkennung beim Laden:** Enthält eine geladene `.md`-Datei ein `**PDIA:**`-Feld, schaltet die App automatisch in den Psychiatrie-Modus - auch wenn kein `**Psychiatrie:** true`-Header vorhanden ist. Eine gemischte Datei [teils HDIA, teils PDIA] wird korrekt eingelesen; der Modus richtet sich nach dem ersten erkannten PDIA-Eintrag.

Der Zustand wird beim Speichern im Header der `.md`-Datei vermerkt [`**Psychiatrie:** true`].

---

## Datei-Dialoge und Thread-Kommunikation

Da `rfd::FileDialog` den Haupt-Thread blockieren würde, laufen alle Dialoge in
eigenen Threads. Die Kommunikation erfolgt über `std::sync::mpsc`:

```rust
let (sender, empfaenger) = mpsc::channel::<DialogErgebnis>();
self.dialog_rx = Some(empfaenger);
std::thread::spawn(move || {
    if let Some(pfad) = rfd::FileDialog::new()...pick_file() {
        let _ = sender.send(DialogErgebnis::Laden(pfad, inhalt));
    }
});
// Im nächsten update()-Aufruf:
if let Ok(ergebnis) = self.dialog_rx.try_recv() { ... }
```

Es kann immer nur ein Dialog gleichzeitig geöffnet sein [`dialog_rx` ist `Option`].

---

## Einstellungen-Dialog

Über den Menüpunkt **„Einstellungen"** im Hamburger-Menü [oder `Strg+Shift+E`] öffnet sich ein zentriertes Popup-Fenster zur Konfiguration der Station.

### Zimmeranzahl ändern

Im Dialog wird die aktuelle Zimmeranzahl angezeigt. Mit den Schaltflächen **„−"** und **„+"** lässt sie sich zwischen 1 und 50 anpassen.

- **Zimmer hinzufügen:** Neue Zimmer werden am Ende der Liste angehängt. Die Zimmernummer wird automatisch aus der höchsten vorhandenen Nummer fortgesetzt [z. B. nach Zimmer 108 folgt 109]. Neue Zimmer enthalten zwei leere Betten [A und B].
- **Zimmer löschen:** Zimmer werden von unten weggenommen. **Alle Patientendaten in gelöschten Zimmern gehen unwiderruflich verloren.** Eine Warnung im Dialog weist darauf explizit hin.

### Verhalten bei Indexkollision

Wird die Zimmeranzahl unter den aktuell ausgewählten oder bearbeiteten Zimmer-Index gesenkt, setzt die App die Auswahl zurück [`ausgewaehlte_karte` und `bearbeitung`], um ungültige Zugriffe zu verhindern.

### Auswirkung auf Exporte

PDF, ODT und `.md`-Datei passen sich automatisch an - alle Exporte arbeiten direkt mit dem aktuellen `station.zimmer`-Vektor, sodass keine gesonderte Anpassung notwendig ist.

---

## Validierung im Detailfenster

Zimmernummer und Bettbezeichnung werden beim Bearbeiten laufend geprüft. Solange ein Fehler besteht, erscheint ein roter Warnbalken direkt unterhalb des Kopfstreifens - und das Fenster lässt sich weder per ESC noch per ×-Button schließen. Auch das Speichern [manuell und Auto-Save] ist blockiert.

Folgende Zustände werden abgefangen:

| Fehler | Meldung |
|--------|---------|
| Zimmernummer ist leer | „Zimmernummer darf nicht leer sein." |
| Bettbezeichnung ist leer | „Bettbezeichnung darf nicht leer sein." |
| Zimmer+Bett-Kombination existiert bereits | „Diese Zimmer+Bett-Kombination existiert bereits - bitte anpassen." |

### Automatisches Trimmen beim Speichern

Direkt vor dem Schreiben der Datei werden folgende Felder automatisch von führenden und abschließenden Leerzeichen und Zeilenumbrüchen bereinigt [`.trim()`]:

- Zimmernummer, Bettbezeichnung
- HDIA / PDIA, NDIA / SDIA, PFLEGE, INFO, TODO

---

## Tastenkombinationen

| Kombination      | Aktion                                                                  |
|------------------|-------------------------------------------------------------------------|
| `Strg+N`         | Neue Station anlegen [aktuelle Daten verwerfen, mit Bestätigung]        |
| `Strg+O`         | Datei öffnen                                                            |
| `Strg+S`         | Speichern                                                               |
| `Strg+Shift+S`   | Speichern unter [neuen Pfad wählen]                                     |
| `Strg+P`         | PDF erzeugen                                                            |
| `Strg+L`         | ODT erzeugen                                                            |
| `Strg+T`         | Theme wechseln                                                          |
| `Strg+M`         | Matrix-Regen-Screensaver ein/aus [Einstellung wird in Datei gespeichert]|
| `Strg+G`         | HyprGross-Modus ein/aus                                                 |
| `Strg+Shift+E`   | Einstellungen öffnen [Zimmeranzahl, Psychiatrie-Modus]                  |
| `Strg+H`         | Hilfe-Website öffnen                                                    |
| `Strg+I`         | Über MZ-HyprNurs                                                        |
| `Strg+Q`         | Beenden [mit Bestätigungsdialog]                                        |
| `ESC`            | Detailfenster schließen / HyprGross beenden                             |
| `←` / `→`       | Feld wechseln [Detailfenster / HyprGross; an Grenzen: nächstes Bett]   |
| `↑` / `↓`       | Nächstes / vorheriges belegtes Bett [HyprGross]                        |
| `←` `→` `↑` `↓` | Karte auswählen [Übersicht]                                             |
| `Enter`          | Detailfenster des markierten Bettes öffnen [Übersicht]                  |

---

## Build und Installation

### Voraussetzungen

- Rust [stable, getestet mit Edition 2021]
- **Linux [Arch/Omarchy]:** Alles außer Rust ist auf Omarchy Linux bereits vorhanden. Für Minimal-Arch-Installationen: `base-devel`, `pkg-config`, `gtk3`, `openssl`, `libxkbcommon`.
- **Linux [Debian/Ubuntu]:** `pkg-config`, `libssl-dev`, `libgtk-3-dev`, `libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`, `libxkbcommon-dev`.
- Für alle Linux-Distributionen zusätzlich: eine Systemschrift [Liberation Sans, Noto Sans oder DejaVu Sans] für den PDF-Export.
- **Windows:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit der Komponente „Desktop development with C++" [enthält MSVC-Compiler und Windows SDK]
- **macOS:** macOS 26 oder neuer, Xcode Command Line Tools [`xcode-select --install`]; Systemschriften [Arial, Verdana etc.] sind standardmäßig vorhanden

### Debug-Build

**Linux / macOS:**
```bash
cargo build
./target/debug/mz-hyprnurs
```

**Windows:**
```cmd
cargo build
target\debug\mz-hyprnurs.exe
```

### Release-Build

**Linux / macOS:**
```bash
cargo build --release
./target/release/mz-hyprnurs
```

**Windows:**
```cmd
cargo build --release
target\release\mz-hyprnurs.exe
```

### Linux-Installation [Omarchy / Arch]

```bash
chmod +x install.sh
./install.sh
```

Das Skript kopiert die Binary nach `~/.local/bin/mz-hyprnurs`, das Icon nach
`~/.local/share/icons/hicolor/256x256/apps/` und erstellt einen `.desktop`-Eintrag unter
`~/.local/share/applications/`, sodass MZ-HyprNurs im Walker-Launcher erscheint.

### macOS - .app-Bundle erstellen

Das `.app`-Bundle wird automatisch vom CI-Workflow `.github/workflows/release.yml`
beim Tag-Release erzeugt [Ziel: `aarch64-apple-darwin`]. Der Workflow erstellt
`MZ-HyprNurs.app` mit Binary, `Info.plist` und `icon.icns` und hängt sie als
`mz-hyprnurs-macos-aarch64.zip` an das GitHub-Release an.

Lokaler Bundle-Aufbau [falls manuell benötigt]:

```bash
cargo build --release --target aarch64-apple-darwin
mkdir -p MZ-HyprNurs.app/Contents/{MacOS,Resources}
cp target/aarch64-apple-darwin/release/mz-hyprnurs MZ-HyprNurs.app/Contents/MacOS/
cp assets/Info.plist MZ-HyprNurs.app/Contents/
cp assets/icon.icns MZ-HyprNurs.app/Contents/Resources/
open MZ-HyprNurs.app
```

### macOS - Gatekeeper-Hinweis

Da die App nicht mit einem Apple-Entwicklerzertifikat signiert ist, blockiert macOS
beim ersten Start die Ausführung. So lässt sich die App trotzdem starten:

```bash
xattr -cr /pfad/zu/MZ-HyprNurs.app
```

---

*Diese README wurde am 17.04.2026 erstellt und zuletzt aktualisiert am 17.04.2026 [Version 1.0.0].*
