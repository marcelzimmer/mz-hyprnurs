#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui::{Color32, FontId, Pos2, Rect, RichText, Rounding, Stroke, Vec2};
use std::collections::HashMap;
use std::sync::mpsc;

// ── URL im Systembrowser öffnen ───────────────────────────────────────────────
fn url_oeffnen(url: &str) {
    #[cfg(windows)]
    let _ = std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

// ── Omarchy-Theme-Unterstützung ───────────────────────────────────────────────
fn hex_farbe_parsen(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}

fn omarchy_farben_laden() -> Option<HashMap<String, Color32>> {
    let home = std::env::var("HOME").ok()?;
    let pfad = format!("{}/.config/omarchy/current/theme/colors.toml", home);
    let inhalt = std::fs::read_to_string(&pfad).ok()?;
    let mut farben = HashMap::new();
    for zeile in inhalt.lines() {
        let zeile = zeile.trim();
        if let Some((schluessel, wert)) = zeile.split_once('=') {
            let schluessel = schluessel.trim().to_string();
            let wert = wert.trim().trim_matches('"');
            if let Some(farbe) = hex_farbe_parsen(wert) {
                farben.insert(schluessel, farbe);
            }
        }
    }
    Some(farben)
}

// sRGB-Kanal [0..=255] → linearer Wert [0..=1]
fn srgb_zu_linear(kanal: u8) -> f32 {
    let x = kanal as f32 / 255.0;
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

// Linearer Wert [0..=1] → sRGB-Kanal [0..=255]
fn linear_zu_srgb(x: f32) -> u8 {
    let v = if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// WCAG-konforme relative Luminanz [0..=1].
fn luminanz(c: Color32) -> f32 {
    let [r, g, b, _] = c.to_array();
    0.2126 * srgb_zu_linear(r) + 0.7152 * srgb_zu_linear(g) + 0.0722 * srgb_zu_linear(b)
}

/// Linearer Farbmix im sRGB-linearen Raum. t=0 → a, t=1 → b.
fn mischen(a: Color32, b: Color32, t: f32) -> Color32 {
    let [ar, ag, ab, _] = a.to_array();
    let [br, bg, bb, _] = b.to_array();
    let t = t.clamp(0.0, 1.0);
    let kanal = |x: u8, y: u8| {
        linear_zu_srgb(srgb_zu_linear(x) * (1.0 - t) + srgb_zu_linear(y) * t)
    };
    Color32::from_rgb(kanal(ar, br), kanal(ag, bg), kanal(ab, bb))
}

/// true, wenn die Farbe eher hell wirkt [Luminanz > 0.5].
fn ist_hell(c: Color32) -> bool {
    luminanz(c) > 0.5
}

// ── Design-Konstanten ────────────────────────────────────────────────────────
const KARTE_B: f32 = 590.0;
const KARTE_ABSTAND: f32 = 24.0;
const KOPF_ZELLE_B: f32 = 72.0;
const KOPF_H: f32 = 50.0;
const FELD_H: f32 = 82.0;
const KARTE_RUND: f32 = 12.0;

// Felder im Bearbeitungsfenster (Index 0 = Patienteninfo)
const FELD_TITEL: [&str; 6] = [
    "Patient",
    "Hauptdiagnose",
    "Nebendiagnose",
    "Pflege",
    "Info",
    "ToDo",
];
const FELD_ANZAHL: usize = FELD_TITEL.len();

// ── HyprGross-Ansicht ─────────────────────────────────────────────────────────
#[derive(PartialEq, Clone, Copy)]
enum HyprGrossAnsicht { Feld, HyprInfo, DienstInfo }

// ── Altersberechnung ──────────────────────────────────────────────────────────
fn alter_aus_geburtsdatum(s: &str) -> Option<u32> {
    let p: Vec<&str> = s.split('.').collect();
    if p.len() != 3 { return None; }
    let gt: u32 = p[0].trim().parse().ok()?;
    let gm: u32 = p[1].trim().parse().ok()?;
    let gj: u32 = p[2].trim().parse().ok()?;
    if !(1900..=2100).contains(&gj) || !(1..=12).contains(&gm) || !(1..=31).contains(&gt) { return None; }
    use chrono::Datelike;
    let heute = chrono::Local::now();
    let (hj, hm, ht) = (heute.year() as u32, heute.month(), heute.day());
    let mut alter = hj.saturating_sub(gj);
    if hm < gm || (hm == gm && ht < gt) { alter = alter.saturating_sub(1); }
    Some(alter)
}

// Alter + Bio-Geschlecht als Suffix: " [*72, m]" / " [*72]" / " [m]" / ""
fn alter_bio_suffix(geburtsdatum: &str, bio_geschlecht: &str) -> String {
    let a = alter_aus_geburtsdatum(geburtsdatum).map(|a| format!("*{}", a));
    let b = if bio_geschlecht.is_empty() { None } else { Some(bio_geschlecht.to_string()) };
    match (a, b) {
        (Some(alt), Some(bio)) => format!(" [{}, {}]", alt, bio),
        (Some(alt), None)      => format!(" [{}]", alt),
        (None,      Some(bio)) => format!(" [{}]", bio),
        (None,      None)      => String::new(),
    }
}

// Namensanzeige kurz für Karten/PDF: "Hr. Löwe [*72, m]" (kein Vorname wenn Anrede gesetzt)
fn name_anzeige(pat: &Patient) -> String {
    let suffix = alter_bio_suffix(&pat.geburtsdatum, &pat.bio_geschlecht);
    let anrede_prefix = if !pat.anrede.is_empty() && pat.anrede != "–" {
        format!("{} ", pat.anrede)
    } else { String::new() };
    if !anrede_prefix.is_empty() {
        format!("{}{}{}", anrede_prefix, pat.nachname, suffix)
    } else {
        match (pat.nachname.trim(), pat.vorname.trim()) {
            (n, v) if !n.is_empty() && !v.is_empty() => format!("{}, {}{}", n, v, suffix),
            (n, _) if !n.is_empty() => format!("{}{}", n, suffix),
            (_, v) if !v.is_empty() => format!("{}{}", v, suffix),
            _ => suffix.trim().to_string(),
        }
    }
}

// Namensanzeige lang für Modal-Header/HyprGross: "Hr. Löwe, Leon [*72, m]" (immer mit Vorname)
fn name_anzeige_lang(pat: &Patient) -> String {
    let suffix = alter_bio_suffix(&pat.geburtsdatum, &pat.bio_geschlecht);
    let anrede_prefix = if !pat.anrede.is_empty() && pat.anrede != "–" {
        format!("{} ", pat.anrede)
    } else { String::new() };
    match (pat.nachname.trim(), pat.vorname.trim()) {
        (n, v) if !n.is_empty() && !v.is_empty() => format!("{}{}, {}{}", anrede_prefix, n, v, suffix),
        (n, _) if !n.is_empty() => format!("{}{}{}", anrede_prefix, n, suffix),
        (_, v) if !v.is_empty() => format!("{}{}{}", anrede_prefix, v, suffix),
        _ => suffix.trim().to_string(),
    }
}

fn format_geburtsdatum(s: &mut String) {
    let ziffern: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut ergebnis = String::new();
    for (i, c) in ziffern.chars().enumerate() {
        if i == 2 || i == 4 { ergebnis.push('.'); }
        if i < 8 { ergebnis.push(c); }
    }
    *s = ergebnis;
}

fn geburtsdatum_gueltig(s: &str) -> bool {
    let p: Vec<&str> = s.split('.').collect();
    if p.len() != 3 { return false; }
    let (Ok(d), Ok(m), Ok(y)) = (
        p[0].parse::<u32>(),
        p[1].parse::<u32>(),
        p[2].parse::<u32>(),
    ) else { return false; };
    if !(1900..=2100).contains(&y) || !(1..=12).contains(&m) || d < 1 { return false; }
    let max_tag = match m {
        1|3|5|7|8|10|12 => 31,
        4|6|9|11 => 30,
        2 => if y % 400 == 0 || (y % 4 == 0 && y % 100 != 0) { 29 } else { 28 },
        _ => return false,
    };
    if d > max_tag || p[0].len() != 2 || p[1].len() != 2 || p[2].len() != 4 {
        return false;
    }
    // Nicht in der Zukunft
    let heute = chrono::Local::now().date_naive();
    let Some(geb) = chrono::NaiveDate::from_ymd_opt(y as i32, m, d) else { return false; };
    geb <= heute
}

// ── Datenmodell ───────────────────────────────────────────────────────────────
#[derive(Default, PartialEq, Clone)]
struct Patient {
    nachname: String,
    vorname: String,
    geburtsdatum: String,
    anrede: String,           // "Hr." / "Fr." / "–"
    bio_geschlecht: String,   // "m" / "w" / "d"
    besonderheiten: String,
    hdia: String,
    ndia: String,
    info: String,
    pflege: String,
    todo: String,
}

struct Bett {
    buchstabe: String,
    patient: Option<Patient>,
}

struct Zimmer {
    nummer: String,
    betten: Vec<Bett>,
}

struct Station {
    zimmer: Vec<Zimmer>,
    dienst_info: String,
}

struct Konflikt {
    zi:             usize,
    bi:             usize,
    zimmer_nr:      String,
    bett_buchstabe: String,
    lokal:          Option<Patient>,
    disk:           Option<Patient>,
}

fn betten_aus_markdown(content: &str) -> std::collections::HashMap<(String, String), Option<Patient>> {
    let mut map = std::collections::HashMap::new();
    #[derive(PartialEq)]
    enum Abs { Header, Dienstinfo, Bett, Andere }
    let mut abs = Abs::Header;
    let mut akt_zimmer: Option<String> = None;
    let mut akt_bett:   Option<String> = None;
    let mut akt_patient: Option<Patient> = None;

    let flush = |map: &mut std::collections::HashMap<(String, String), Option<Patient>>,
                     zimmer: &Option<String>, bett: &Option<String>, patient: Option<Patient>| {
        if let (Some(z), Some(b)) = (zimmer, bett) {
            map.insert((z.clone(), b.clone()), patient);
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## Zimmer ") {
            flush(&mut map, &akt_zimmer, &akt_bett, akt_patient.take());
            akt_zimmer = Some(trimmed.trim_start_matches("## Zimmer ").to_string());
            akt_bett = None;
            abs = Abs::Andere;
            continue;
        }
        if trimmed == "## Dienstinfo" { abs = Abs::Dienstinfo; continue; }
        if trimmed.starts_with("### Bett ") {
            flush(&mut map, &akt_zimmer, &akt_bett, akt_patient.take());
            akt_bett = Some(trimmed.trim_start_matches("### Bett ").to_string());
            abs = Abs::Bett;
            continue;
        }
        if abs == Abs::Bett {
            if trimmed.is_empty() || trimmed == "(leer)" || trimmed == "[leer]" { continue; }
            let pat = akt_patient.get_or_insert_with(Patient::default);
            if let Some(v) = trimmed.strip_prefix("**Nachname:** ")          { pat.nachname         = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Vorname:** ")       { pat.vorname          = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Geburtsdatum:** ")  { pat.geburtsdatum     = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Anrede:** ")        { pat.anrede           = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Biologisches Geschlecht:** ") { pat.bio_geschlecht = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Besonderheiten:** "){ pat.besonderheiten   = v.to_string(); }
            else if let Some(v) = trimmed.strip_prefix("**Hauptdiagnose:** ") { pat.hdia = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**Nebendiagnose:** ") { pat.ndia = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**PDIA:** ")          { pat.hdia = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**SDIA:** ")          { pat.ndia = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**Info:** ")          { pat.info  = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**Pflege:** ")        { pat.pflege = v.replace("\\n", "\n"); }
            else if let Some(v) = trimmed.strip_prefix("**ToDo:** ")          { pat.todo  = v.replace("\\n", "\n"); }
        }
    }
    flush(&mut map, &akt_zimmer, &akt_bett, akt_patient.take());
    map
}

impl Station {
    fn beispieldaten() -> Self {
        Station {
            zimmer: vec![
                Zimmer {
                    nummer: "101".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "102".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "103".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "104".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "105".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "106".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "107".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "108".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "109".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "110".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "111".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
                Zimmer {
                    nummer: "112".to_string(),
                    betten: vec![
                        Bett { buchstabe: "A".to_string(), patient: None },
                        Bett { buchstabe: "B".to_string(), patient: None },
                    ],
                },
            ],
            dienst_info: String::new(),
        }
    }
}

// ── Theme ─────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
enum Theme {
    /// Helles Standard-Theme
    Hell,
    /// Dunkles Theme mit rotem Akzent
    Dunkel,
    /// Amstrad CPC 464 Grünmonitor
    CPCgruen,
    /// Amstrad CPC 464 Rotmonitor
    CPCrot,
    /// Amstrad CPC 464 Farbmonitor (Blau/Gelb)
    CPCblaugelb,
    /// Omarchy-Desktop-Theme (nur wenn Konfigurationsdatei vorhanden)
    Omarchy,
}

impl Theme {
    fn naechstes(self, hat_omarchy: bool) -> Self {
        match self {
            Theme::Omarchy  => Theme::Dunkel,
            Theme::Dunkel   => Theme::Hell,
            Theme::Hell     => Theme::CPCgruen,
            Theme::CPCgruen => Theme::CPCrot,
            Theme::CPCrot   => Theme::CPCblaugelb,
            Theme::CPCblaugelb => if hat_omarchy { Theme::Omarchy } else { Theme::Dunkel },
        }
    }
    fn akzent_farbe(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::from_rgb(236, 72, 153),
            Theme::Dunkel   => Color32::from_rgb(255, 0, 0),
            Theme::CPCgruen => Color32::from_rgb(0x33, 0xFF, 0x33),
            Theme::CPCrot   => Color32::from_rgb(0xFF, 0x33, 0x33),
            Theme::CPCblaugelb => Color32::from_rgb(0xFF, 0xFF, 0x00),
            Theme::Omarchy  => Color32::from_rgb(122, 162, 247),
        }
    }
    fn karten_hintergrund(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::WHITE,
            Theme::Dunkel   => Color32::from_rgb(0, 0, 0),
            Theme::CPCgruen => Color32::from_rgb(0x00, 0x20, 0x00),
            Theme::CPCrot   => Color32::from_rgb(0x28, 0x00, 0x00),
            Theme::CPCblaugelb => Color32::from_rgb(0x00, 0x00, 0xA0),
            Theme::Omarchy  => Color32::from_rgb(26, 27, 37),
        }
    }
    fn karten_hover(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::from_rgb(255, 248, 252),
            Theme::Dunkel   => Color32::from_rgb(52, 52, 62),
            Theme::CPCgruen => Color32::from_rgb(0x00, 0x33, 0x00),
            Theme::CPCrot   => Color32::from_rgb(0x40, 0x00, 0x00),
            Theme::CPCblaugelb => Color32::from_rgb(0x00, 0x00, 0xB0),
            Theme::Omarchy  => Color32::from_rgb(36, 37, 50),
        }
    }
    fn fenster_hintergrund(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::WHITE,
            Theme::Dunkel   => Color32::from_rgb(0, 0, 0),
            Theme::CPCgruen => Color32::from_rgb(0x00, 0x18, 0x00),
            Theme::CPCrot   => Color32::from_rgb(0x18, 0x00, 0x00),
            Theme::CPCblaugelb => Color32::from_rgb(0x00, 0x00, 0x80),
            Theme::Omarchy  => Color32::from_rgb(20, 21, 31),
        }
    }
    fn text_farbe(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::BLACK,
            Theme::Dunkel   => Color32::from_gray(225),
            Theme::CPCgruen => Color32::from_rgb(0x33, 0xFF, 0x33),
            Theme::CPCrot   => Color32::from_rgb(0xFF, 0x33, 0x33),
            Theme::CPCblaugelb => Color32::from_rgb(0xFF, 0xFF, 0x00),
            Theme::Omarchy  => Color32::from_gray(200),
        }
    }
    fn bezeichnung_farbe(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::from_gray(100),
            Theme::Dunkel   => Color32::from_gray(145),
            Theme::CPCgruen => Color32::from_rgb(0x22, 0xCC, 0x22),
            Theme::CPCrot   => Color32::from_rgb(0xCC, 0x22, 0x22),
            Theme::CPCblaugelb => Color32::from_rgb(0xCC, 0xCC, 0x00),
            Theme::Omarchy  => Color32::from_gray(140),
        }
    }
    fn trennlinie_farbe(self) -> Color32 {
        match self {
            Theme::Hell     => Color32::from_gray(215),
            Theme::Dunkel   => Color32::from_gray(62),
            Theme::CPCgruen => Color32::from_rgb(0x00, 0x66, 0x00),
            Theme::CPCrot   => Color32::from_rgb(0x66, 0x00, 0x00),
            Theme::CPCblaugelb => Color32::from_rgb(0x00, 0x00, 0x60),
            Theme::Omarchy  => Color32::from_gray(55),
        }
    }
    /// Textfarbe auf dem Akzent-Hintergrund (Kopfblock, Kopfstreifen)
    fn kopf_text_farbe(self) -> Color32 {
        match self {
            Theme::CPCblaugelb => Color32::BLACK,
            _               => Color32::WHITE,
        }
    }
    /// Gedimmte Farbe auf dem Akzent-Hintergrund (Hints, Trennzeichen, Dots)
    fn kopf_dim_farbe(self) -> Color32 {
        match self {
            Theme::CPCblaugelb => Color32::from_black_alpha(160),
            _               => Color32::from_white_alpha(140),
        }
    }
}

// ── Aufgelöste Farben für den aktuellen Frame ─────────────────────────────────
struct AktFarben {
    akzent:       Color32,
    karten_bg:    Color32,
    karten_hover: Color32,
    fenster_bg:   Color32,
    text:         Color32,
    bezeichnung:  Color32,
    trennlinie:   Color32,
    kopf_text:    Color32,
    kopf_dim:     Color32,
}

impl AktFarben {
    fn von_theme(theme: Theme, omarchy: Option<&HashMap<String, Color32>>) -> Self {
        if theme == Theme::Omarchy {
            if let Some(f) = omarchy {
                // Stabile Keys der Omarchy-colors.toml direkt übernehmen.
                let bg     = f.get("background").copied().unwrap_or(Color32::from_rgb(26, 27, 37));
                let fg     = f.get("foreground").copied().unwrap_or(Color32::from_gray(200));
                let akzent = f.get("accent")    .copied().unwrap_or(Color32::from_rgb(122, 162, 247));

                // UI-Chrome aus fg/bg-Blend ableiten, adaptiv für helle vs. dunkle Themes.
                let (t_hover, t_trenn, t_bez) = if ist_hell(bg) {
                    (0.06, 0.18, 0.30)
                } else {
                    (0.08, 0.18, 0.50)
                };
                let hover = mischen(bg, fg, t_hover);
                let trenn = mischen(bg, fg, t_trenn);
                let bez   = mischen(fg, bg, t_bez);

                // Kopftext: Schwarz auf hellem Akzent, sonst Weiß.
                let (kopf_text, kopf_dim) = if luminanz(akzent) > 0.18 {
                    (Color32::BLACK, Color32::from_black_alpha(160))
                } else {
                    (Color32::WHITE, Color32::from_white_alpha(160))
                };

                return Self {
                    akzent,
                    karten_bg:    bg,
                    karten_hover: hover,
                    fenster_bg:   bg,
                    text:         fg,
                    bezeichnung:  bez,
                    trennlinie:   trenn,
                    kopf_text,
                    kopf_dim,
                };
            }
        }
        Self {
            akzent:       theme.akzent_farbe(),
            karten_bg:    theme.karten_hintergrund(),
            karten_hover: theme.karten_hover(),
            fenster_bg:   theme.fenster_hintergrund(),
            text:         theme.text_farbe(),
            bezeichnung:  theme.bezeichnung_farbe(),
            trennlinie:   theme.trennlinie_farbe(),
            kopf_text:    theme.kopf_text_farbe(),
            kopf_dim:     theme.kopf_dim_farbe(),
        }
    }
}

// ── Datum/Zeit-Hilfsfunktionen ────────────────────────────────────────────────

fn jetzt_formatiert() -> String {
    chrono::Local::now().format("%d.%m.%Y @ %H:%M:%S Uhr").to_string()
}

fn benutzer_info() -> String {
    // USERNAME auf Windows, USER auf Linux/macOS
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Unbekannt".to_string());
    let hostname = hostname_ermitteln();
    format!("{} @ {}", user, hostname)
}

fn hostname_ermitteln() -> String {
    // Windows: COMPUTERNAME ist immer gesetzt
    #[cfg(target_os = "windows")]
    if let Ok(h) = std::env::var("COMPUTERNAME") { return h; }

    // Linux: /proc ist zuverlässiger als env-Variable
    #[cfg(target_os = "linux")]
    if let Ok(h) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        let h = h.trim();
        if !h.is_empty() { return h.to_string(); }
    }

    // macOS + Linux-Fallback: hostname-Befehl
    #[cfg(not(target_os = "windows"))]
    if let Ok(out) = std::process::Command::new("hostname").output() {
        if let Ok(s) = std::str::from_utf8(&out.stdout) {
            let s = s.trim();
            if !s.is_empty() { return s.to_string(); }
        }
    }

    // Letzter Fallback: HOSTNAME env-Variable (manchmal auf macOS/Linux gesetzt)
    std::env::var("HOSTNAME").unwrap_or_else(|_| "UnbekannterHost".to_string())
}

fn backup_erstellen(pfad: &std::path::Path) {
    if !pfad.exists() { return; }
    let backup_pfad = pfad.with_extension("backup");
    let _ = std::fs::copy(pfad, &backup_pfad);
}

// Atomar schreiben: erst in *.tmp im Zielverzeichnis, dann rename.
// Verhindert, dass ein Crash/Stromausfall mitten im Schreiben die Datei korrumpiert.
fn atomar_schreiben(pfad: &std::path::Path, inhalt: &[u8]) -> std::io::Result<()> {
    let mut tmp_name = pfad.file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("mz-hyprnurs"));
    tmp_name.push(".tmp");
    let tmp_pfad = pfad.with_file_name(tmp_name);
    std::fs::write(&tmp_pfad, inhalt)?;
    std::fs::rename(&tmp_pfad, pfad)
}

// ── Dialog-Ergebnisse (mpsc) ───────────────────────────────────────────────────
enum DialogErgebnis {
    // Datei wurde bereits im Hintergrund geschrieben. Haupt-Thread übernimmt
    // nur noch die Zustandsaktualisierung (Zeitstempel + Inhalts-Snapshot),
    // damit "geändert"-Erkennung konsistent bleibt.
    Speichern {
        pfad:    std::path::PathBuf,
        inhalt:  String,
        am:      String,
        von:     String,
    },
    Laden(std::path::PathBuf, String),
    PdfExport(std::path::PathBuf),
    OdtExport(std::path::PathBuf),
}

// ── App ───────────────────────────────────────────────────────────────────────
struct MatrixSpalte {  // ANIM
    x:      f32,       // ANIM: x-Position der Spalte in px
    y:      f32,       // ANIM: y-Position des Kopfzeichens in px
    speed:  f32,       // ANIM: Fallgeschwindigkeit in px/s
    laenge: f32,       // ANIM: Länge des Zeichenschweifs in px
}                      // ANIM

struct MzHyprNursApp {
    station: Station,
    station_name: String,
    station_hyprinfo: String,
    bearbeitung: Option<(usize, usize)>,       // (zimmer_idx, bett_idx)
    bearbeitungsfeld: usize,                   // aktuell angezeigtes Feld (0-5)
    ausgewaehlte_karte: Option<(usize, usize)>, // Tastaturauswahl in der Übersicht
    info_ausgewaehlt: bool,                    // Dienstinfo-Box per Tastatur ausgewählt
    loeschen_bestaetigen: bool,                // Lösch-Bestätigungsdialog offen
    beenden_bestaetigen: bool,                 // Beenden-Bestätigungsdialog offen
    neu_bestaetigen: bool,                     // Neu-Bestätigungsdialog offen
    ueber_dialog_offen: bool,                   // Über-Dialog offen
    einstellungen_offen: bool,                 // Einstellungen-Dialog offen
    einstellungen_zimmer_anzahl: usize,        // temporäre Zimmeranzahl im Dialog
    einstellungen_psychiatrie: bool,           // temporärer Psychiatrie-Modus im Dialog
    geb_fehler: bool,                          // Geburtsdatum ungültig nach lost_focus
    theme: Theme,                              // aktives Farbschema
    hat_omarchy: bool,                         // Omarchy-Konfiguration gefunden
    speicher_pfad: Option<std::path::PathBuf>, // zuletzt genutzter Speicherpfad
    letzte_aenderung_am: Option<String>,       // Zeitstempel der letzten Speicherung
    letzte_aenderung_von: Option<String>,      // Benutzer @ Host der letzten Speicherung
    dialog_rx: Option<mpsc::Receiver<DialogErgebnis>>, // laufender Dateidialog
    ausstehende_pdf_schrift: Option<(Vec<u8>, Vec<u8>)>, // (regular, bold) TTF-Bytes für PDF
    inhalt_beim_speichern: String,             // Markdown-Stand beim letzten Speichern
    geaendert: bool,                           // lokale Änderungen seit letztem Speichern
    letzter_inhalt_check: std::time::Instant,  // wann zuletzt auf Änderungen geprüft
    letzte_auto_speicherung: std::time::Instant, // wann zuletzt auto-gespeichert
    konflikte:              Vec<Konflikt>,        // offene Konflikt-Popups beim Speichern
    speichern_nach_konflikt: bool,               // nach Konfliktlösung tatsächlich speichern
    speichern_nach_schliessen: bool,             // nach Schließen des Erfassungsfensters speichern
    letzte_interaktion: std::time::Instant,       // für Screensaver-Timeout
    screensaver_texture: Option<egui::TextureHandle>, // Hintergrundbild-Textur
    matrix_modus:       bool,                     // Matrix-Regen-Screensaver an/aus (Strg+M)
    psychiatrie_modus:  bool,                     // Psychiatrie-Modus an/aus [Einstellungen]
    anim_matrix:        Vec<MatrixSpalte>,        // ANIM: Matrix-Regen-Spalten
    anim_letzter_tick:  std::time::Instant,       // ANIM: für Delta-Zeit-Berechnung
    anim_frame:         u64,                      // ANIM: Frame-Zähler für Zeichenwechsel

    hyprgross_aktiv: bool,
    hyprgross_bett_pos: usize,                 // Index in nicht_leere_betten()
    hyprgross_feld: usize,                     // 0-5
    hyprgross_ansicht: HyprGrossAnsicht,
}

impl MzHyprNursApp {
    fn new() -> Self {
        let hat_omarchy = omarchy_farben_laden().is_some();
        Self {
            station: Station::beispieldaten(),
            station_name: String::new(),
            station_hyprinfo: String::new(),
            bearbeitung: None,
            bearbeitungsfeld: 0,
            ausgewaehlte_karte: Some((0, 0)),
            info_ausgewaehlt: false,
            loeschen_bestaetigen: false,
            beenden_bestaetigen: false,
            neu_bestaetigen: false,
            ueber_dialog_offen: false,
            einstellungen_offen: false,
            einstellungen_zimmer_anzahl: 0,
            einstellungen_psychiatrie: false,
            geb_fehler: false,
            hat_omarchy,
            theme: if hat_omarchy { Theme::Omarchy } else { Theme::Dunkel },
            speicher_pfad: None,
            letzte_aenderung_am: None,
            letzte_aenderung_von: None,
            dialog_rx: None,
            ausstehende_pdf_schrift: None,
            inhalt_beim_speichern: String::new(),
            geaendert: false,
            letzter_inhalt_check: std::time::Instant::now(),
            letzte_auto_speicherung: std::time::Instant::now(),
            konflikte: Vec::new(),
            speichern_nach_konflikt: false,
            speichern_nach_schliessen: false,
            letzte_interaktion: std::time::Instant::now(),
            screensaver_texture: None,
            matrix_modus:      true,
            psychiatrie_modus: false,
            anim_matrix:       Vec::new(),                    // ANIM
            anim_letzter_tick: std::time::Instant::now(),     // ANIM
            anim_frame:        0,                             // ANIM

            hyprgross_aktiv: false,
            hyprgross_bett_pos: 0,
            hyprgross_feld: 0,
            hyprgross_ansicht: HyprGrossAnsicht::Feld,
        }
    }

    // ── Hilfsfunktionen ─────────────────────────────────────────────────────

    /// Flache Liste aller Betten mit echtem Patienten (für HyprGross-Navigation).
    fn nicht_leere_betten(&self) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for (zi, zimmer) in self.station.zimmer.iter().enumerate() {
            for (bi, bett) in zimmer.betten.iter().enumerate() {
                if let Some(ref pat) = bett.patient {
                    if !pat.vorname.is_empty() || !pat.nachname.is_empty() {
                        result.push((zi, bi));
                    }
                }
            }
        }
        result
    }


    fn jetzt_zeitstempel() -> String {
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
    }

    // Lädt die Hintergrundtextur (PNG aus den eingebetteten Assets) einmalig
    // und liefert deren Id und Originalgröße. Wird vom Screensaver und vom
    // HyprGross-Modus genutzt.
    fn hintergrund_textur(&mut self, ctx: &egui::Context) -> (egui::TextureId, egui::Vec2) {
        let t = self.screensaver_texture.get_or_insert_with(|| {
            let png_bytes = include_bytes!("../assets/hintergrund.png");
            let image = image::load_from_memory(png_bytes).expect("hintergrund.png");
            let size = [image.width() as usize, image.height() as usize];
            let buf = image.to_rgba8();
            let px  = buf.as_flat_samples();
            let ci  = egui::ColorImage::from_rgba_unmultiplied(size, px.as_slice());
            ctx.load_texture("screensaver-bg", ci, egui::TextureOptions::LINEAR)
        });
        (t.id(), t.size_vec2())
    }


    fn dateinamen_erstellen(&self) -> String {
        let name: String = self.station_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
            .collect();
        format!("MZ-HYPRNURS_{}.md", name)
    }

    fn neu(&mut self) {
        self.station_name  = String::new();
        self.station_hyprinfo = String::new();
        self.station = Station {
            zimmer: (1..=12).map(|i| Zimmer {
                nummer: format!("{}", 100 + i),
                betten: vec![
                    Bett { buchstabe: "A".into(), patient: None },
                    Bett { buchstabe: "B".into(), patient: None },
                ],
            }).collect(),
            dienst_info: String::new(),
        };
        self.speicher_pfad = None;
        self.letzte_aenderung_am  = None;
        self.letzte_aenderung_von = None;
        self.bearbeitung   = None;
        self.bearbeitungsfeld = 0;
        self.ausgewaehlte_karte = Some((0, 0));
        self.info_ausgewaehlt = false;
        self.loeschen_bestaetigen = false;
        self.geb_fehler    = false;
        self.inhalt_beim_speichern = String::new();
        self.geaendert = false;
        self.konflikte.clear();
        self.speichern_nach_konflikt = false;
        self.speichern_nach_schliessen = false;
        // matrix_modus und psychiatrie_modus absichtlich NICHT zurücksetzen —
        // das sind Nutzereinstellungen, die beim Leeren der Station bestehen bleiben.
        self.hyprgross_aktiv = false;
        self.hyprgross_bett_pos = 0;
        self.hyprgross_feld = 0;
        self.hyprgross_ansicht = HyprGrossAnsicht::Feld;
    }

    fn zimmer_anzahl_setzen(&mut self, neue_anzahl: usize) {
        let aktuelle = self.station.zimmer.len();
        if neue_anzahl > aktuelle {
            let max_nr = self.station.zimmer.iter()
                .filter_map(|z| z.nummer.parse::<u32>().ok())
                .max()
                .unwrap_or(100);
            for i in 0..(neue_anzahl - aktuelle) {
                self.station.zimmer.push(Zimmer {
                    nummer: format!("{}", max_nr + 1 + i as u32),
                    betten: vec![
                        Bett { buchstabe: "A".into(), patient: None },
                        Bett { buchstabe: "B".into(), patient: None },
                    ],
                });
            }
        } else if neue_anzahl < aktuelle {
            self.station.zimmer.truncate(neue_anzahl);
            // Veraltete Indizes korrigieren
            if let Some((zi, _)) = self.ausgewaehlte_karte {
                if zi >= neue_anzahl {
                    self.ausgewaehlte_karte = if neue_anzahl > 0 { Some((neue_anzahl - 1, 0)) } else { None };
                }
            }
            if let Some((zi, _)) = self.bearbeitung {
                if zi >= neue_anzahl { self.bearbeitung = None; }
            }
        }
    }

    // ── Markdown-Serialisierung ──────────────────────────────────────────────

    fn markdown_erstellen(&self) -> String {
        self.markdown_erstellen_mit(
            self.letzte_aenderung_am.as_deref(),
            self.letzte_aenderung_von.as_deref(),
        )
    }

    // Baut den Markdown-Inhalt mit expliziten Zeitstempeln — nützlich, wenn
    // noch nicht feststeht, ob der Speichervorgang überhaupt durchgeführt wird
    // (z.B. während ein Save-As-Dialog noch offen ist).
    fn markdown_erstellen_mit(&self, am: Option<&str>, von: Option<&str>) -> String {
        let mut md = String::new();
        md.push_str("# MZ-HyprNurs\n\n");
        md.push_str(&format!("**Station:** {}\n", self.station_name));
        md.push_str(&format!("**HyprInfo:** {}\n", self.station_hyprinfo));
        if let Some(am) = am {
            md.push_str(&format!("**Letzte Aenderung am:** {}\n", am));
        }
        if let Some(von) = von {
            md.push_str(&format!("**Letzte Aenderung von:** {}\n", von));
        }
        if self.matrix_modus {
            md.push_str("**Matrix:** true\n");
        }
        if self.psychiatrie_modus {
            md.push_str("**Psychiatrie:** true\n");
        }

        if !self.station.dienst_info.is_empty() {
            md.push_str("\n## Dienstinfo\n\n");
            md.push_str(&self.station.dienst_info.replace('\n', "\\n"));
            md.push('\n');
        }

        for zimmer in &self.station.zimmer {
            md.push_str(&format!("\n## Zimmer {}\n", zimmer.nummer));
            for bett in &zimmer.betten {
                md.push_str(&format!("\n### Bett {}\n\n", bett.buchstabe));
                match &bett.patient {
                    None => { md.push_str("(leer)\n"); }
                    Some(pat) => {
                        let alle_leer = pat.nachname.is_empty() && pat.vorname.is_empty()
                            && pat.geburtsdatum.is_empty() && pat.besonderheiten.is_empty()
                            && pat.hdia.is_empty() && pat.ndia.is_empty()
                            && pat.info.is_empty() && pat.pflege.is_empty() && pat.todo.is_empty();
                        if alle_leer {
                            md.push_str("(leer)\n");
                        } else {
                            if !pat.nachname.is_empty()         { md.push_str(&format!("**Nachname:** {}\n",                   pat.nachname)); }
                            if !pat.vorname.is_empty()          { md.push_str(&format!("**Vorname:** {}\n",                    pat.vorname)); }
                            if !pat.geburtsdatum.is_empty()     { md.push_str(&format!("**Geburtsdatum:** {}\n",               pat.geburtsdatum)); }
                            if !pat.anrede.is_empty()           { md.push_str(&format!("**Anrede:** {}\n",                     pat.anrede)); }
                            if !pat.bio_geschlecht.is_empty()   { md.push_str(&format!("**Biologisches Geschlecht:** {}\n",    pat.bio_geschlecht)); }
                            if !pat.besonderheiten.is_empty()   { md.push_str(&format!("**Besonderheiten:** {}\n",             pat.besonderheiten)); }
                            if !pat.hdia.is_empty() {
                                let key = if self.psychiatrie_modus { "PDIA" } else { "Hauptdiagnose" };
                                md.push_str(&format!("**{}:** {}\n", key, pat.hdia.replace('\n', "\\n")));
                            }
                            if !pat.ndia.is_empty() {
                                let key = if self.psychiatrie_modus { "SDIA" } else { "Nebendiagnose" };
                                md.push_str(&format!("**{}:** {}\n", key, pat.ndia.replace('\n', "\\n")));
                            }
                            if !pat.pflege.is_empty()        { md.push_str(&format!("**Pflege:** {}\n",        pat.pflege.replace('\n', "\\n"))); }
                            if !pat.info.is_empty()          { md.push_str(&format!("**Info:** {}\n",          pat.info.replace('\n', "\\n"))); }
                            if !pat.todo.is_empty()          { md.push_str(&format!("**ToDo:** {}\n",          pat.todo.replace('\n', "\\n"))); }
                        }
                    }
                }
            }
        }
        md
    }

    fn markdown_parsen(&mut self, content: &str) {
        self.station_name = String::new();
        self.station_hyprinfo = String::new();
        self.station.dienst_info = String::new();
        self.letzte_aenderung_am  = None;
        self.letzte_aenderung_von = None;
        self.psychiatrie_modus    = false;
        // Station komplett aus Datei aufbauen – alte Zimmer verwerfen
        self.station.zimmer.clear();
        self.bearbeitung = None;
        self.ausgewaehlte_karte = Some((0, 0));

        #[derive(PartialEq)]
        enum Abschnitt { Header, Dienstinfo, Zimmer, Bett }

        let mut abschnitt = Abschnitt::Header;
        let mut akt_zimmer_nr: Option<String> = None;
        let mut akt_bett_buchstabe: Option<String> = None;
        let mut dienstinfo_zeilen: Vec<&str> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("## Zimmer ") {
                if abschnitt == Abschnitt::Dienstinfo {
                    self.station.dienst_info = dienstinfo_zeilen.join("\n").replace("\\n", "\n").trim().to_string();
                    dienstinfo_zeilen.clear();
                }
                let nr = trimmed.trim_start_matches("## Zimmer ").to_string();
                // Zimmer aus Datei übernehmen (bei Duplikat: Betten leeren)
                if let Some(zi) = self.station.zimmer.iter().position(|z| z.nummer == nr) {
                    self.station.zimmer[zi].betten.clear();
                } else {
                    self.station.zimmer.push(Zimmer {
                        nummer: nr.clone(),
                        betten: Vec::new(),
                    });
                }
                akt_zimmer_nr = Some(nr);
                akt_bett_buchstabe = None;
                abschnitt = Abschnitt::Zimmer;
                continue;
            }
            if trimmed == "## Dienstinfo" {
                abschnitt = Abschnitt::Dienstinfo;
                continue;
            }
            if trimmed.starts_with("### Bett ") {
                let buchstabe = trimmed.trim_start_matches("### Bett ").to_string();
                if let Some(ref nr) = akt_zimmer_nr {
                    if let Some(zi) = self.station.zimmer.iter().position(|z| z.nummer == *nr) {
                        if self.station.zimmer[zi].betten.iter().all(|b| b.buchstabe != buchstabe) {
                            self.station.zimmer[zi].betten.push(Bett { buchstabe: buchstabe.clone(), patient: None });
                        }
                    }
                }
                akt_bett_buchstabe = Some(buchstabe);
                abschnitt = Abschnitt::Bett;
                continue;
            }

            match abschnitt {
                Abschnitt::Header => {
                    if let Some(v) = trimmed.strip_prefix("**Station:** ")            { self.station_name = v.to_string(); }
                    else if let Some(v) = trimmed.strip_prefix("**HyprInfo:** ")       { self.station_hyprinfo = v.to_string(); }
                    else if let Some(v) = trimmed.strip_prefix("**Letzte Aenderung am:** ")  { self.letzte_aenderung_am = Some(v.to_string()); }
                    else if let Some(v) = trimmed.strip_prefix("**Letzte Aenderung von:** ") { self.letzte_aenderung_von = Some(v.to_string()); }
                    else if trimmed == "**Matrix:** true"       { self.matrix_modus = true; }
                    else if trimmed == "**Matrix:** false"      { self.matrix_modus = false; }
                    else if trimmed == "**Psychiatrie:** true"  { self.psychiatrie_modus = true; }
                    else if trimmed == "**Psychiatrie:** false" { self.psychiatrie_modus = false; }
                }
                Abschnitt::Dienstinfo => {
                    dienstinfo_zeilen.push(line);
                }
                Abschnitt::Bett => {
                    if trimmed.is_empty() || trimmed == "(leer)" || trimmed == "[leer]" { continue; }
                    if let (Some(ref nr), Some(ref buchstabe)) = (&akt_zimmer_nr, &akt_bett_buchstabe) {
                        if let Some(zi) = self.station.zimmer.iter().position(|z| z.nummer == *nr) {
                            if let Some(bi) = self.station.zimmer[zi].betten.iter().position(|b| b.buchstabe == *buchstabe) {
                                if self.station.zimmer[zi].betten[bi].patient.is_none() {
                                    self.station.zimmer[zi].betten[bi].patient = Some(Patient::default());
                                }
                                if let Some(pat) = &mut self.station.zimmer[zi].betten[bi].patient {
                                    if let Some(v) = trimmed.strip_prefix("**Nachname:** ")          { pat.nachname         = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Vorname:** ")       { pat.vorname          = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Geburtsdatum:** ")  { pat.geburtsdatum     = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Anrede:** ")        { pat.anrede           = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Biologisches Geschlecht:** ") { pat.bio_geschlecht = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Besonderheiten:** "){ pat.besonderheiten   = v.to_string(); }
                                    else if let Some(v) = trimmed.strip_prefix("**Hauptdiagnose:** ") { pat.hdia = v.replace("\\n", "\n"); }
                                    else if let Some(v) = trimmed.strip_prefix("**Nebendiagnose:** ") { pat.ndia = v.replace("\\n", "\n"); }
                                    else if let Some(v) = trimmed.strip_prefix("**PDIA:** ") { pat.hdia = v.replace("\\n", "\n"); self.psychiatrie_modus = true; }
                                    else if let Some(v) = trimmed.strip_prefix("**SDIA:** ") { pat.ndia = v.replace("\\n", "\n"); }
                                    else if let Some(v) = trimmed.strip_prefix("**Info:** ")          { pat.info             = v.replace("\\n", "\n"); }
                                    else if let Some(v) = trimmed.strip_prefix("**Pflege:** ")        { pat.pflege           = v.replace("\\n", "\n"); }
                                    else if let Some(v) = trimmed.strip_prefix("**ToDo:** ")          { pat.todo             = v.replace("\\n", "\n"); }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if abschnitt == Abschnitt::Dienstinfo && !dienstinfo_zeilen.is_empty() {
            self.station.dienst_info = dienstinfo_zeilen.join("\n").replace("\\n", "\n").trim().to_string();
        }
        // Betten alphabetisch nach Buchstabe sortieren (A vor B). Bewahrt die
        // physische Reihenfolge unabhängig davon, welches Bett belegt ist.
        for zimmer in &mut self.station.zimmer {
            zimmer.betten.sort_by(|a, b| a.buchstabe.cmp(&b.buchstabe));
        }
    }

    // ── Speichern / Laden ────────────────────────────────────────────────────

    fn hat_duplikate(&self) -> bool {
        for (zi, z) in self.station.zimmer.iter().enumerate() {
            if z.nummer.trim().is_empty() { return true; }
            for (bi, b) in z.betten.iter().enumerate() {
                if b.buchstabe.trim().is_empty() { return true; }
                let duplikat = self.station.zimmer.iter().enumerate().any(|(zii, zz)| {
                    zz.betten.iter().enumerate().any(|(bii, bb)| {
                        (zii != zi || bii != bi) && zz.nummer == z.nummer && bb.buchstabe == b.buchstabe
                    })
                });
                if duplikat { return true; }
            }
        }
        false
    }

    fn konflikte_pruefen(&self, disk_inhalt: &str) -> Vec<Konflikt> {
        let disk = betten_aus_markdown(disk_inhalt);
        let normalisieren = |p: Option<Patient>| -> Option<Patient> {
            match p {
                None => None,
                Some(ref pat) if pat.nachname.is_empty() && pat.vorname.is_empty()
                    && pat.geburtsdatum.is_empty() && pat.anrede.is_empty() && pat.bio_geschlecht.is_empty()
                    && pat.besonderheiten.is_empty()
                    && pat.hdia.is_empty() && pat.ndia.is_empty()
                    && pat.info.is_empty() && pat.pflege.is_empty() && pat.todo.is_empty() => None,
                other => other,
            }
        };
        let mut liste = Vec::new();
        for (zi, zimmer) in self.station.zimmer.iter().enumerate() {
            for (bi, bett) in zimmer.betten.iter().enumerate() {
                let key = (zimmer.nummer.clone(), bett.buchstabe.clone());
                if let Some(disk_patient) = disk.get(&key) {
                    let lokal_norm = normalisieren(bett.patient.clone());
                    let disk_norm  = normalisieren(disk_patient.clone());
                    if lokal_norm != disk_norm {
                        liste.push(Konflikt {
                            zi, bi,
                            zimmer_nr:      zimmer.nummer.clone(),
                            bett_buchstabe: bett.buchstabe.clone(),
                            lokal: bett.patient.clone(),
                            disk:  disk_patient.clone(),
                        });
                    }
                }
            }
        }
        liste
    }

    fn tatsaechlich_speichern(&mut self) {
        // Trim auf alle relevanten Felder
        for zimmer in &mut self.station.zimmer {
            zimmer.nummer = zimmer.nummer.trim().to_string();
            for bett in &mut zimmer.betten {
                bett.buchstabe = bett.buchstabe.trim().to_string();
                if let Some(pat) = &mut bett.patient {
                    pat.hdia   = pat.hdia.trim().to_string();
                    pat.ndia   = pat.ndia.trim().to_string();
                    pat.info   = pat.info.trim().to_string();
                    pat.pflege = pat.pflege.trim().to_string();
                    pat.todo   = pat.todo.trim().to_string();
                }
            }
        }
        // Zeitstempel erst hier setzen, damit bei einem abgebrochenen Dialog
        // der RAM-Stand nicht bereits "gespeichert" aussieht.
        self.letzte_aenderung_am  = Some(jetzt_formatiert());
        self.letzte_aenderung_von = Some(benutzer_info());
        let content = self.markdown_erstellen();
        if let Some(ref path) = self.speicher_pfad.clone() {
            backup_erstellen(path);
            if atomar_schreiben(path, content.as_bytes()).is_ok() {
                self.inhalt_beim_speichern = content;
                self.geaendert = false;
                self.letzte_auto_speicherung = std::time::Instant::now();
            }
        }
    }

    fn speichern(&mut self) {
        if self.hat_duplikate() { return; }
        // Kein Pfad noch → Speichern-Dialog öffnen (kein Konflikt-Check nötig)
        if self.speicher_pfad.is_none() {
            // Zeitstempel lokal berechnen, NICHT self mutieren.
            // Falls der Dialog abgebrochen wird, bleibt self unverändert
            // (keine "gespeichert am"-Anzeige ohne tatsächliches Speichern).
            let am  = jetzt_formatiert();
            let von = benutzer_info();
            let content = self.markdown_erstellen_mit(Some(&am), Some(&von));
            let filename = self.dateinamen_erstellen();
            let (tx, rx) = mpsc::channel();
            self.dialog_rx = Some(rx);
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&filename)
                    .add_filter("Markdown", &["md"])
                    .save_file()
                {
                    backup_erstellen(&path);
                    if atomar_schreiben(&path, content.as_bytes()).is_ok() {
                        let _ = tx.send(DialogErgebnis::Speichern {
                            pfad: path, inhalt: content, am, von,
                        });
                    }
                }
            });
            return;
        }
        // Pfad bekannt → Disk auf Konflikte prüfen
        if let Some(ref path) = self.speicher_pfad.clone() {
            if let Ok(disk_inhalt) = std::fs::read_to_string(path) {
                // Nur prüfen wenn jemand anderes die Datei verändert hat
                if disk_inhalt != self.inhalt_beim_speichern {
                    let konflikte = self.konflikte_pruefen(&disk_inhalt);
                    if !konflikte.is_empty() {
                        self.konflikte = konflikte;
                        self.speichern_nach_konflikt = true;
                        return;
                    }
                }
            }
        }
        self.tatsaechlich_speichern();
    }

    fn speichern_unter(&mut self) {
        // Zeitstempel lokal — self wird erst bei erfolgreichem Schreiben
        // durch den Dialog-Ergebnis-Handler aktualisiert.
        let am  = jetzt_formatiert();
        let von = benutzer_info();
        let content = self.markdown_erstellen_mit(Some(&am), Some(&von));
        let filename = self.dateinamen_erstellen();
        let (tx, rx) = mpsc::channel();
        self.dialog_rx = Some(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&filename)
                .add_filter("Markdown", &["md"])
                .save_file()
            {
                backup_erstellen(&path);
                if atomar_schreiben(&path, content.as_bytes()).is_ok() {
                    let _ = tx.send(DialogErgebnis::Speichern {
                        pfad: path, inhalt: content, am, von,
                    });
                }
            }
        });
    }

    fn stilles_speichern(&mut self) {
        if self.hat_duplikate() { return; }
        if self.speicher_pfad.is_none() { return; }
        if let Some(ref path) = self.speicher_pfad.clone() {
            if let Ok(disk_inhalt) = std::fs::read_to_string(path) {
                // Nur prüfen wenn jemand anderes die Datei verändert hat
                if disk_inhalt != self.inhalt_beim_speichern {
                    let konflikte = self.konflikte_pruefen(&disk_inhalt);
                    if !konflikte.is_empty() {
                        self.konflikte = konflikte;
                        self.speichern_nach_konflikt = true;
                        return;
                    }
                }
            }
        }
        self.tatsaechlich_speichern();
    }

    fn laden(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.dialog_rx = Some(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Markdown", &["md"])
                .pick_file()
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let _ = tx.send(DialogErgebnis::Laden(path, content));
                }
            }
        });
    }

    // ── PDF-Export ───────────────────────────────────────────────────────────

    fn schrift_laden_bytes(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        #[cfg(target_os = "linux")]
        {
            let paare = [
                ("/usr/share/fonts/liberation/LiberationSans-Regular.ttf",           "/usr/share/fonts/liberation/LiberationSans-Bold.ttf"),
                ("/usr/share/fonts/TTF/LiberationSans-Regular.ttf",                  "/usr/share/fonts/TTF/LiberationSans-Bold.ttf"),
                ("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",  "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf"),
                ("/usr/share/fonts/TTF/DejaVuSans.ttf",                              "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf"),
                ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",                  "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
            ];
            for (r, b) in paare {
                if let (Ok(rb), Ok(bb)) = (std::fs::read(r), std::fs::read(b)) {
                    return Some((rb, bb));
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            let paare = [
                ("/System/Library/Fonts/Supplemental/Arial.ttf",    "/System/Library/Fonts/Supplemental/Arial Bold.ttf"),
                ("/System/Library/Fonts/Supplemental/Verdana.ttf",  "/System/Library/Fonts/Supplemental/Verdana Bold.ttf"),
                ("/System/Library/Fonts/Supplemental/Georgia.ttf",  "/System/Library/Fonts/Supplemental/Georgia Bold.ttf"),
            ];
            for (r, b) in paare {
                if let (Ok(rb), Ok(bb)) = (std::fs::read(r), std::fs::read(b)) {
                    return Some((rb, bb));
                }
            }
        }
        #[cfg(windows)]
        {
            let paare = [
                ("C:\\Windows\\Fonts\\arial.ttf",   "C:\\Windows\\Fonts\\arialbd.ttf"),
                ("C:\\Windows\\Fonts\\verdana.ttf", "C:\\Windows\\Fonts\\verdanab.ttf"),
                ("C:\\Windows\\Fonts\\calibri.ttf", "C:\\Windows\\Fonts\\calibrib.ttf"),
            ];
            for (r, b) in paare {
                if let (Ok(rb), Ok(bb)) = (std::fs::read(r), std::fs::read(b)) {
                    return Some((rb, bb));
                }
            }
        }
        None
    }

    fn pdf_exportieren(&mut self) {
        self.stilles_speichern();
        let Some(bytes) = self.schrift_laden_bytes() else { return; };
        let dateiname = {
            let name: String = self.station_name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect();
            format!("MZ-HYPRNURS_{}_{}.pdf", name, Self::jetzt_zeitstempel())
        };
        let (tx, rx) = mpsc::channel();
        self.dialog_rx = Some(rx);
        self.ausstehende_pdf_schrift = Some(bytes);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&dateiname)
                .add_filter("PDF", &["pdf"])
                .save_file()
            {
                let _ = tx.send(DialogErgebnis::PdfExport(path));
            }
        });
    }

    fn pdf_generieren(&self, pfad: &std::path::Path, reg_bytes: Vec<u8>, bold_bytes: Vec<u8>) {
        use printpdf::*;

        // ── Hilfsfunktionen ─────────────────────────────────────────────────

        // Geschlossenes Rechteck. gefuellt=true → nur Füllung, sonst nur Kontur.
        fn kasten(x: f64, y: f64, w: f64, h: f64, gefuellt: bool) -> Line {
            Line {
                points: vec![
                    (Point::new(Mm(x),     Mm(y)),     false),
                    (Point::new(Mm(x + w), Mm(y)),     false),
                    (Point::new(Mm(x + w), Mm(y + h)), false),
                    (Point::new(Mm(x),     Mm(y + h)), false),
                ],
                is_closed: true, has_fill: gefuellt, has_stroke: !gefuellt,
                is_clipping_path: false,
            }
        }

        fn linie(x1: f64, y1: f64, x2: f64, y2: f64) -> Line {
            Line {
                points: vec![
                    (Point::new(Mm(x1), Mm(y1)), false),
                    (Point::new(Mm(x2), Mm(y2)), false),
                ],
                is_closed: false, has_fill: false, has_stroke: true,
                is_clipping_path: false,
            }
        }

        // Zeilenumbruch
        fn text_umbrechen(text: &str, max_mm: f64, pt: f64, max_zeilen: usize) -> Vec<String> {
            let avg_mm   = (0.47 * pt * 0.353_f64).max(0.5);
            let max_chars = ((max_mm / avg_mm) as usize).max(1);
            let mut result: Vec<String> = Vec::new();
            'outer: for raw_line in text.lines() {
                let mut current = String::new();
                for word in raw_line.split_whitespace() {
                    if result.len() >= max_zeilen { break 'outer; }
                    if current.is_empty() {
                        current = word.to_string();
                    } else if current.len() + 1 + word.len() <= max_chars {
                        current.push(' ');
                        current.push_str(word);
                    } else {
                        result.push(std::mem::take(&mut current));
                        if result.len() >= max_zeilen { break 'outer; }
                        current = word.to_string();
                    }
                }
                if !current.is_empty() && result.len() < max_zeilen {
                    result.push(current);
                }
            }
            result
        }

        // ── Farben ──────────────────────────────────────────────────────────
        let schwarz    = Color::Rgb(Rgb { r: 0.0,  g: 0.0,  b: 0.0,  icc_profile: None });
        let weiss      = Color::Rgb(Rgb { r: 1.0,  g: 1.0,  b: 1.0,  icc_profile: None });
        let dunkelgrau = Color::Rgb(Rgb { r: 0.18, g: 0.18, b: 0.18, icc_profile: None });
        let hellgrau   = Color::Rgb(Rgb { r: 0.72, g: 0.72, b: 0.72, icc_profile: None });

        // ── Datum + Uhrzeit ──────────────────────────────────────────────────
        let datum_str = format!(
            "Druckdatum: {}",
            chrono::Local::now().format("%d.%m.%Y %H:%M:%S Uhr")
        );

        // ── Dokument (dynamische Seitenanzahl: 6 Zimmer pro Seite) ──────────
        let n_seiten = self.station.zimmer.len().div_ceil(6).max(1);
        let (doc, erste_s, erste_e) = PdfDocument::new(
            format!("Übergabe {}", self.station_name),
            Mm(210.0), Mm(297.0), "Ebene 1",
        );
        let mut seiten = vec![(erste_s, erste_e)];
        for _ in 1..n_seiten {
            seiten.push(doc.add_page(Mm(210.0), Mm(297.0), "Ebene 1"));
        }

        let font_reg  = match doc.add_external_font(std::io::Cursor::new(&reg_bytes))  { Ok(f) => f, Err(_) => return };
        let font_bold = match doc.add_external_font(std::io::Cursor::new(&bold_bytes)) { Ok(f) => f, Err(_) => return };

        // ── Layout-Konstanten (mm) ───────────────────────────────────────────
        const RAND_L:    f64 = 10.0;
        const RAND_R:    f64 = 10.0;
        const RAND_O:    f64 = 12.0;  // eng zum Blattrand → mehr Platz für Felder
        const RAND_U:    f64 = 14.0;
        const GAP_B:     f64 =  4.0;
        const GAP_H:     f64 =  1.5;
        const ZELLE_B:   f64 = 15.0;  // Zimmer-Zelle = Bett-Zelle = Label-Spalte
        const KOPF_HH:   f64 =  6.0;  // Kopfzeile (Zimmer | Bett | Name+Alter / Besonderheiten)
        const N_FELDER:  usize = 5;
        const HEADER_H:  f64 =  6.5;  // Station + HyprInfo (beide Seiten)
        const DIENST_H:  f64 = 18.0;  // Dienstinfo-Block (beide Seiten, über Fußzeile)
        const TEXT_PT:   f64 =  7.0;  // Schriftgröße Feldinhalt
        const ZEILENABS: f64 =  2.15; // Zeilenabstand im Feldinhalt (mm)

        let nutz_b = 210.0 - RAND_L - RAND_R;
        let box_b  = (nutz_b - GAP_B) / 2.0;
        let felder = if self.psychiatrie_modus {
            ["PDIA", "SDIA", "NURS", "INFO", "TODO"]
        } else {
            ["HDIA", "NDIA", "NURS", "INFO", "TODO"]
        };

        // Beide Seiten identisch: Header + 6 Boxen + Dienstinfo + Fußzeile
        let usable_h = 297.0 - RAND_O - RAND_U;
        let h_for_zi = usable_h - HEADER_H - DIENST_H;
        let box_h    = (h_for_zi - 5.0 * GAP_H) / 6.0;
        let feld_h   = (box_h - KOPF_HH) / N_FELDER as f64;
        // Wie viele Zeilen passen? (erste Zeile 1.5mm Abstand von Feld-Oberkante)
        let max_zeilen = ((feld_h - 1.5) / ZEILENABS).floor() as usize;

        // ── Seiten zeichnen ──────────────────────────────────────────────────
        for (seite_idx, &(s_id, e_id)) in seiten.iter().enumerate() {
            let ebene = doc.get_page(s_id).get_layer(e_id);
            let zi_start = seite_idx * 6;
            let seite_nr = (seite_idx + 1) as u8;

            // ── Stationsheader (beide Seiten) ────────────────────────────────
            let name_y = 297.0 - RAND_O + 2.0;
            ebene.set_fill_color(schwarz.clone());
            ebene.use_text(&self.station_name, 8.5, Mm(RAND_L), Mm(name_y), &font_bold);
            if !self.station_hyprinfo.is_empty() {
                let avg9 = 0.60 * 8.5 * 0.353_f64;
                let name_mm  = self.station_name.len() as f64 * avg9;
                let hyprinfo_x = (RAND_L + name_mm + 5.0).min(RAND_L + 50.0);
                let hyprinfo_kurz: String = self.station_hyprinfo.chars().take(70).collect();
                ebene.use_text(&hyprinfo_kurz, 6.5, Mm(hyprinfo_x), Mm(name_y), &font_reg);
            }

            // ── 6 Zimmer-Boxen ───────────────────────────────────────────────
            let top_start = 297.0 - RAND_O - HEADER_H;

            for si in 0..6_usize {
                let z_idx  = zi_start + si;
                let zimmer = if let Some(z) = self.station.zimmer.get(z_idx) { z } else { continue };
                let box_top = top_start - si as f64 * (box_h + GAP_H);
                let box_y   = box_top - box_h;

                for bi in 0..2_usize {
                    let leer  = Bett { buchstabe: if bi == 0 { "A".into() } else { "B".into() }, patient: None };
                    let bett  = zimmer.betten.get(bi).unwrap_or(&leer);
                    let box_x = RAND_L + bi as f64 * (box_b + GAP_B);

                    // Weißer Kartenhintergrund (kein Schatten)
                    ebene.set_fill_color(weiss.clone());
                    ebene.add_shape(kasten(box_x, box_y, box_b, box_h, true));

                    // Kartenrahmen (zuerst, damit Zellen ihn überdecken)
                    ebene.set_outline_color(hellgrau.clone());
                    ebene.set_outline_thickness(0.3);
                    ebene.add_shape(kasten(box_x, box_y, box_b, box_h, false));

                    // Zimmer-Zelle links (mit Outline in gleicher Farbe, überdeckt Rahmen)
                    let kopf_unten = box_y + box_h - KOPF_HH;
                    ebene.set_fill_color(dunkelgrau.clone());
                    ebene.set_outline_color(dunkelgrau.clone());
                    ebene.set_outline_thickness(0.5);
                    ebene.add_shape(kasten(box_x, kopf_unten, ZELLE_B, KOPF_HH, true));

                    // Bett-Zelle rechts (mit Outline in gleicher Farbe, überdeckt Rahmen)
                    ebene.set_fill_color(dunkelgrau.clone());
                    ebene.set_outline_color(dunkelgrau.clone());
                    ebene.set_outline_thickness(0.5);
                    ebene.add_shape(kasten(box_x + box_b - ZELLE_B, kopf_unten, ZELLE_B, KOPF_HH, true));

                    // Text Kopfzeile
                    let kopf_zelle_y = kopf_unten + KOPF_HH / 2.0 - 0.9; // zentriert in dunkler Zelle
                    let zeile1_y     = kopf_unten + 3.3;                  // Name+Alter, ~1mm Abstand oben
                    let zeile2_y     = kopf_unten + 1.14;                 // Besonderheiten, ~1mm Abstand unten
                    ebene.set_fill_color(weiss.clone());

                    let zim_avg = 0.60 * 7.0 * 0.353_f64;
                    let zim_w   = zimmer.nummer.len() as f64 * zim_avg;
                    ebene.use_text(&zimmer.nummer, 7.0, Mm(box_x + ZELLE_B / 2.0 - zim_w / 2.0), Mm(kopf_zelle_y), &font_bold);

                    let bett_w = bett.buchstabe.len() as f64 * zim_avg;
                    ebene.use_text(&bett.buchstabe, 7.0, Mm(box_x + box_b - ZELLE_B + ZELLE_B / 2.0 - bett_w / 2.0), Mm(kopf_zelle_y), &font_bold);

                    if let Some(pat) = &bett.patient {
                        // Zeile 1: Anrede + Nachname + [*Alter, Bio] (kein Vorname im PDF)
                        let suffix = alter_bio_suffix(&pat.geburtsdatum, &pat.bio_geschlecht);
                        let anrede = if !pat.anrede.is_empty() && pat.anrede != "–" {
                            format!("{} ", pat.anrede)
                        } else { String::new() };
                        let name_alter = format!("{}{}{}", anrede, pat.nachname.trim(), suffix);
                        if !name_alter.is_empty() {
                            let name_x   = box_x + ZELLE_B + 2.0;
                            let name_max = box_b - ZELLE_B * 2.0 - 3.0;
                            let avg_n    = 0.55 * 7.0 * 0.353_f64;
                            let max_c    = ((name_max / avg_n) as usize).max(1);
                            let kurz: String = name_alter.chars().take(max_c).collect();
                            ebene.set_fill_color(schwarz.clone());
                            ebene.use_text(&kurz, 7.0, Mm(name_x), Mm(zeile1_y), &font_bold);
                        }

                        // Zeile 2: Besonderheiten mit *fett*-Support
                        let besond = pat.besonderheiten.trim();
                        if !besond.is_empty() {
                            let besond_x_start = box_x + ZELLE_B + 2.0;
                            let besond_max     = box_b - ZELLE_B * 2.0 - 3.0;
                            let avg_b          = 0.55 * 5.5 * 0.353_f64;
                            let mut seg_x      = besond_x_start;
                            ebene.set_fill_color(schwarz.clone());
                            for (seg, ist_fett) in stern_segmente(besond) {
                                let verbl = besond_max - (seg_x - besond_x_start);
                                if verbl <= 0.0 { break; }
                                let max_c = ((verbl / avg_b) as usize).max(1);
                                let seg_kurz: String = seg.chars().take(max_c).collect();
                                let font_s = if ist_fett { &font_bold } else { &font_reg };
                                ebene.use_text(&seg_kurz, 5.5, Mm(seg_x), Mm(zeile2_y), font_s);
                                seg_x += seg_kurz.chars().count() as f64 * avg_b;
                            }
                        }
                    }

                    // Trennlinie Kopf / Felder
                    ebene.set_outline_color(hellgrau.clone());
                    ebene.set_outline_thickness(0.35);
                    ebene.add_shape(linie(box_x, kopf_unten, box_x + box_b, kopf_unten));

                    // Senkrechte Label/Inhalt-Trennlinie
                    ebene.set_outline_color(hellgrau.clone());
                    ebene.set_outline_thickness(0.2);
                    ebene.add_shape(linie(box_x + ZELLE_B, box_y + 0.5, box_x + ZELLE_B, kopf_unten - 0.3));

                    // Felder
                    let fwerte: [&str; 5] = if let Some(pat) = &bett.patient {
                        [pat.hdia.as_str(), pat.ndia.as_str(), pat.pflege.as_str(),
                         pat.info.as_str(), pat.todo.as_str()]
                    } else { ["", "", "", "", ""] };

                    for (fi, (fname, fwert)) in felder.iter().zip(fwerte.iter()).enumerate() {
                        let feld_unten = box_y + box_h - KOPF_HH - (fi as f64 + 1.0) * feld_h;
                        let feld_mitte = feld_unten + feld_h / 2.0;

                        // Horizontale Trennlinie
                        if fi < N_FELDER - 1 {
                            ebene.set_outline_color(hellgrau.clone());
                            ebene.set_outline_thickness(0.12);
                            ebene.add_shape(linie(box_x + 1.0, feld_unten, box_x + box_b - 1.0, feld_unten));
                        }

                        // Label: zentriert horizontal + vertikal
                        let lbl_avg = 0.62 * 7.0 * 0.353_f64;
                        let lbl_w   = fname.len() as f64 * lbl_avg;
                        let lbl_x   = box_x + ZELLE_B / 2.0 - lbl_w / 2.0;
                        ebene.set_fill_color(hellgrau.clone());
                        ebene.use_text(*fname, 7.0, Mm(lbl_x), Mm(feld_mitte - 0.9), &font_bold);

                        // Wert: so viele Zeilen wie passen, *…* → fett (konsistent
                        // zu UI und Besonderheiten-Rendering). Feldweise fett oder normal,
                        // keine Segment-Mischung innerhalb einer Zeile.
                        if !fwert.is_empty() {
                            let hat_bold_mark = fwert.contains('*');
                            let text_clean: String = fwert.replace('*', "");
                            let inhalt_mm = box_b - ZELLE_B - 2.0;
                            let zeilen    = text_umbrechen(&text_clean, inhalt_mm, TEXT_PT, max_zeilen.max(3));
                            let font_v    = if hat_bold_mark { &font_bold } else { &font_reg };
                            let zeile1_y  = feld_unten + feld_h - 2.02;
                            for (li, zeile) in zeilen.iter().enumerate() {
                                let line_y = zeile1_y - li as f64 * ZEILENABS;
                                if line_y > feld_unten + 0.2 {
                                    ebene.set_fill_color(schwarz.clone());
                                    ebene.use_text(zeile.as_str(), TEXT_PT,
                                        Mm(box_x + ZELLE_B + 0.47), Mm(line_y), font_v);
                                }
                            }
                        }
                    }
                }
            }

            // ── Dienstinfo (beide Seiten, unterhalb der Zimmer) ──────────────
            let dienst_start_y = RAND_U + DIENST_H;
            ebene.set_fill_color(schwarz.clone());
            ebene.use_text("WICHTIGE INFORMATIONEN FÜR DEN DIENST",
                6.5, Mm(RAND_L), Mm(dienst_start_y - 4.0), &font_bold);
            if !self.station.dienst_info.is_empty() {
                let zeilen = text_umbrechen(&self.station.dienst_info, nutz_b, 6.5, 4);
                for (li, zeile) in zeilen.iter().enumerate() {
                    let z_y = dienst_start_y - 8.0 - li as f64 * 3.2;
                    if z_y > RAND_U {
                        ebene.use_text(zeile.as_str(), 6.5, Mm(RAND_L), Mm(z_y), &font_reg);
                    }
                }
            }

            // ── Fußzeile ─────────────────────────────────────────────────────
            let fuss_linie_y = RAND_U - 2.0;
            let fuss_text_y  = RAND_U - 5.5;
            ebene.set_outline_color(hellgrau.clone());
            ebene.set_outline_thickness(0.35);
            ebene.add_shape(linie(RAND_L, fuss_linie_y, 210.0 - RAND_R, fuss_linie_y));
            ebene.set_fill_color(schwarz.clone());
            ebene.use_text("MZ-HYPRNURS", 6.0, Mm(RAND_L), Mm(fuss_text_y), &font_bold);
            ebene.use_text(&datum_str, 6.0, Mm(105.0 - 23.0), Mm(fuss_text_y), &font_reg);
            let seite_str = format!("Seite {}/{}", seite_nr, n_seiten);
            let avg_pt    = 0.55 * 6.0 * 0.353_f64;
            let seite_mm  = seite_str.len() as f64 * avg_pt;
            ebene.use_text(&seite_str, 6.0, Mm(210.0 - RAND_R - seite_mm), Mm(fuss_text_y), &font_reg);
        }

        // ── Speichern ───────────────────────────────────────────────────────
        if let Ok(file) = std::fs::File::create(pfad) {
            let _ = doc.save(&mut std::io::BufWriter::new(file));
        }
    }

    // ── ODT-Export ───────────────────────────────────────────────────────────
    fn odt_exportieren(&mut self) {
        self.stilles_speichern();
        let dateiname = {
            let name: String = self.station_name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect();
            format!("MZ-HYPRNURS_{}_{}.odt", name, Self::jetzt_zeitstempel())
        };
        let (tx, rx) = mpsc::channel();
        self.dialog_rx = Some(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&dateiname)
                .add_filter("ODT", &["odt"])
                .save_file()
            {
                let _ = tx.send(DialogErgebnis::OdtExport(path));
            }
        });
    }

    fn odt_generieren(&self, pfad: &std::path::Path) {
        use std::io::Write;
        use zip::write::SimpleFileOptions;

        let datei = match std::fs::File::create(pfad) {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut zip = zip::ZipWriter::new(datei);

        let stored = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // mimetype muss erste Datei, unkomprimiert (ODT-Spezifikation)
        let _ = zip.start_file("mimetype", stored);
        let _ = zip.write_all(b"application/vnd.oasis.opendocument.text");

        let _ = zip.start_file("META-INF/manifest.xml", deflated);
        let _ = zip.write_all(Self::odt_manifest().as_bytes());

        let _ = zip.start_file("styles.xml", deflated);
        let _ = zip.write_all(Self::odt_styles().as_bytes());

        let content = self.odt_inhalt_erstellen();
        let _ = zip.start_file("content.xml", deflated);
        let _ = zip.write_all(content.as_bytes());

        let _ = zip.finish();
    }

    fn odt_manifest() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
  <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text" manifest:version="1.3"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#.to_string()
    }

    fn odt_styles() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles
  xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
  office:version="1.3">
  <office:styles>
    <style:style style:name="Standard" style:family="paragraph" style:class="text">
      <style:text-properties fo:font-size="11pt"/>
    </style:style>
  </office:styles>
  <office:automatic-styles>
    <style:page-layout style:name="Seite">
      <style:page-layout-properties fo:page-width="210mm" fo:page-height="297mm"
        fo:margin-top="10mm" fo:margin-bottom="10mm"
        fo:margin-left="10mm" fo:margin-right="10mm"/>
    </style:page-layout>
  </office:automatic-styles>
  <office:master-styles>
    <style:master-page style:name="Standard" style:page-layout-name="Seite"/>
  </office:master-styles>
</office:document-styles>"#.to_string()
    }

    fn odt_inhalt_erstellen(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        }
        fn p(stil: &str, inhalt: &str) -> String {
            format!("<text:p text:style-name=\"{}\">{}</text:p>", stil, inhalt)
        }

        let mut out = String::from(concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<office:document-content\n",
            "  xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\"\n",
            "  xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\"\n",
            "  xmlns:table=\"urn:oasis:names:tc:opendocument:xmlns:table:1.0\"\n",
            "  xmlns:style=\"urn:oasis:names:tc:opendocument:xmlns:style:1.0\"\n",
            "  xmlns:fo=\"urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0\"\n",
            "  office:version=\"1.3\">\n",
            "  <office:automatic-styles>\n",
            "    <style:style style:name=\"Standard\" style:family=\"paragraph\">\n",
            "      <style:text-properties fo:font-size=\"11pt\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"Titel\" style:family=\"paragraph\" style:parent-style-name=\"Standard\">\n",
            "      <style:text-properties fo:font-size=\"14pt\" fo:font-weight=\"bold\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"KopfText\" style:family=\"paragraph\">\n",
            "      <style:text-properties fo:font-size=\"11pt\" fo:font-weight=\"bold\" fo:color=\"#ffffff\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"LabelText\" style:family=\"paragraph\">\n",
            "      <style:text-properties fo:font-size=\"11pt\" fo:font-weight=\"bold\" fo:color=\"#888888\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"Fett\" style:family=\"text\">\n",
            "      <style:text-properties fo:font-weight=\"bold\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"TabBett\" style:family=\"table\">\n",
            "      <style:table-properties style:width=\"190mm\" table:align=\"margins\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"SpalteLabel\" style:family=\"table-column\">\n",
            "      <style:table-column-properties style:column-width=\"18mm\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"SpalteInhalt\" style:family=\"table-column\">\n",
            "      <style:table-column-properties style:column-width=\"172mm\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"ZelleKopf\" style:family=\"table-cell\">\n",
            "      <style:table-cell-properties fo:background-color=\"#1a1a1a\" fo:padding=\"1.5mm\" fo:border=\"0.5pt solid #1a1a1a\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"ZelleNormal\" style:family=\"table-cell\">\n",
            "      <style:table-cell-properties fo:padding=\"1mm\" fo:border-bottom=\"0.5pt solid #cccccc\" fo:border-left=\"0.5pt solid #cccccc\" fo:border-right=\"0.5pt solid #cccccc\" fo:border-top=\"none\"/>\n",
            "    </style:style>\n",
            "    <style:style style:name=\"ZelleLabelNormal\" style:family=\"table-cell\">\n",
            "      <style:table-cell-properties fo:padding=\"1mm\" fo:border-bottom=\"0.5pt solid #cccccc\" fo:border-left=\"0.5pt solid #cccccc\" fo:border-right=\"none\" fo:border-top=\"none\"/>\n",
            "    </style:style>\n",
            "  </office:automatic-styles>\n",
            "  <office:body>\n",
            "    <office:text>\n",
        ));

        // ── Kopf: Station + Stand + HyprInfo ─────────────────────────────────
        out.push_str(&p("Titel", &esc(&self.station_name)));
        {
            let stand = format!(
                "Stand: {}",
                chrono::Local::now().format("%d.%m.%Y @ %H:%M Uhr")
            );
            out.push_str(&p("Standard", &esc(&stand)));
        }
        if !self.station_hyprinfo.is_empty() {
            out.push_str(&p("Standard", &esc(&self.station_hyprinfo)));
        }
        out.push_str(&p("Standard", ""));

        // ── Patienten als Tabellen ───────────────────────────────────────────
        let alle_betten: Vec<(&crate::Zimmer, &crate::Bett)> = self.station.zimmer.iter()
            .flat_map(|z| z.betten.iter().map(move |b| (z, b)))
            .collect();

        let felder_labels: [&str; 5] = if self.psychiatrie_modus {
            ["PDIA", "SDIA", "NURS", "INFO", "TODO"]
        } else {
            ["HDIA", "NDIA", "NURS", "INFO", "TODO"]
        };

        for (tab_nr, (zimmer, bett)) in alle_betten.iter().enumerate() {
            // Kopfzeile: Zimmer | Bett | Name
            let kopf = match &bett.patient {
                Some(pat) => {
                    let suffix = alter_bio_suffix(&pat.geburtsdatum, &pat.bio_geschlecht);
                    let anrede = if !pat.anrede.is_empty() && pat.anrede != "–" {
                        format!("{} ", pat.anrede)
                    } else { String::new() };
                    format!("Zimmer {} | Bett {} | {}{}{}",
                        esc(&zimmer.nummer), esc(&bett.buchstabe),
                        esc(&anrede), esc(pat.nachname.trim()), esc(&suffix))
                }
                None => format!("Zi. {} | Bett {} | [leer]",
                    esc(&zimmer.nummer), esc(&bett.buchstabe)),
            };

            let felder_werte: [String; 5] = match &bett.patient {
                Some(pat) => [
                    pat.hdia.clone(), pat.ndia.clone(), pat.pflege.clone(),
                    pat.info.clone(), pat.todo.clone(),
                ],
                None => [String::new(), String::new(), String::new(), String::new(), String::new()],
            };

            // Besonderheiten in Kopfzeile einfügen falls vorhanden
            let besond = bett.patient.as_ref()
                .map(|p| p.besonderheiten.trim().to_string())
                .unwrap_or_default();

            let tab_name = format!("Bett{}", tab_nr);
            out.push_str(&format!(
                "<table:table table:name=\"{}\" table:style-name=\"TabBett\">",
                tab_name
            ));
            out.push_str("<table:table-column table:style-name=\"SpalteLabel\"/>");
            out.push_str("<table:table-column table:style-name=\"SpalteInhalt\"/>");

            // Kopfzeile (über beide Spalten)
            out.push_str("<table:table-row>");
            out.push_str("<table:table-cell table:style-name=\"ZelleKopf\" table:number-columns-spanned=\"2\" office:value-type=\"string\">");
            out.push_str(&p("KopfText", &kopf));
            if !besond.is_empty() {
                out.push_str(&p("KopfText", &esc(&besond)));
            }
            out.push_str("</table:table-cell>");
            out.push_str("<table:covered-table-cell/>");
            out.push_str("</table:table-row>");

            // 5 Feld-Zeilen
            for (fi, (label, wert)) in felder_labels.iter().zip(felder_werte.iter()).enumerate() {
                let _ = fi;
                let anzeige = if wert.is_empty() { "-".to_string() } else { esc(wert) };
                out.push_str("<table:table-row>");
                out.push_str("<table:table-cell table:style-name=\"ZelleLabelNormal\" office:value-type=\"string\">");
                out.push_str(&p("LabelText", label));
                out.push_str("</table:table-cell>");
                out.push_str("<table:table-cell table:style-name=\"ZelleNormal\" office:value-type=\"string\">");
                out.push_str(&p("Standard", &anzeige));
                out.push_str("</table:table-cell>");
                out.push_str("</table:table-row>");
            }

            out.push_str("</table:table>");
            out.push_str(&p("Standard", "")); // Abstand zwischen Tabellen
        }

        // ── Wichtige Infos ───────────────────────────────────────────────────
        out.push_str(&format!(
            "<table:table table:name=\"Dienst\" table:style-name=\"TabBett\">\
             <table:table-column table:style-name=\"SpalteLabel\"/>\
             <table:table-column table:style-name=\"SpalteInhalt\"/>\
             <table:table-row>\
             <table:table-cell table:style-name=\"ZelleKopf\" table:number-columns-spanned=\"2\" office:value-type=\"string\">\
             {}\
             </table:table-cell>\
             <table:covered-table-cell/>\
             </table:table-row>\
             <table:table-row>\
             <table:table-cell table:style-name=\"ZelleNormal\" table:number-columns-spanned=\"2\" office:value-type=\"string\">\
             {}\
             </table:table-cell>\
             <table:covered-table-cell/>\
             </table:table-row>\
             </table:table>",
            p("KopfText", "WICHTIGE INFORMATIONEN FÜR DEN DIENST"),
            if self.station.dienst_info.is_empty() {
                p("Standard", "-")
            } else {
                self.station.dienst_info.lines()
                    .map(|z| p("Standard", &esc(z)))
                    .collect::<String>()
            }
        ));

        // ── Fußzeile: MZ-HyprNurs + Druckdatum ─────────────────────────────
        out.push_str(&p("Standard", ""));
        {
            let fuss = format!(
                "MZ-HyprNurs - {}",
                chrono::Local::now().format("%d.%m.%Y %H:%M Uhr")
            );
            out.push_str(&p("Standard", &esc(&fuss)));
        }

        out.push_str("    </office:text>\n  </office:body>\n</office:document-content>\n");
        out
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "macos")]
    let icon_bytes: &[u8] = include_bytes!("../assets/icon_macos.png");
    #[cfg(not(target_os = "macos"))]
    let icon_bytes: &[u8] = include_bytes!("../assets/icon.png");
    let icon = eframe::icon_data::from_png_bytes(icon_bytes).expect("Failed to load icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mz-hyprnurs")
            .with_inner_size([1300.0, 900.0])
            .with_app_id("mz-hyprnurs")
            .with_icon(icon),
        vsync: false,
        ..Default::default()
    };
    eframe::run_native(
        "mz-hyprnurs",
        options,
        Box::new(|cc| {
            // Bold-Font laden (plattformübergreifend, mehrere Pfade)
            let mut fonts = egui::FontDefinitions::default();
            let bold_pfade: &[&str] = {
                #[cfg(target_os = "linux")]
                { &[
                    "/usr/share/fonts/liberation/LiberationSans-Bold.ttf",
                    "/usr/share/fonts/TTF/LiberationSans-Bold.ttf",
                    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
                    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                    "/usr/share/fonts/noto/NotoSans-Bold.ttf",
                    "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
                ] }
                #[cfg(target_os = "macos")]
                { &[
                    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                    "/System/Library/Fonts/Supplemental/Verdana Bold.ttf",
                    "/System/Library/Fonts/Supplemental/Georgia Bold.ttf",
                ] }
                #[cfg(windows)]
                { &[
                    "C:\\Windows\\Fonts\\arialbd.ttf",
                    "C:\\Windows\\Fonts\\verdanab.ttf",
                    "C:\\Windows\\Fonts\\calibrib.ttf",
                ] }
            };
            for pfad in bold_pfade {
                if let Ok(data) = std::fs::read(pfad) {
                    fonts.font_data.insert(
                        "system_bold".to_owned(),
                        egui::FontData::from_owned(data),
                    );
                    fonts.families
                        .entry(egui::FontFamily::Name("bold".into()))
                        .or_default()
                        .push("system_bold".to_owned());
                    break;
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MzHyprNursApp::new()))
        }),
    )
}

impl eframe::App for MzHyprNursApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hyprland: verhindert Einfrieren bei Workspace-Wechsel
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // ── Screensaver: Interaktion erkennen ───────────────────────────────
        let hat_eingabe = ctx.input(|i| {
            !i.events.is_empty()
                || i.pointer.any_pressed()
                || i.pointer.delta() != egui::Vec2::ZERO
        });
        if hat_eingabe {
            self.letzte_interaktion = std::time::Instant::now();
            self.anim_matrix.clear(); // ANIM
        }
        let screensaver_aktiv =
            self.letzte_interaktion.elapsed() >= std::time::Duration::from_secs(60);

        if screensaver_aktiv {
            let screen = ctx.screen_rect();

            // Textur laden - tex_id/img_size per Copy rauslösen, Borrow endet danach
            let (tex_id, img_size) = self.hintergrund_textur(ctx);

            // Bild zentriert, volle Fenstergröße
            let scale = (screen.width() / img_size.x).min(screen.height() / img_size.y).min(1.0_f32);
            let bild_b = img_size.x * scale;
            let bild_h = img_size.y * scale;
            let bild_rect = egui::Rect::from_min_size(
                screen.min + egui::Vec2::new((screen.width() - bild_b) * 0.5, (screen.height() - bild_h) * 0.5),
                egui::Vec2::new(bild_b, bild_h),
            );

            // Matrix-Regen vorbereiten (nur wenn Modus aktiv)
            const CHAR_W:  f32 = 16.0;                                          // ANIM
            const CHAR_H:  f32 = 22.0;                                          // ANIM
            const ZEICHEN: &[u8] = b"0123456789MZHYPRNURS!@#$%/\\|<>?-+=*~";   // ANIM
            let zeichen_liste: Vec<(egui::Pos2, char, egui::Color32)> = if self.matrix_modus { // ANIM
                let dt = self.anim_letzter_tick.elapsed().as_secs_f32().min(0.05); // ANIM
                self.anim_letzter_tick = std::time::Instant::now();              // ANIM
                self.anim_frame = self.anim_frame.wrapping_add(1);               // ANIM
                if self.anim_matrix.is_empty() {                                 // ANIM
                    let n = (screen.width() / CHAR_W).ceil() as usize + 2;      // ANIM
                    for i in 0..n {                                              // ANIM
                        self.anim_matrix.push(MatrixSpalte {                     // ANIM
                            x:      i as f32 * CHAR_W + CHAR_W * 0.5,           // ANIM
                            y:     -(((i * 73 + 17) % 600) as f32),             // ANIM
                            speed:  120.0 + ((i * 31 + 11) % 140) as f32,       // ANIM
                            laenge: 160.0 + ((i * 53 +  7) % 220) as f32,       // ANIM
                        });                                                      // ANIM
                    }                                                            // ANIM
                }                                                                // ANIM
                let frame_idx = (self.anim_frame / 3) as usize;                 // ANIM
                let mut liste = Vec::with_capacity(8192);                        // ANIM
                for (si, spalte) in self.anim_matrix.iter_mut().enumerate() {   // ANIM
                    spalte.y += spalte.speed * dt;                               // ANIM
                    if spalte.y - spalte.laenge > screen.height() {              // ANIM
                        spalte.y = -(((si * 17 + 5) % 150) as f32);             // ANIM
                    }                                                            // ANIM
                    let n_z = (spalte.laenge / CHAR_H) as usize + 1;            // ANIM
                    for ci in 0..n_z {                                           // ANIM
                        let cy = spalte.y - ci as f32 * CHAR_H;                 // ANIM
                        if cy < -CHAR_H || cy > screen.height() { continue; }   // ANIM
                        let frac  = ci as f32 / n_z as f32;                     // ANIM
                        let alpha = ((1.0 - frac) * 230.0) as u8;               // ANIM
                        let farbe = if ci == 0 {                                 // ANIM
                            egui::Color32::from_rgb(255, 210, 210)               // ANIM
                        } else {                                                 // ANIM
                            egui::Color32::from_rgba_unmultiplied(210, 0, 0, alpha) // ANIM
                        };                                                       // ANIM
                        let idx = si.wrapping_mul(7).wrapping_add(ci.wrapping_mul(3)) // ANIM
                            .wrapping_add(frame_idx) % ZEICHEN.len();           // ANIM
                        liste.push((egui::Pos2::new(spalte.x, cy), ZEICHEN[idx] as char, farbe)); // ANIM
                    }                                                            // ANIM
                }                                                                // ANIM
                liste                                                            // ANIM
            } else { Vec::new() };                                               // ANIM

            egui::Area::new(egui::Id::new("screensaver"))
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let p = ui.painter();
                    p.rect_filled(screen, 0.0, egui::Color32::BLACK);
                    p.image(tex_id, bild_rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                        egui::Color32::WHITE);
                    for (pos, c, farbe) in &zeichen_liste {                      // ANIM
                        p.text(*pos, egui::Align2::CENTER_TOP, c.to_string(),   // ANIM
                            egui::FontId::monospace(18.0), *farbe);              // ANIM
                    }                                                            // ANIM
                });
            if self.matrix_modus {                                               // ANIM
                ctx.request_repaint();                                           // ANIM
            }
            return;
        }

        // ── HyprGross-Modus ──────────────────────────────────────────────────
        if self.hyprgross_aktiv {
            // Beenden per Ctrl+G oder ESC
            if ctx.input(|i| (i.modifiers.ctrl && i.key_pressed(egui::Key::G))
                || i.key_pressed(egui::Key::Escape))
            {
                self.hyprgross_aktiv = false;
                return;
            }

            let flat = self.nicht_leere_betten();
            let n = flat.len();

            // Tastatureingaben
            let links  = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
            let rechts = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
            let hoch   = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
            let runter = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));

            match self.hyprgross_ansicht {
                HyprGrossAnsicht::HyprInfo => {
                    // Rechts → zurück zum ersten Bett, Feld 0
                    if rechts && n > 0 {
                        self.hyprgross_ansicht = HyprGrossAnsicht::Feld;
                        self.hyprgross_bett_pos = 0;
                        self.hyprgross_feld = 0;
                    }
                }
                HyprGrossAnsicht::DienstInfo => {
                    // Links → zurück zum letzten Bett, Feld 5
                    if links && n > 0 {
                        self.hyprgross_ansicht = HyprGrossAnsicht::Feld;
                        self.hyprgross_bett_pos = n - 1;
                        self.hyprgross_feld = 5;
                    }
                }
                HyprGrossAnsicht::Feld => {
                    // ← Feld zurück
                    if links {
                        if self.hyprgross_feld == 0 {
                            if self.hyprgross_bett_pos == 0 {
                                self.hyprgross_ansicht = HyprGrossAnsicht::HyprInfo;
                            } else {
                                self.hyprgross_bett_pos -= 1;
                                self.hyprgross_feld = 5;
                            }
                        } else {
                            self.hyprgross_feld -= 1;
                        }
                    }
                    // → Feld vor
                    if rechts {
                        if self.hyprgross_feld == 5 {
                            if n > 0 && self.hyprgross_bett_pos == n - 1 {
                                self.hyprgross_ansicht = HyprGrossAnsicht::DienstInfo;
                            } else {
                                self.hyprgross_bett_pos += 1;
                                self.hyprgross_feld = 0;
                            }
                        } else {
                            self.hyprgross_feld += 1;
                        }
                    }
                    // ↓ nächstes Bett (mit Wrap) - immer bei Patient starten
                    if runter && n > 0 {
                        self.hyprgross_bett_pos = (self.hyprgross_bett_pos + 1) % n;
                        self.hyprgross_feld = 0;
                    }
                    // ↑ vorheriges Bett (mit Wrap) - immer bei Patient starten
                    if hoch && n > 0 {
                        self.hyprgross_bett_pos = (self.hyprgross_bett_pos + n - 1) % n;
                        self.hyprgross_feld = 0;
                    }
                }
            }

            // Inhalte ermitteln
            let screen = ctx.screen_rect();
            let margin = 36.0;
            let rot = egui::Color32::from_rgb(255, 0, 0);
            let weiss = egui::Color32::WHITE;

            // (feld_label, inhalt, zimmer_bett, kopf_nachname, kopf_rest, besonderheiten)
            let (feld_label, inhalt, zimmer_bett, kopf_nachname, kopf_rest, besonderheiten) = match self.hyprgross_ansicht {
                HyprGrossAnsicht::HyprInfo => {
                    let text = format!("{}\n{}", self.station_name, self.station_hyprinfo);
                    ("HyprInfo".to_string(), text, String::new(), String::new(), String::new(), String::new())
                }
                HyprGrossAnsicht::DienstInfo => {
                    ("Wichtige Infos".to_string(), self.station.dienst_info.clone(),
                     String::new(), String::new(), String::new(), String::new())
                }
                HyprGrossAnsicht::Feld => {
                    if n == 0 {
                        ("–".to_string(), "(keine Patienten)".to_string(),
                         String::new(), String::new(), String::new(), String::new())
                    } else {
                        let (zi, bi) = flat[self.hyprgross_bett_pos.min(n - 1)];
                        let zimmer = &self.station.zimmer[zi];
                        let bett   = &zimmer.betten[bi];
                        let zb = format!("Zimmer {} | Bett {}", zimmer.nummer, bett.buchstabe);
                        if let Some(ref pat) = bett.patient {
                            let anrede_prefix = if !pat.anrede.is_empty() && pat.anrede != "–" {
                                format!("{} ", pat.anrede)
                            } else { String::new() };
                            let bio_suffix = alter_bio_suffix(&pat.geburtsdatum, &pat.bio_geschlecht);
                            let k_nach = format!("{}{}", anrede_prefix, pat.nachname);
                            let k_rest = format!(", {}{}", pat.vorname, bio_suffix);
                            let (label, text) = match self.hyprgross_feld {
                                0 => ("Patient".to_string(),
                                      format!("{}{}, {}{}", anrede_prefix, pat.nachname, pat.vorname, bio_suffix)),
                                1 => (if self.psychiatrie_modus { "Psychiatrische Diagnose".to_string() } else { "Hauptdiagnose".to_string() }, pat.hdia.clone()),
                                2 => (if self.psychiatrie_modus { "Somatische Diagnose".to_string() } else { "Nebendiagnose".to_string() }, pat.ndia.clone()),
                                3 => ("Pflege".to_string(),        pat.pflege.clone()),
                                4 => ("Info".to_string(),          pat.info.clone()),
                                _ => ("ToDo".to_string(),          pat.todo.clone()),
                            };
                            (label, text, zb, k_nach, k_rest, pat.besonderheiten.clone())
                        } else {
                            ("–".to_string(), String::new(), zb, String::new(), String::new(), String::new())
                        }
                    }
                }
            };

            // Datum/Uhrzeit live
            let zeitstring = chrono::Local::now().format("%d.%m.%Y @%H:%M").to_string();

            // Hintergrundbild für oben-rechts laden (selbe Textur wie Screensaver)
            let (hyprgross_tex_id, hyprgross_img_size) = self.hintergrund_textur(ctx);

            // Vollbild-Overlay rendern
            egui::Area::new(egui::Id::new("hyprgross"))
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let p = ui.painter();
                    p.rect_filled(screen, 0.0, egui::Color32::BLACK);

                    let label_font = egui::FontId::proportional(24.0);
                    let top_y = margin;
                    let bot_y = screen.height() - margin;

                    // Oben links: Feldname
                    p.text(
                        egui::Pos2::new(margin, top_y),
                        egui::Align2::LEFT_TOP,
                        &feld_label,
                        label_font.clone(),
                        weiss,
                    );
                    // Oben Mitte: Nachname (fett) + ", Vorname [*Alter]" (normal)
                    if !kopf_nachname.is_empty() {
                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            &kopf_nachname,
                            0.0,
                            egui::TextFormat {
                                font_id: egui::FontId::new(24.0, egui::FontFamily::Name("bold".into())),
                                color: weiss,
                                ..Default::default()
                            },
                        );
                        if !kopf_rest.is_empty() {
                            job.append(
                                &kopf_rest,
                                0.0,
                                egui::TextFormat {
                                    font_id: label_font.clone(),
                                    color: weiss,
                                    ..Default::default()
                                },
                            );
                        }
                        let galley = ctx.fonts(|f| f.layout_job(job));
                        let gw = galley.size().x;
                        p.galley(
                            egui::Pos2::new((screen.width() - gw) * 0.5, top_y),
                            galley,
                            weiss,
                        );
                    }
                    // Oben rechts: Hintergrundbild klein
                    let bild_hoehe = 100.0_f32;
                    let bild_breite = hyprgross_img_size.x / hyprgross_img_size.y * bild_hoehe;
                    let img_rand_oben   = 0.0_f32;
                    let img_rand_rechts = 16.0_f32;
                    let bild_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(screen.width() - img_rand_rechts - bild_breite, img_rand_oben),
                        egui::Vec2::new(bild_breite, bild_hoehe),
                    );
                    p.image(hyprgross_tex_id, bild_rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                        egui::Color32::WHITE);
                    // Unten links: Zimmer | Bett
                    if !zimmer_bett.is_empty() {
                        p.text(
                            egui::Pos2::new(margin, bot_y),
                            egui::Align2::LEFT_BOTTOM,
                            &zimmer_bett,
                            label_font.clone(),
                            weiss,
                        );
                    }
                    // Unten Mitte: Besonderheiten
                    if !besonderheiten.is_empty() {
                        p.text(
                            egui::Pos2::new(screen.width() * 0.5, bot_y),
                            egui::Align2::CENTER_BOTTOM,
                            &besonderheiten,
                            label_font.clone(),
                            weiss,
                        );
                    }
                    // Unten rechts: Datum/Uhrzeit
                    p.text(
                        egui::Pos2::new(screen.width() - margin, bot_y),
                        egui::Align2::RIGHT_BOTTOM,
                        &zeitstring,
                        label_font.clone(),
                        weiss,
                    );

                    // Mitte: großer roter Text
                    let max_breite = screen.width() - 2.0 * margin;
                    let galley = ctx.fonts(|f| f.layout(
                        if inhalt.is_empty() { "–".to_string() } else { inhalt.clone() },
                        egui::FontId::new(80.0, egui::FontFamily::Name("bold".into())),
                        rot,
                        max_breite,
                    ));
                    let galley_h = galley.size().y;
                    let center_y = (screen.height() - galley_h) * 0.5;
                    p.galley(egui::Pos2::new(margin, center_y), galley, rot);
                });

            return;
        }

        // Strg+T: Theme wechseln
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::T)) {
            self.theme = self.theme.naechstes(self.hat_omarchy);
        }
        // Strg+N: Neu
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
            self.neu_bestaetigen = true;
        }
        // Strg+S: Speichern
        if ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.speichern();
        }
        // Strg+Shift+E: Einstellungen
        if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::E)) {
            self.einstellungen_offen = true;
            self.einstellungen_zimmer_anzahl = self.station.zimmer.len();
            self.einstellungen_psychiatrie = self.psychiatrie_modus;
        }
        // Strg+Shift+S: Speichern unter
        if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.speichern_unter();
        }
        // Strg+O: Öffnen
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            self.laden();
        }

        // ── Änderungserkennung (alle 30 Sek.) ───────────────────────────────
        if self.letzter_inhalt_check.elapsed() > std::time::Duration::from_secs(30) {
            self.letzter_inhalt_check = std::time::Instant::now();
            let aktuell = self.markdown_erstellen();
            self.geaendert = aktuell != self.inhalt_beim_speichern;
        }

        // ── Speichern nach Schließen des Erfassungsfensters (nächster Frame) ──
        if self.speichern_nach_schliessen && self.bearbeitung.is_none() {
            self.speichern_nach_schliessen = false;
            self.stilles_speichern();
        }

        // ── Auto-Speichern (alle 10 Min., nur bei Änderungen) ────────────────
        if self.geaendert
            && self.speicher_pfad.is_some()
            && self.letzte_auto_speicherung.elapsed() > std::time::Duration::from_secs(600)
        {
            self.stilles_speichern();
        }

        // Strg+P: PDF erzeugen
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::P)) {
            self.pdf_exportieren();
        }
        // Strg+L: ODT erzeugen
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
            self.odt_exportieren();
        }
        // Strg+Q: Beenden
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            self.beenden_bestaetigen = true;
        }
        // Strg+H: Hilfe
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::H)) {
            url_oeffnen("https://www.marcelzimmer.de");
        }
        // Strg+I: Über
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::I)) {
            self.ueber_dialog_offen = true;
        }
        // Strg+M: Matrix-Regen-Screensaver an/aus
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::M)) {
            self.matrix_modus = !self.matrix_modus;
            self.anim_matrix.clear(); // ANIM
            self.stilles_speichern();
        }
        // Strg+G: HyprGross ein/aus (nur aus Bettenübersicht)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
            if self.hyprgross_aktiv {
                self.hyprgross_aktiv = false;
            } else if self.bearbeitung.is_none() {
                self.hyprgross_bett_pos = 0;
                self.hyprgross_feld = 0;
                self.hyprgross_ansicht = HyprGrossAnsicht::HyprInfo;
                self.hyprgross_aktiv = true;
            }
        }

        // Dialog-Ergebnisse abholen
        let mut dialog_ergebnis: Option<DialogErgebnis> = None;
        if let Some(rx) = &self.dialog_rx {
            if let Ok(e) = rx.try_recv() {
                dialog_ergebnis = Some(e);
            }
        }
        if let Some(ergebnis) = dialog_ergebnis {
            match ergebnis {
                DialogErgebnis::Speichern { pfad, inhalt, am, von } => {
                    self.speicher_pfad = Some(pfad);
                    self.letzte_aenderung_am  = Some(am);
                    self.letzte_aenderung_von = Some(von);
                    self.inhalt_beim_speichern = inhalt;
                    self.geaendert = false;
                    self.letzte_auto_speicherung = std::time::Instant::now();
                }
                DialogErgebnis::Laden(pfad, inhalt) => {
                    self.speicher_pfad = Some(pfad);
                    self.inhalt_beim_speichern = inhalt.clone();
                    self.geaendert = false;
                    self.letzte_auto_speicherung = std::time::Instant::now();
                    self.markdown_parsen(&inhalt);
                }
                DialogErgebnis::PdfExport(pfad) => {
                    if let Some((reg, bold)) = self.ausstehende_pdf_schrift.take() {
                        self.pdf_generieren(&pfad, reg, bold);
                    }
                }
                DialogErgebnis::OdtExport(pfad) => {
                    self.odt_generieren(&pfad);
                }
            }
            self.dialog_rx = None;
        }

        // Omarchy-Farben einmalig laden (wird in Theme-Setup und Zeichnung verwendet)
        let omarchy_farben: Option<HashMap<String, Color32>> = if self.theme == Theme::Omarchy {
            omarchy_farben_laden()
        } else {
            None
        };
        let akt = AktFarben::von_theme(self.theme, omarchy_farben.as_ref());
        let akt_akzent     = akt.akzent;
        let akt_karten_bg  = akt.karten_bg;
        let akt_fenster_bg = akt.fenster_bg;
        let akt_kopf_text  = akt.kopf_text;
        let akt_kopf_dim   = akt.kopf_dim;
        let akt_trennlinie = akt.trennlinie;

        match self.theme {
            Theme::Hell => {
                let mut visuals = egui::Visuals::light();
                visuals.panel_fill = Color32::from_rgb(242, 242, 247);
                ctx.set_visuals(visuals);
            }
            Theme::Dunkel => {
                let mut visuals = egui::Visuals::dark();
                visuals.panel_fill = Color32::from_rgb(0, 0, 0);
                ctx.set_visuals(visuals);
            }
            Theme::CPCgruen => {
                let mut visuals = egui::Visuals::dark();
                let dunkelgruen = Color32::from_rgb(0x00, 0x18, 0x00);
                let gruen = Color32::from_rgb(0x33, 0xFF, 0x33);
                visuals.panel_fill = dunkelgruen;
                visuals.window_fill = dunkelgruen;
                visuals.extreme_bg_color = dunkelgruen;
                let gruen_strich = Stroke::new(1.0, gruen);
                visuals.widgets.noninteractive.fg_stroke = gruen_strich;
                visuals.widgets.inactive.fg_stroke = gruen_strich;
                visuals.widgets.hovered.fg_stroke = gruen_strich;
                visuals.widgets.active.fg_stroke = gruen_strich;
                visuals.widgets.noninteractive.bg_stroke = gruen_strich;
                visuals.widgets.inactive.bg_stroke = gruen_strich;
                visuals.widgets.hovered.bg_stroke = gruen_strich;
                visuals.widgets.active.bg_stroke = gruen_strich;
                visuals.widgets.open.bg_stroke = gruen_strich;
                visuals.widgets.open.fg_stroke = gruen_strich;
                visuals.widgets.open.bg_fill = dunkelgruen;
                visuals.widgets.inactive.bg_fill = dunkelgruen;
                visuals.widgets.active.bg_fill = dunkelgruen;
                visuals.selection.bg_fill = Color32::from_rgb(0x00, 0x66, 0x00);
                visuals.hyperlink_color = gruen;
                visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x00, 0x33, 0x00);
                ctx.set_visuals(visuals);
            }
            Theme::CPCrot => {
                let mut visuals = egui::Visuals::dark();
                let dunkelrot = Color32::from_rgb(0x18, 0x00, 0x00);
                let rot = Color32::from_rgb(0xFF, 0x33, 0x33);
                visuals.panel_fill = dunkelrot;
                visuals.window_fill = dunkelrot;
                visuals.extreme_bg_color = dunkelrot;
                let rot_strich = Stroke::new(1.0, rot);
                visuals.widgets.noninteractive.fg_stroke = rot_strich;
                visuals.widgets.inactive.fg_stroke = rot_strich;
                visuals.widgets.hovered.fg_stroke = rot_strich;
                visuals.widgets.active.fg_stroke = rot_strich;
                visuals.widgets.noninteractive.bg_stroke = rot_strich;
                visuals.widgets.inactive.bg_stroke = rot_strich;
                visuals.widgets.hovered.bg_stroke = rot_strich;
                visuals.widgets.active.bg_stroke = rot_strich;
                visuals.widgets.open.bg_stroke = rot_strich;
                visuals.widgets.open.fg_stroke = rot_strich;
                visuals.widgets.open.bg_fill = dunkelrot;
                visuals.widgets.inactive.bg_fill = dunkelrot;
                visuals.widgets.active.bg_fill = dunkelrot;
                visuals.selection.bg_fill = Color32::from_rgb(0x66, 0x00, 0x00);
                visuals.hyperlink_color = rot;
                visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x40, 0x00, 0x00);
                ctx.set_visuals(visuals);
            }
            Theme::CPCblaugelb => {
                let mut visuals = egui::Visuals::dark();
                let blau = Color32::from_rgb(0x00, 0x00, 0x80);
                let gelb = Color32::from_rgb(0xFF, 0xFF, 0x00);
                visuals.panel_fill = blau;
                visuals.window_fill = blau;
                visuals.extreme_bg_color = blau;
                let gelb_strich = Stroke::new(1.0, gelb);
                visuals.widgets.noninteractive.fg_stroke = gelb_strich;
                visuals.widgets.inactive.fg_stroke = gelb_strich;
                visuals.widgets.hovered.fg_stroke = gelb_strich;
                visuals.widgets.active.fg_stroke = gelb_strich;
                visuals.widgets.noninteractive.bg_stroke = gelb_strich;
                visuals.widgets.inactive.bg_stroke = gelb_strich;
                visuals.widgets.hovered.bg_stroke = gelb_strich;
                visuals.widgets.active.bg_stroke = gelb_strich;
                visuals.widgets.inactive.bg_fill = blau;
                visuals.widgets.active.bg_fill = blau;
                visuals.selection.bg_fill = Color32::from_rgb(0x80, 0x80, 0x00);
                visuals.hyperlink_color = gelb;
                visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x00, 0x00, 0xB0);
                ctx.set_visuals(visuals);
            }
            Theme::Omarchy => {
                if let Some(ref farben) = omarchy_farben {
                    let bg     = farben.get("background").copied().unwrap_or(Color32::from_rgb(26, 27, 37));
                    let fg     = farben.get("foreground").copied().unwrap_or(Color32::from_gray(200));
                    let akzent = farben.get("accent")    .copied().unwrap_or(Color32::from_rgb(122, 162, 247));

                    // Light/Dark-Basis nach Hintergrund-Helligkeit wählen.
                    let mut visuals = if ist_hell(bg) {
                        egui::Visuals::light()
                    } else {
                        egui::Visuals::dark()
                    };

                    // Flächenfarben direkt aus colors.toml.
                    visuals.panel_fill       = bg;
                    visuals.window_fill      = bg;
                    visuals.extreme_bg_color = bg;

                    // Trennlinien/Rahmen aus fg/bg-Blend [nicht mehr aus cursor oder color8].
                    let trenn = mischen(bg, fg, 0.18);
                    let trennstrich = Stroke::new(1.0, trenn);
                    visuals.widgets.noninteractive.bg_stroke = trennstrich;
                    visuals.widgets.inactive.bg_stroke       = trennstrich;

                    // Primärtext folgt dem foreground der Theme-Datei.
                    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, fg);

                    // Interaktive Widgets: Akzent als Strich.
                    let akzent_strich = Stroke::new(1.0, akzent);
                    visuals.widgets.inactive.fg_stroke = akzent_strich;
                    visuals.widgets.hovered.fg_stroke  = akzent_strich;
                    visuals.widgets.active.fg_stroke   = akzent_strich;

                    // Selektion direkt aus colors.toml nutzen, sonst Akzent.
                    visuals.selection.bg_fill = farben
                        .get("selection_background")
                        .copied()
                        .unwrap_or(akzent);
                    if let Some(sel_fg) = farben.get("selection_foreground").copied() {
                        visuals.selection.stroke = Stroke::new(1.0, sel_fg);
                    }

                    visuals.hyperlink_color = akzent;

                    // Hover-Fläche dezent mit Akzent tönen [funktioniert für hell und dunkel].
                    visuals.widgets.hovered.bg_fill = mischen(bg, akzent, 0.20);

                    ctx.set_visuals(visuals);
                } else {
                    // Fallback, wenn ~/.config/omarchy/current/theme/colors.toml fehlt.
                    let mut visuals = egui::Visuals::dark();
                    visuals.panel_fill  = Color32::from_rgb(26, 27, 37);
                    visuals.window_fill = Color32::from_rgb(26, 27, 37);
                    ctx.set_visuals(visuals);
                }
            }
        }

        // CPC464: CRT-Scanlines über den gesamten Bildschirm
        if self.theme == Theme::CPCgruen || self.theme == Theme::CPCrot {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("scanlines"),
            ));
            let rect = ctx.screen_rect();
            let scanline_farbe = Color32::from_rgba_premultiplied(0, 0, 0, 60);
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(rect.left(), y), egui::vec2(rect.width(), 2.0)),
                    0.0,
                    scanline_farbe,
                );
                y += 4.0;
            }
        }

        // ── Übersicht ──
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                // Stationsname - genau so breit wie eine Bett-Karte
                ui.add(
                    egui::TextEdit::singleline(&mut self.station_name)
                        .frame(false)
                        .font(FontId::new(22.0, egui::FontFamily::Name("bold".into())))
                        .desired_width(KARTE_B)
                        .char_limit(30)
                        .hint_text(
                            RichText::new("STATIONSNAME")
                                .color(Color32::from_gray(200))
                                .size(22.0),
                        ),
                );
                ui.add_space(KARTE_ABSTAND);
                // HyprInfo - startet genau über dem rechten Bett, gleich breit
                ui.add(
                    egui::TextEdit::singleline(&mut self.station_hyprinfo)
                        .frame(false)
                        .font(FontId::proportional(18.0))
                        .text_color(akt.text)
                        .desired_width(KARTE_B)
                        .char_limit(55)
                        .hint_text(
                            RichText::new("HYPRINFO")
                                .color(Color32::from_gray(200))
                                .size(18.0),
                        ),
                );

                // ── Menü oben rechts ──────────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let menue_eintraege: &[(&str, &str, i32)] = &[
                        ("Neu",              "Strg+N",    0),
                        ("Öffnen",           "Strg+O",   0),
                        ("Speichern",        "Strg+S",   0),
                        ("Speichern unter",  "Strg+Shift+S", 0),
                        ("PDF erzeugen",     "Strg+P",   0),
                        ("ODT erzeugen",     "Strg+L",    0),
                        ("",                 "",          1), // Trennlinie
                        ("Theme ändern",     "Strg+T",   0),
                        ("__MATRIX__",       "Strg+M",   0),
                        ("HyprGross",        "Strg+G",   0),
                        ("",                 "",               1), // Trennlinie
                        ("Einstellungen",    "Strg+Shift+E",  0),
                        ("",                 "",               1), // Trennlinie
                        ("Hilfe",            "Strg+H",         0),
                        ("Über",             "Strg+I",         0),
                        ("",                 "",               1), // Trennlinie
                        ("Beenden",          "Strg+Q",         0),
                    ];
                    let matrix_label = if self.matrix_modus { "Matrix: ausschalten" } else { "Matrix: einschalten" };
                    egui::menu::menu_button(ui, RichText::new("☰").size(16.0), |ui| {
                        ui.set_width(190.0);
                        for &(bezeichnung, kuerzel, ist_trennlinie) in menue_eintraege {
                            if ist_trennlinie == 1 {
                                ui.separator();
                                continue;
                            }
                            let anzeige = if bezeichnung == "__MATRIX__" { matrix_label } else { bezeichnung };
                            let breite = ui.available_width();
                            let (rect, antwort) = ui.allocate_exact_size(
                                egui::vec2(breite, 24.0),
                                egui::Sense::click(),
                            );
                            if ui.is_rect_visible(rect) {
                                if antwort.hovered() {
                                    ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
                                }
                                ui.painter().text(
                                    rect.left_center() + egui::vec2(8.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    anzeige,
                                    egui::FontId::proportional(13.0),
                                    ui.visuals().text_color(),
                                );
                                if !kuerzel.is_empty() {
                                    ui.painter().text(
                                        rect.right_center() - egui::vec2(8.0, 0.0),
                                        egui::Align2::RIGHT_CENTER,
                                        kuerzel,
                                        egui::FontId::proportional(12.0),
                                        ui.visuals().weak_text_color(),
                                    );
                                }
                            }
                            if antwort.clicked() {
                                match bezeichnung {
                                    "Neu"             => self.neu_bestaetigen = true,
                                    "Öffnen"          => self.laden(),
                                    "Speichern"       => self.speichern(),
                                    "Speichern unter" => self.speichern_unter(),
                                    "PDF erzeugen"    => self.pdf_exportieren(),
                                    "ODT erzeugen"    => self.odt_exportieren(),
                                    "Theme ändern"    => self.theme = self.theme.naechstes(self.hat_omarchy),
                                    "HyprGross"       => {
                                        self.hyprgross_bett_pos = 0;
                                        self.hyprgross_feld = 0;
                                        self.hyprgross_ansicht = HyprGrossAnsicht::HyprInfo;
                                        self.hyprgross_aktiv = true;
                                    }
                                    "__MATRIX__"      => {
                                        self.matrix_modus = !self.matrix_modus;
                                        self.anim_matrix.clear(); // ANIM
                                        self.stilles_speichern();
                                    }
                                    "Beenden"         => self.beenden_bestaetigen = true,
                                    "Einstellungen"   => {
                                        self.einstellungen_offen = true;
                                        self.einstellungen_zimmer_anzahl = self.station.zimmer.len();
                                        self.einstellungen_psychiatrie = self.psychiatrie_modus;
                                    }
                                    "Hilfe"           => url_oeffnen("https://www.marcelzimmer.de"),
                                    "Über"            => self.ueber_dialog_offen = true,
                                    _ => {}
                                }
                                ui.close_menu();
                            }
                        }
                    });
                });
            });
            ui.add_space(16.0);

            // ── Tastaturnavigation in der Übersicht ──
            let kein_fokus = ctx.memory(|m| m.focused().is_none());
            let mut scroll_noetig = false;
            if self.bearbeitung.is_none() && kein_fokus {
                let gehe_rechts  = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
                let gehe_links   = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
                let gehe_runter  = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
                let gehe_hoch    = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
                let gehe_oeffnen = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                let anz_zimmer   = self.station.zimmer.len();

                if gehe_rechts || gehe_links || gehe_runter || gehe_hoch {
                    scroll_noetig = true;
                    if self.info_ausgewaehlt {
                        // Von Dienstinfo: nur Hoch navigiert zurück zu letztem Zimmer links
                        if gehe_hoch {
                            self.info_ausgewaehlt = false;
                            self.ausgewaehlte_karte = Some((anz_zimmer - 1, 0));
                        }
                    } else if self.ausgewaehlte_karte.is_none() {
                        self.ausgewaehlte_karte = Some((0, 0));
                    } else {
                        let (zi, bi) = self.ausgewaehlte_karte.unwrap();
                        if gehe_runter && zi == anz_zimmer - 1 {
                            // Letztes Zimmer + Runter → Dienstinfo
                            self.ausgewaehlte_karte = None;
                            self.info_ausgewaehlt = true;
                        } else {
                            self.ausgewaehlte_karte = Some(if gehe_rechts {
                                (zi, (bi + 1).min(1))
                            } else if gehe_links {
                                (zi, bi.saturating_sub(1))
                            } else if gehe_runter {
                                ((zi + 1).min(anz_zimmer - 1), bi)
                            } else {
                                (zi.saturating_sub(1), bi)
                            });
                        }
                    }
                }
                if gehe_oeffnen {
                    if let Some((zi, bi)) = self.ausgewaehlte_karte {
                        // Bett anlegen falls es noch nicht existiert
                        while self.station.zimmer[zi].betten.len() <= bi {
                            let buchstabe = (b'A' + self.station.zimmer[zi].betten.len() as u8) as char;
                            self.station.zimmer[zi].betten.push(Bett {
                                buchstabe: buchstabe.to_string(),
                                patient: None,
                            });
                        }
                        self.bearbeitung = Some((zi, bi));
                        self.bearbeitungsfeld = 0;
                    }
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Alle Zimmer haben mindestens 2 Betten
                for zi in 0..self.station.zimmer.len() {
                    while self.station.zimmer[zi].betten.len() < 2 {
                        let buchstabe = (b'A' + self.station.zimmer[zi].betten.len() as u8) as char;
                        self.station.zimmer[zi].betten.push(Bett { buchstabe: buchstabe.to_string(), patient: None });
                    }
                }
                for (zi, zimmer) in self.station.zimmer.iter().enumerate() {
                    ui.horizontal_top(|ui| {
                        ui.add_space(16.0);
                        for bi in 0..2 {
                            let bett = &zimmer.betten[bi];
                            let ausgewaehlt = self.ausgewaehlte_karte == Some((zi, bi));
                            let (geklicktes_feld, karte_rect) = bett_karte_zeichnen(ui, &self.station.zimmer[zi].nummer, bett, ausgewaehlt, &akt, self.psychiatrie_modus);

                            if ausgewaehlt && scroll_noetig {
                                ui.scroll_to_rect(karte_rect, Some(egui::Align::Center));
                            }
                            if let Some(feld_idx) = geklicktes_feld {
                                self.ausgewaehlte_karte = Some((zi, bi));
                                self.bearbeitung = Some((zi, bi));
                                self.bearbeitungsfeld = feld_idx;
                            }
                            ui.add_space(KARTE_ABSTAND);
                        }
                    });
                    ui.add_space(KARTE_ABSTAND);
                }
                ui.add_space(24.0);

                // ── Dienstinfo-Card (gleicher Stil wie Bett-Karten) ──
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let info_b = KARTE_B * 2.0 + KARTE_ABSTAND;
                    let koerper_h = 130.0; // 5 Zeilen à ~18px + Padding
                    let karte_h = KOPF_H + koerper_h;
                    let (karte_rect, _) = ui.allocate_exact_size(
                        Vec2::new(info_b, karte_h),
                        egui::Sense::hover(),
                    );
                    let zeichner = ui.painter();
                    // Schatten
                    zeichner.rect_filled(
                        karte_rect.translate(Vec2::new(2.0, 4.0)),
                        Rounding::same(KARTE_RUND),
                        Color32::from_black_alpha(18),
                    );
                    zeichner.rect_filled(
                        karte_rect.translate(Vec2::new(1.0, 2.0)),
                        Rounding::same(KARTE_RUND),
                        Color32::from_black_alpha(10),
                    );
                    // Hintergrund
                    let info_ausgewaehlt_lokal = self.info_ausgewaehlt;
                    zeichner.rect_filled(karte_rect, Rounding::same(KARTE_RUND), akt_karten_bg);
                    // Kartenrahmen (gleicher Stil wie Bett-Karten)
                    let rahmen = if info_ausgewaehlt_lokal {
                        Stroke::new(2.5, akt_akzent)
                    } else {
                        Stroke::new(0.5, akt_trennlinie)
                    };
                    zeichner.rect_stroke(karte_rect, Rounding::same(KARTE_RUND), rahmen);
                    // Farbiger Header
                    let kopf_rect = Rect::from_min_size(karte_rect.min, Vec2::new(info_b, KOPF_H));
                    zeichner.rect_filled(
                        kopf_rect,
                        Rounding { nw: KARTE_RUND, ne: KARTE_RUND, sw: 0.0, se: 0.0 },
                        akt_akzent,
                    );
                    for dx in [0.0_f32, 0.6] {
                        zeichner.text(
                            Pos2::new(kopf_rect.min.x + 16.0 + dx, kopf_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Wichtige Informationen für den Dienst!",
                            FontId::proportional(17.0),
                            akt_kopf_text,
                        );
                    }
                    // TextEdit in der Body-Zone
                    let koerper_rect = Rect::from_min_size(
                        Pos2::new(karte_rect.min.x, karte_rect.min.y + KOPF_H),
                        Vec2::new(info_b, koerper_h),
                    );
                    let mut koerper_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(koerper_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    koerper_ui.add(
                        egui::TextEdit::multiline(&mut self.station.dienst_info)
                            .desired_width(info_b - 16.0)
                            .desired_rows(5)
                            .frame(false)
                            .font(FontId::proportional(18.0))
                            .margin(egui::Margin::symmetric(10.0, 8.0)),
                    );
                    // Leerzeilen verhindern, max 5 Zeilen (4 Zeilenumbrüche)
                    while self.station.dienst_info.contains("\n\n") {
                        self.station.dienst_info = self.station.dienst_info.replace("\n\n", "\n");
                    }
                    let mut zeilenumbrueche = 0usize;
                    self.station.dienst_info.retain(|c| {
                        if c == '\n' { zeilenumbrueche += 1; zeilenumbrueche <= 4 } else { true }
                    });
                    // Zu Dienstinfo scrollen wenn per Tastatur ausgewählt
                    if self.info_ausgewaehlt && scroll_noetig {
                        ui.scroll_to_rect(karte_rect, Some(egui::Align::Center));
                    }
                });

                ui.add_space(24.0);
            });
        });

        // ── Bearbeitungsfenster ──
        if let Some((zi, bi)) = self.bearbeitung {
            if self.station.zimmer[zi].betten[bi].patient.is_none() {
                self.station.zimmer[zi].betten[bi].patient = Some(Patient::default());
            }

            // Validierung vor dem mutable Borrow
            let nr_leer  = self.station.zimmer[zi].nummer.trim().is_empty();
            let bet_leer = self.station.zimmer[zi].betten[bi].buchstabe.trim().is_empty();
            let ist_duplikat = !nr_leer && !bet_leer && {
                let nr = &self.station.zimmer[zi].nummer;
                let buchstabe = &self.station.zimmer[zi].betten[bi].buchstabe;
                self.station.zimmer.iter().enumerate().any(|(zii, z)| {
                    z.betten.iter().enumerate().any(|(bii, b)| {
                        (zii != zi || bii != bi) && &z.nummer == nr && &b.buchstabe == buchstabe
                    })
                })
            };
            let ist_ungueltig = nr_leer || bet_leer || ist_duplikat;

            let mut schliessen = false;
            let mut naechstes = false;
            let mut vorheriges = false;
            let mut springe_zu: Option<usize> = None;
            let aktuelles_feld = self.bearbeitungsfeld; // Kopie - kein Borrow

            // Pfeiltasten navigieren zwischen Feldern wenn kein Textfeld fokussiert ist
            let kein_fokus = ctx.memory(|m| m.focused().is_none());
            if kein_fokus {
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) { naechstes = true; }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft))  { vorheriges = true; }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !self.loeschen_bestaetigen && !ist_ungueltig { schliessen = true; }
            }

            let bildschirm = ctx.screen_rect();
            let fenster_b = (bildschirm.width() * 0.94).min(1300.0);
            let fenster_h = (bildschirm.height() * 0.92).min(950.0);

            let bearbeitungs_rahmen = egui::Frame::none()
                .fill(akt_fenster_bg)
                .stroke(Stroke::new(3.0, akt_akzent))
                .rounding(Rounding::same(16.0))
                .inner_margin(egui::Margin::same(0.0))
                .shadow(egui::epaint::Shadow {
                    offset: Vec2::new(0.0, 8.0),
                    blur: 32.0,
                    spread: 0.0,
                    color: Color32::from_black_alpha(70),
                });

            // Borrows vor der Closure trennen
            let geb_fehler = &mut self.geb_fehler;
            let loeschen_bestaetigen = &mut self.loeschen_bestaetigen;
            let zimmer = &mut self.station.zimmer[zi];
            let zimmer_nummer = &mut zimmer.nummer;
            let bett = &mut zimmer.betten[bi];

            egui::Window::new(format!("edit_{}_{}", zi, bi))
                .title_bar(false)
                .frame(bearbeitungs_rahmen)
                .resizable(false)
                .fixed_size([fenster_b, fenster_h])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // ── Farbiger Kopfstreifen ──
                    let kopf_rahmen = egui::Frame::none()
                        .fill(akt_akzent)
                        .rounding(Rounding { nw: 14.0, ne: 14.0, sw: 0.0, se: 0.0 })
                        .inner_margin(egui::Margin::symmetric(20.0, 10.0));

                    kopf_rahmen.show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_max_height(64.0);
                        ui.horizontal(|ui| {
                            let schrift = FontId::proportional(22.0);

                            // Textbreite messen → Feldbreite = Textbreite + innere Ränder (8px)
                            let zim_breite = ui.fonts(|f| {
                                let anzeige = if zimmer_nummer.is_empty() { "ZIMM".to_string() } else { zimmer_nummer.clone() };
                                f.layout_no_wrap(anzeige, schrift.clone(), akt_kopf_text).size().x
                            }) + 4.0;
                            let bett_breite = ui.fonts(|f| {
                                let anzeige = if bett.buchstabe.is_empty() { "BETT".to_string() } else { bett.buchstabe.clone() };
                                f.layout_no_wrap(anzeige, schrift.clone(), akt_kopf_text).size().x
                            }) + 4.0;

                            // ZIMMER - inline, kein Rahmen, max 4 Zeichen
                            ui.spacing_mut().item_spacing.x = 3.0;
                            ui.add(
                                egui::TextEdit::singleline(zimmer_nummer)
                                    .frame(false)
                                    .font(schrift.clone())
                                    .text_color(akt_kopf_text)
                                    .hint_text(RichText::new("ZIMMER").color(akt_kopf_dim).size(22.0))
                                    .desired_width(zim_breite)
                                    .char_limit(4),
                            );

                            ui.label(RichText::new("·").color(akt_kopf_dim).size(22.0));

                            // BETT - inline, max 7 Zeichen
                            ui.spacing_mut().item_spacing.x = 3.0;
                            ui.add(
                                egui::TextEdit::singleline(&mut bett.buchstabe)
                                    .frame(false)
                                    .font(schrift.clone())
                                    .text_color(akt_kopf_text)
                                    .hint_text(RichText::new("BETT").color(akt_kopf_dim).size(22.0))
                                    .desired_width(bett_breite)
                                    .char_limit(7),
                            );

                            // Besonderheiten
                            if let Some(pat) = &mut bett.patient {
                                ui.spacing_mut().item_spacing.x = 3.0;
                                ui.label(RichText::new("·").color(akt_kopf_dim).size(22.0));
                                ui.spacing_mut().item_spacing.x = 8.0;
                                ui.add(
                                    egui::TextEdit::singleline(&mut pat.besonderheiten)
                                        .id(egui::Id::new("te_besonderheiten"))
                                        .frame(false)
                                        .font(FontId::proportional(24.0))
                                        .text_color(akt_kopf_text)
                                        .hint_text(
                                            RichText::new("BESONDERHEITEN")
                                                .color(akt_kopf_dim)
                                                .size(24.0),
                                        )
                                        .desired_width(380.0)
                                        .char_limit(30),
                                );
                            }

                            // Buttons + Name ganz rechts (right_to_left: × dann 🗑 dann Name)
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // × Schließen
                                let knopf_bg = Color32::from_black_alpha(75);
                                let (schliessen_rect, schliessen_antwort) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                                ui.painter().circle_filled(schliessen_rect.center(), 15.0, knopf_bg);
                                ui.painter().text(schliessen_rect.center(), egui::Align2::CENTER_CENTER, "×", FontId::proportional(22.0), akt_kopf_text);
                                if schliessen_antwort.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !ist_ungueltig { schliessen = true; }

                                ui.add_space(8.0);

                                // 🗑 Löschen
                                let (loeschen_rect, loeschen_antwort) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                                ui.painter().circle_filled(loeschen_rect.center(), 15.0, knopf_bg);
                                ui.painter().text(loeschen_rect.center(), egui::Align2::CENTER_CENTER, "🗑", FontId::proportional(15.0), akt_kopf_text);
                                if loeschen_antwort.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    *loeschen_bestaetigen = true;
                                }

                                ui.add_space(16.0);
                                let kopf_name = if let Some(pat) = &bett.patient {
                                    let n = name_anzeige_lang(pat);
                                    if !n.is_empty() { n } else { FELD_TITEL[aktuelles_feld].to_string() }
                                } else {
                                    FELD_TITEL[aktuelles_feld].to_string()
                                };
                                ui.label(RichText::new(kopf_name).color(akt_kopf_text).size(20.0).strong());
                                ui.add_space(8.0);
                            });
                        });
                    });

                    // ── Validierungs-Warnung ──
                    if ist_ungueltig {
                        let meldung = if nr_leer {
                            "⚠  Zimmernummer darf nicht leer sein."
                        } else if bet_leer {
                            "⚠  Bettbezeichnung darf nicht leer sein."
                        } else {
                            "⚠  Diese Zimmer+Bett-Kombination existiert bereits - bitte anpassen."
                        };
                        ui.add_space(4.0);
                        let warn_rahmen = egui::Frame::none()
                            .fill(Color32::from_rgb(180, 30, 30))
                            .inner_margin(egui::Margin::symmetric(12.0, 6.0));
                        warn_rahmen.show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new(meldung).size(13.0).color(Color32::WHITE).strong());
                        });
                        ui.add_space(4.0);
                    }

                    // ── Hauptbereich: Pfeile absolut, Inhalt mittig ──
                    let verfuegbar = ui.available_size();
                    let pfeil_b = 72.0;
                    let punkt_h = 60.0;
                    let inhalt_h = (verfuegbar.y - punkt_h).max(120.0);
                    let inhalt_b = verfuegbar.x - pfeil_b * 2.0;

                    // Gesamtfläche reservieren (Pfeile + Inhalt)
                    let (bereich_rect, _) = ui.allocate_exact_size(
                        Vec2::new(verfuegbar.x, inhalt_h),
                        egui::Sense::hover(),
                    );

                    // Pfeil links - absolut, immer vertikal mittig in bereich_rect
                    let links_rect = Rect::from_min_size(
                        bereich_rect.min,
                        Vec2::new(pfeil_b, inhalt_h),
                    );
                    let links_antwort = ui.interact(links_rect, ui.id().with("larr"), egui::Sense::click());
                    ui.painter().text(
                        links_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "‹",
                        FontId::proportional(64.0),
                        if links_antwort.hovered() { akt_akzent } else { Color32::from_gray(160) },
                    );
                    if links_antwort.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        vorheriges = true;
                    }

                    // Pfeil rechts - absolut
                    let rechts_rect = Rect::from_min_size(
                        Pos2::new(bereich_rect.max.x - pfeil_b, bereich_rect.min.y),
                        Vec2::new(pfeil_b, inhalt_h),
                    );
                    let rechts_antwort = ui.interact(rechts_rect, ui.id().with("rarr"), egui::Sense::click());
                    ui.painter().text(
                        rechts_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "›",
                        FontId::proportional(64.0),
                        if rechts_antwort.hovered() { akt_akzent } else { Color32::from_gray(160) },
                    );
                    if rechts_antwort.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        naechstes = true;
                    }

                    // Inhalt - kind_ui im mittleren Bereich
                    let innen_rect = Rect::from_min_size(
                        Pos2::new(bereich_rect.min.x + pfeil_b, bereich_rect.min.y),
                        Vec2::new(inhalt_b, inhalt_h),
                    );
                    let mut kind_ui = ui.new_child(egui::UiBuilder::new().max_rect(innen_rect).layout(egui::Layout::top_down(egui::Align::Min)));
                    egui::ScrollArea::vertical()
                        .max_height(inhalt_h)
                        .show(&mut kind_ui, |ui| {
                            ui.add_space(28.0);
                            let abstand = 32.0;
                            let feld_b = inhalt_b - abstand * 2.0;
                            if let Some(pat) = &mut bett.patient {
                                match aktuelles_feld {
                                    0 => {
                                        ui.add_space(abstand);
                                        ui.horizontal(|ui| {
                                            ui.add_space(10.0);
                                            ui.label(egui::RichText::new("PATIENT").size(20.0).color(Color32::from_gray(160)).strong());
                                        });

                                        // ANREDE / RECHTLICHES GESCHLECHT  +  BIOLOGISCHES GESCHLECHT
                                        ui.add_space(abstand);
                                        ui.columns(2, |cols| {
                                            // Linke Spalte: Anrede / Rechtliches Geschlecht
                                            cols[0].horizontal(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(RichText::new("ANREDE / RECHTLICHES GESCHLECHT").size(20.0).color(Color32::from_gray(160)).strong());
                                            });
                                            cols[0].add_space(8.0);
                                            cols[0].horizontal(|ui| {
                                                ui.add_space(10.0);
                                                for opt in ["Hr.", "Fr.", "–"] {
                                                    let sel = pat.anrede == opt;
                                                    let (bg, fg) = if sel { (akt_akzent, Color32::WHITE) } else { (Color32::TRANSPARENT, Color32::from_gray(160)) };
                                                    if ui.add(egui::Button::new(RichText::new(opt).size(36.0).strong().color(fg)).fill(bg)).clicked() {
                                                        pat.anrede = if sel { String::new() } else { opt.to_string() };
                                                    }
                                                    ui.add_space(8.0);
                                                }
                                            });
                                            // Rechte Spalte: Biologisches Geschlecht
                                            cols[1].horizontal(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(RichText::new("BIOLOGISCHES GESCHLECHT").size(20.0).color(Color32::from_gray(160)).strong());
                                            });
                                            cols[1].add_space(8.0);
                                            cols[1].horizontal(|ui| {
                                                ui.add_space(10.0);
                                                for opt in ["m", "w", "d"] {
                                                    let sel = pat.bio_geschlecht == opt;
                                                    let (bg, fg) = if sel { (akt_akzent, Color32::WHITE) } else { (Color32::TRANSPARENT, Color32::from_gray(160)) };
                                                    if ui.add(egui::Button::new(RichText::new(opt).size(36.0).strong().color(fg)).fill(bg)).clicked() {
                                                        pat.bio_geschlecht = if sel { String::new() } else { opt.to_string() };
                                                    }
                                                    ui.add_space(8.0);
                                                }
                                            });
                                        });

                                        // NACHNAME + VORNAME
                                        for (bezeichnung, wert) in [
                                            ("NACHNAME", &mut pat.nachname),
                                            ("VORNAME",  &mut pat.vorname),
                                        ] {
                                            let te_id = egui::Id::new(
                                                if bezeichnung == "NACHNAME" { "te_nachname" } else { "te_vorname" });
                                            ui.add_space(abstand);
                                            ui.horizontal(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(RichText::new(bezeichnung).size(20.0).color(Color32::from_gray(160)).strong());
                                            });
                                            ui.add_space(4.0);
                                            ui.add(
                                                egui::TextEdit::singleline(wert)
                                                    .id(te_id)
                                                    .desired_width(feld_b)
                                                    .frame(false)
                                                    .font(FontId::new(46.0, egui::FontFamily::Name("bold".into())))
                                                    .margin(egui::Margin::symmetric(10.0, 8.0)),
                                            );
                                        }

                                        // ALTER (Geburtsdatum)
                                        *geb_fehler = !pat.geburtsdatum.is_empty()
                                            && !geburtsdatum_gueltig(&pat.geburtsdatum);
                                        ui.add_space(abstand);
                                        let geb_label_farbe = if *geb_fehler { Color32::from_rgb(220, 50, 50) } else { Color32::from_gray(160) };
                                        ui.horizontal(|ui| {
                                            ui.add_space(10.0);
                                            ui.label(RichText::new("ALTER (tt.mm.jjjj)").size(20.0).color(geb_label_farbe).strong());
                                        });
                                        ui.add_space(4.0);
                                        let mut geb_edit = egui::TextEdit::singleline(&mut pat.geburtsdatum)
                                            .id(egui::Id::new("te_geburtsdatum"))
                                            .desired_width(feld_b)
                                            .frame(false)
                                            .font(FontId::new(46.0, egui::FontFamily::Name("bold".into())))
                                            .margin(egui::Margin::symmetric(10.0, 8.0))
                                            .char_limit(10)
                                            .hint_text(RichText::new("tt.mm.jjjj").color(Color32::from_gray(180)).size(46.0));
                                        if *geb_fehler { geb_edit = geb_edit.text_color(Color32::from_rgb(220, 50, 50)); }
                                        let geb_antwort = ui.add(geb_edit);
                                        if geb_antwort.lost_focus() { format_geburtsdatum(&mut pat.geburtsdatum); }

                                    }
                                    f => {
                                        let titel = match f {
                                            1 if self.psychiatrie_modus => "PSYCHIATRISCHE DIAGNOSE".to_string(),
                                            2 if self.psychiatrie_modus => "SOMATISCHE DIAGNOSE".to_string(),
                                            _ => FELD_TITEL[f].to_uppercase(),
                                        };
                                        let feld_wert = match f {
                                            1 => &mut pat.hdia,
                                            2 => &mut pat.ndia,
                                            3 => &mut pat.pflege,
                                            4 => &mut pat.info,
                                            5 => &mut pat.todo,
                                            _ => unreachable!(),
                                        };
                                        ui.add_space(abstand);
                                        // Titel links-bündig als Label, eingerückt wie TextEdit-Inhalt
                                        ui.horizontal(|ui| {
                                            ui.add_space(16.0);
                                            ui.label(egui::RichText::new(&titel).size(20.0).color(Color32::from_gray(160)).strong());
                                        });
                                        ui.add_space(6.0);
                                        let feld_h = (inhalt_h - 220.0).max(80.0);
                                        let te_id = egui::Id::new(match f {
                                            1 => "te_hdia", 2 => "te_ndia",
                                            3 => "te_pflege", 4 => "te_info",
                                            _ => "te_todo",
                                        });
                                        ui.add(
                                            egui::TextEdit::multiline(feld_wert)
                                                .id(te_id)
                                                .desired_width(feld_b)
                                                .min_size(Vec2::new(feld_b, feld_h))
                                                .frame(false)
                                                .font(FontId::new(46.0, egui::FontFamily::Name("bold".into())))
                                                .margin(egui::Margin::symmetric(16.0, 14.0))
                                                .char_limit(182),
                                        );
                                        // Zeilenumbrüche komplett verhindern (auch per Copy & Paste)
                                        // \r\n (Windows), \r (altes macOS), \n (Linux/macOS) → Leerzeichen
                                        if feld_wert.contains('\n') || feld_wert.contains('\r') {
                                            *feld_wert = feld_wert.replace(['\r', '\n'], " ");
                                        }
                                    }
                                }
                            }
                        });

                    // ── Punkte: 36px vom unteren Rand des Fensters (= pfeil_b/2) ──
                    // Berechnung aus bildschirm + fenster_h, nicht aus ui.max_rect() (unzuverlässig)
                    let punkt_my = bildschirm.center().y + fenster_h / 2.0 - pfeil_b * 0.75;
                    let punkt_mx = bildschirm.center().x;
                    let punkt_abstand = 20.0;
                    let gesamt_b = (FELD_ANZAHL - 1) as f32 * punkt_abstand;
                    let punkt_x0 = punkt_mx - gesamt_b / 2.0;
                    for i in 0..FELD_ANZAHL {
                        let center = Pos2::new(punkt_x0 + i as f32 * punkt_abstand, punkt_my);
                        let grundfarbe = if i == aktuelles_feld { akt_akzent } else { Color32::from_gray(150) };
                        ui.painter().circle_filled(center, 7.0, grundfarbe);
                        let punkt_rect = Rect::from_center_size(center, Vec2::splat(18.0));
                        let punkt_antwort = ui.interact(punkt_rect, ui.id().with(("dot", i)), egui::Sense::click());
                        if punkt_antwort.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            springe_zu = Some(i);
                        }
                    }
                });

            if schliessen {
                if self.geb_fehler {
                    if let Some((zi, bi)) = self.bearbeitung {
                        if let Some(pat) = &mut self.station.zimmer[zi].betten[bi].patient {
                            pat.geburtsdatum.clear();
                        }
                    }
                    self.geb_fehler = false;
                }
                self.loeschen_bestaetigen = false;
                self.bearbeitung = None;
                self.speichern_nach_schliessen = true;
            }
            if naechstes  { self.bearbeitungsfeld = (self.bearbeitungsfeld + 1) % FELD_ANZAHL; }
            if vorheriges { self.bearbeitungsfeld = (self.bearbeitungsfeld + FELD_ANZAHL - 1) % FELD_ANZAHL; }
            if let Some(f) = springe_zu { self.bearbeitungsfeld = f; }

            // ── Bestätigungsdialog Löschen ──
            if self.loeschen_bestaetigen {
                let taste_j   = ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::Enter));
                let taste_nein  = ctx.input(|i| i.key_pressed(egui::Key::N) || i.key_pressed(egui::Key::Escape));
                egui::Window::new("confirm_delete_dlg")
                    .title_bar(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(
                        egui::Frame::none()
                            .fill(akt_fenster_bg)
                            .stroke(Stroke::new(2.0, akt_akzent))
                            .rounding(Rounding::same(12.0))
                            .inner_margin(egui::Margin::same(24.0)),
                    )
                    .show(ctx, |ui| {
                        ui.label(
                            RichText::new("Alle Felder dieses Bettes wirklich löschen?")
                                .size(20.0)
                                .strong(),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(
                                RichText::new("  Ja  ").size(18.0).color(akt_kopf_text)
                            ).fill(akt_akzent).rounding(Rounding::same(8.0))).clicked() || taste_j {
                                if let Some((zi, bi)) = self.bearbeitung {
                                    self.station.zimmer[zi].betten[bi].patient = Some(Patient::default());
                                }
                                self.loeschen_bestaetigen = false;
                                self.geb_fehler = false;
                            }
                            ui.add_space(12.0);
                            if ui.add(egui::Button::new(
                                RichText::new("  Nein  ").size(18.0)
                            ).rounding(Rounding::same(8.0))).clicked() || taste_nein {
                                self.loeschen_bestaetigen = false;
                            }
                        });
                    });
            }

        }

        // ── Einstellungen-Dialog ──
        if self.einstellungen_offen {
            let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            egui::Window::new("einstellungen_dlg")
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::none()
                        .fill(akt_fenster_bg)
                        .stroke(Stroke::new(2.0, akt_akzent))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(24.0)),
                )
                .show(ctx, |ui| {
                    // Warnung
                    let warn_farbe = Color32::from_rgb(220, 60, 60);
                    ui.label(
                        RichText::new("⚠  ACHTUNG: Zimmer löschen entfernt alle Patientendaten unwiderruflich!")
                            .size(13.0)
                            .color(warn_farbe)
                            .strong(),
                    );
                    ui.add_space(14.0);

                    ui.label(RichText::new("Einstellungen").size(20.0).strong());
                    ui.add_space(16.0);

                    // Zimmeranzahl
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Anzahl Zimmer:").size(16.0));
                        ui.add_space(12.0);
                        if ui.add(
                            egui::Button::new(RichText::new("  −  ").size(18.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() && self.einstellungen_zimmer_anzahl > 1 {
                            self.einstellungen_zimmer_anzahl -= 1;
                        }
                        ui.label(RichText::new(format!("  {}  ", self.einstellungen_zimmer_anzahl)).size(22.0).strong());
                        if ui.add(
                            egui::Button::new(RichText::new("  +  ").size(18.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() && self.einstellungen_zimmer_anzahl < 50 {
                            self.einstellungen_zimmer_anzahl += 1;
                        }
                    });
                    ui.add_space(16.0);

                    // Psychiatrie-Modus
                    ui.separator();
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Psychiatrie-Modus:").size(16.0));
                        ui.add_space(12.0);
                        let psych_btn_text = if self.einstellungen_psychiatrie {
                            RichText::new("  Ein  ").size(15.0).color(akt_kopf_text)
                        } else {
                            RichText::new("  Aus  ").size(15.0)
                        };
                        let psych_btn = if self.einstellungen_psychiatrie {
                            egui::Button::new(psych_btn_text).fill(akt_akzent).rounding(Rounding::same(6.0))
                        } else {
                            egui::Button::new(psych_btn_text).rounding(Rounding::same(6.0))
                        };
                        if ui.add(psych_btn).clicked() {
                            self.einstellungen_psychiatrie = !self.einstellungen_psychiatrie;
                        }
                        ui.add_space(10.0);
                        ui.label(RichText::new("(HDIA > PDIA, NDIA > SDIA)").size(12.0).color(ui.visuals().weak_text_color()));
                    });
                    ui.add_space(20.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("  Übernehmen  ").size(16.0).color(akt_kopf_text))
                                .fill(akt_akzent)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            let n = self.einstellungen_zimmer_anzahl;
                            self.zimmer_anzahl_setzen(n);
                            self.psychiatrie_modus = self.einstellungen_psychiatrie;
                            self.einstellungen_offen = false;
                        }
                        ui.add_space(12.0);
                        if ui.add(
                            egui::Button::new(RichText::new("  Abbrechen  ").size(16.0))
                                .rounding(Rounding::same(8.0))
                        ).clicked() || esc {
                            self.einstellungen_offen = false;
                        }
                    });
                });
        }

        // ── Konflikt-Popup ──────────────────────────────────────────────────
        if !self.konflikte.is_empty() {
            let konflikt = &self.konflikte[0];
            let titel = format!("Konflikt: Zimmer {} · Bett {}", konflikt.zimmer_nr, konflikt.bett_buchstabe);
            let verbleibend = self.konflikte.len();
            let lokal  = konflikt.lokal.clone();
            let disk   = konflikt.disk.clone();
            let zi     = konflikt.zi;
            let bi     = konflikt.bi;

            let mut meine = false;
            let mut andere = false;

            egui::Window::new("konflikt_dlg")
                .title_bar(false)
                .resizable(false)
                .min_width(860.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .frame(
                    egui::Frame::none()
                        .fill(akt_fenster_bg)
                        .stroke(Stroke::new(2.0, akt_akzent))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(24.0)),
                )
                .show(ctx, |ui| {
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::Escape)) { meine  = true; }
                    if ctx.input(|i| i.key_pressed(egui::Key::N)) { andere = true; }

                    ui.label(RichText::new(&titel).size(18.0).strong());
                    ui.add_space(4.0);
                    let warn = Color32::from_rgb(210, 140, 0);
                    ui.label(RichText::new("Die Datei wurde zwischenzeitlich von jemand anderem geändert. Welche Version soll behalten werden?")
                        .size(13.0).color(warn));
                    if verbleibend > 1 {
                        ui.label(RichText::new(format!("({} Konflikte insgesamt)", verbleibend))
                            .size(12.0).color(ui.visuals().weak_text_color()));
                    }
                    ui.add_space(12.0);

                    let pat_text = |p: &Option<Patient>| -> String {
                        match p {
                            None => "(leer)".to_string(),
                            Some(p) => {
                                let mut t = String::new();
                                if !p.nachname.is_empty() || !p.vorname.is_empty() {
                                    t.push_str(&format!("{}, {}\n", p.nachname, p.vorname));
                                }
                                if !p.geburtsdatum.is_empty()  { t.push_str(&format!("Geb: {}\n",    p.geburtsdatum)); }
                                if !p.besonderheiten.is_empty(){ t.push_str(&format!("Besond: {}\n", p.besonderheiten)); }
                                if !p.hdia.is_empty()          { t.push_str(&format!("HDIA: {}\n",   p.hdia.replace('\n', " | "))); }
                                if !p.ndia.is_empty()          { t.push_str(&format!("NDIA: {}\n",   p.ndia.replace('\n', " | "))); }
                                if !p.pflege.is_empty()        { t.push_str(&format!("PFLEGE: {}\n", p.pflege.replace('\n', " | "))); }
                                if !p.info.is_empty()          { t.push_str(&format!("INFO: {}\n",   p.info.replace('\n', " | "))); }
                                if !p.todo.is_empty()          { t.push_str(&format!("TODO: {}\n",   p.todo.replace('\n', " | "))); }
                                if t.is_empty() { "(keine Daten)".to_string() } else { t.trim_end().to_string() }
                            }
                        }
                    };

                    ui.columns(2, |cols| {
                        cols[0].group(|ui| {
                            ui.label(RichText::new("Meine Änderungen").size(14.0).strong().color(akt_akzent));
                            ui.add_space(6.0);
                            ui.label(RichText::new(pat_text(&lokal)).size(13.0));
                        });

                        cols[1].group(|ui| {
                            ui.label(RichText::new("Andere Änderungen").size(14.0).strong());
                            ui.add_space(6.0);
                            ui.label(RichText::new(pat_text(&disk)).size(13.0));
                        });
                    });

                    ui.add_space(12.0);
                    ui.columns(2, |cols| {
                        cols[0].with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("  Meine behalten  ").size(15.0).color(akt_kopf_text))
                                .fill(akt_akzent).rounding(Rounding::same(8.0))).clicked() {
                                meine = true;
                            }
                        });
                        cols[1].with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("  Andere übernehmen  ").size(15.0))
                                .rounding(Rounding::same(8.0))).clicked() {
                                andere = true;
                            }
                        });
                    });
                });

            if andere {
                self.station.zimmer[zi].betten[bi].patient = disk;
            }
            if meine || andere {
                self.konflikte.remove(0);
                if self.konflikte.is_empty() && self.speichern_nach_konflikt {
                    self.speichern_nach_konflikt = false;
                    self.tatsaechlich_speichern();
                }
            }
        }

        // ── Bestätigungsdialog Neu ──
        if self.neu_bestaetigen {
            let taste_ja   = ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::Enter));
            let taste_nein = ctx.input(|i| i.key_pressed(egui::Key::N) || i.key_pressed(egui::Key::Escape));
            egui::Window::new("confirm_neu_dlg")
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::none()
                        .fill(akt_fenster_bg)
                        .stroke(Stroke::new(2.0, akt_akzent))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(24.0)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("Alle Daten löschen und neu beginnen?")
                            .size(20.0)
                            .strong(),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(
                            RichText::new("  Ja  ").size(18.0).color(akt_kopf_text)
                        ).fill(akt_akzent).rounding(Rounding::same(8.0))).clicked() || taste_ja {
                            self.neu();
                            self.neu_bestaetigen = false;
                        }
                        ui.add_space(12.0);
                        if ui.add(egui::Button::new(
                            RichText::new("  Nein  ").size(18.0)
                        ).rounding(Rounding::same(8.0))).clicked() || taste_nein {
                            self.neu_bestaetigen = false;
                        }
                    });
                });
        }

        // ── Bestätigungsdialog Beenden ──
        if self.beenden_bestaetigen {
            let taste_j    = ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::Enter));
            let taste_nein = ctx.input(|i| i.key_pressed(egui::Key::N) || i.key_pressed(egui::Key::Escape));
            egui::Window::new("confirm_exit_dlg")
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::none()
                        .fill(akt_fenster_bg)
                        .stroke(Stroke::new(2.0, akt_akzent))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(24.0)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("Wirklich beenden?")
                            .size(20.0)
                            .strong(),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(
                            RichText::new("  Ja  ").size(18.0).color(akt_kopf_text)
                        ).fill(akt_akzent).rounding(Rounding::same(8.0))).clicked() || taste_j {
                            self.stilles_speichern();
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(12.0);
                        if ui.add(egui::Button::new(
                            RichText::new("  Nein  ").size(18.0)
                        ).rounding(Rounding::same(8.0))).clicked() || taste_nein {
                            self.beenden_bestaetigen = false;
                        }
                    });
                });
        }

        // ── Über-Dialog ──────────────────────────────────────────────────────
        if self.ueber_dialog_offen {
            let mut open = true;
            egui::Window::new("Über mz-hyprnurs")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(320.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(12.0);
                        ui.label(RichText::new("mz-hyprnurs").strong().size(24.0));
                        ui.add_space(4.0);
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(16.0);
                        if ui.add(egui::Button::new("www.marcelzimmer.de").min_size(egui::vec2(220.0, 32.0))).clicked() {
                            url_oeffnen("https://www.marcelzimmer.de");
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new("X @marcelzimmer").min_size(egui::vec2(220.0, 32.0))).clicked() {
                            url_oeffnen("https://www.x.com/marcelzimmer");
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new("GitHub @marcelzimmer").min_size(egui::vec2(220.0, 32.0))).clicked() {
                            url_oeffnen("https://github.com/marcelzimmer");
                        }
                        ui.add_space(12.0);
                    });
                });
            if !open { self.ueber_dialog_offen = false; }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stilles_speichern();
    }
}

// ── *text*-Segmente splitten: Vec<(String, bool)> - bool = fett ──────────────
fn stern_segmente(text: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(pos) = rest.find('*') {
            if pos > 0 { result.push((rest[..pos].to_string(), false)); }
            let danach = &rest[pos + 1..];
            if let Some(ende) = danach.find('*') {
                result.push((danach[..ende].to_string(), true));
                rest = &danach[ende + 1..];
            } else {
                result.push((rest[pos..].to_string(), false));
                break;
            }
        } else {
            result.push((rest.to_string(), false));
            break;
        }
    }
    result
}

// ── Formatierter Text: *wort* = rot/fett ─────────────────────────────────────
fn galley_formatieren(
    ui: &egui::Ui,
    text: &str,
    font_id: FontId,
    default_color: Color32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    use egui::text::{LayoutJob, TextFormat};
    let mut job = LayoutJob::default();
    job.wrap.max_width = max_width;

    let fett_familie = egui::FontFamily::Name("bold".into());
    let normal   = TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() };
    let hervorgehoben = TextFormat { font_id: FontId::new(font_id.size, fett_familie), color: Color32::from_rgb(220, 40, 40), ..Default::default() };

    let mut rest = text;
    while !rest.is_empty() {
        let naechstes = rest.find('*').unwrap_or(usize::MAX);

        if naechstes == usize::MAX {
            job.append(rest, 0.0, normal.clone());
            break;
        }

        if naechstes > 0 {
            job.append(&rest[..naechstes], 0.0, normal.clone());
        }

        let danach = &rest[naechstes + 1..];
        if let Some(ende) = danach.find('*') {
            job.append(&danach[..ende], 0.0, hervorgehoben.clone());
            rest = &danach[ende + 1..];
        } else {
            job.append(&rest[naechstes..], 0.0, normal.clone());
            break;
        }
    }

    ui.fonts(|f| f.layout_job(job))
}

// ── Bett-Karte (Übersicht, schreibgeschützt) ──────────────────────────────────
// Gibt (geklicktes_feld, karte_rect) zurück.
// geklicktes_feld: None = nicht geklickt, Some(0) = Header/Patient, Some(1..5) = HDIA..INFO
fn bett_karte_zeichnen(ui: &mut egui::Ui, zimmer_nummer: &str, bett: &Bett, ausgewaehlt: bool, farben: &AktFarben, psych: bool) -> (Option<usize>, Rect) {
    let felder_psych = ["PDIA", "SDIA", "NURS", "INFO", "TODO"];
    let felder_normal = ["HDIA", "NDIA", "NURS", "INFO", "TODO"];
    let felder: &[&str; 5] = if psych { &felder_psych } else { &felder_normal };

    let karte_h = KOPF_H + FELD_H * felder.len() as f32 + 8.0;
    let (karte_rect, antwort) =
        ui.allocate_exact_size(Vec2::new(KARTE_B, karte_h), egui::Sense::click());

    let grundfarbe = if antwort.hovered() { farben.karten_hover } else { farben.karten_bg };

    let trenn_x = karte_rect.min.x + KOPF_ZELLE_B;
    let zeichner = ui.painter();

    // Schatten
    zeichner.rect_filled(
        karte_rect.translate(Vec2::new(2.0, 4.0)),
        Rounding::same(KARTE_RUND),
        Color32::from_black_alpha(18),
    );
    zeichner.rect_filled(
        karte_rect.translate(Vec2::new(1.0, 2.0)),
        Rounding::same(KARTE_RUND),
        Color32::from_black_alpha(10),
    );

    // Hintergrund
    zeichner.rect_filled(karte_rect, Rounding::same(KARTE_RUND), grundfarbe);

    // Kopfblock Zimmer
    let zimmer_zelle = Rect::from_min_size(karte_rect.min, Vec2::new(KOPF_ZELLE_B, KOPF_H));
    zeichner.rect_filled(
        zimmer_zelle,
        Rounding { nw: KARTE_RUND, ne: 0.0, sw: 0.0, se: 0.0 },
        farben.akzent,
    );

    // Kopfblock Bett
    let bett_zelle = Rect::from_min_size(
        Pos2::new(trenn_x, karte_rect.min.y),
        Vec2::new(KOPF_ZELLE_B, KOPF_H),
    );
    zeichner.rect_filled(bett_zelle, Rounding::ZERO, farben.akzent);

    // Pipe
    zeichner.line_segment(
        [
            Pos2::new(trenn_x, karte_rect.min.y + 7.0),
            Pos2::new(trenn_x, karte_rect.min.y + KOPF_H - 7.0),
        ],
        Stroke::new(1.5, farben.kopf_dim),
    );

    // Texte Kopfblock (Fake-Bold)
    for dx in [0.0_f32, 0.6] {
        zeichner.text(
            zimmer_zelle.center() + Vec2::new(dx, 0.0),
            egui::Align2::CENTER_CENTER,
            zimmer_nummer,
            FontId::proportional(17.0),
            farben.kopf_text,
        );
        zeichner.text(
            bett_zelle.center() + Vec2::new(dx, 0.0),
            egui::Align2::CENTER_CENTER,
            &bett.buchstabe,
            FontId::proportional(17.0),
            farben.kopf_text,
        );
    }

    // Patientenname
    let name_x = karte_rect.min.x + KOPF_ZELLE_B * 2.0 + 10.0;
    let kopf_my = karte_rect.min.y + KOPF_H / 2.0;
    if let Some(pat) = &bett.patient {
        let name = name_anzeige(pat);
        let name_gal =
            zeichner.layout_no_wrap(name.clone(), FontId::proportional(17.0), farben.text);
        zeichner.text(
            Pos2::new(name_x, kopf_my),
            egui::Align2::LEFT_CENTER,
            &name,
            FontId::proportional(17.0),
            farben.text,
        );
        if !pat.besonderheiten.is_empty() {
            let besond_x = name_x + name_gal.size().x + 16.0;
            let besond_gal = galley_formatieren(
                ui,
                &pat.besonderheiten,
                FontId::proportional(15.0),
                farben.bezeichnung,
                f32::INFINITY,
            );
            let besond_y = kopf_my - besond_gal.size().y / 2.0;
            zeichner.galley(Pos2::new(besond_x, besond_y), besond_gal, farben.bezeichnung);
        }
    }

    // Rahmen
    let rahmen = if ausgewaehlt {
        Stroke::new(3.0, farben.akzent)
    } else if antwort.hovered() {
        Stroke::new(1.5, farben.akzent)
    } else {
        Stroke::new(0.5, farben.trennlinie)
    };
    zeichner.rect_stroke(karte_rect, Rounding::same(KARTE_RUND), rahmen);

    // Trennlinie Kopf/Tabelle
    let tabelle_oben = karte_rect.min.y + KOPF_H;
    zeichner.line_segment(
        [Pos2::new(karte_rect.min.x, tabelle_oben), Pos2::new(karte_rect.max.x, tabelle_oben)],
        Stroke::new(1.0, farben.trennlinie),
    );

    // Senkrechte Tabellen-Trennlinie
    zeichner.line_segment(
        [Pos2::new(trenn_x, tabelle_oben), Pos2::new(trenn_x, karte_rect.max.y - 4.0)],
        Stroke::new(1.0, farben.trennlinie),
    );

    // Tabellenzeilen
    let feld_werte: Vec<&str> = if let Some(pat) = &bett.patient {
        vec![pat.hdia.as_str(), pat.ndia.as_str(), pat.pflege.as_str(), pat.info.as_str(), pat.todo.as_str()]
    } else {
        vec!["", "", "", "", ""]
    };

    let inhalt_b = karte_rect.max.x - trenn_x - 12.0;
    let mut zeile_y = tabelle_oben;

    for (i, (bezeichnung, wert)) in felder.iter().zip(feld_werte.iter()).enumerate() {
        let zeile_h = FELD_H;
        let bezeichnung_cx = karte_rect.min.x + KOPF_ZELLE_B / 2.0;
        zeichner.text(
            Pos2::new(bezeichnung_cx, zeile_y + zeile_h / 2.0),
            egui::Align2::CENTER_CENTER,
            bezeichnung,
            FontId::monospace(15.0),
            farben.bezeichnung,
        );

        if !wert.is_empty() {
            let galley = galley_formatieren(
                ui,
                wert,
                FontId::proportional(16.0),
                farben.text,
                inhalt_b,
            );
            let zellen_rect = Rect::from_min_size(
                Pos2::new(trenn_x, zeile_y),
                Vec2::new(karte_rect.max.x - trenn_x, zeile_h - 1.0),
            );
            let geclippt = zeichner.with_clip_rect(zellen_rect);
            geclippt.galley(Pos2::new(trenn_x + 8.0, zeile_y + 6.0), galley, farben.text);
        }

        if i < felder.len() - 1 {
            let trenn_y = zeile_y + zeile_h;
            zeichner.line_segment(
                [
                    Pos2::new(karte_rect.min.x + 10.0, trenn_y),
                    Pos2::new(karte_rect.max.x - 10.0, trenn_y),
                ],
                Stroke::new(0.5, farben.trennlinie),
            );
        }

        zeile_y += zeile_h;
    }

    let antwort = antwort.on_hover_cursor(egui::CursorIcon::PointingHand);
    let geklicktes_feld = if antwort.clicked() {
        let feld = if let Some(pos) = antwort.interact_pointer_pos() {
            let rel_y = pos.y - karte_rect.min.y;
            if rel_y < KOPF_H {
                0 // Patient
            } else {
                // Akkumuliere Zeilenhöhen um korrekte Zeile zu finden
                let mut y = KOPF_H;
                let mut idx = 0;
                for f in 0..felder.len() {
                    let rh = FELD_H;
                    if rel_y < y + rh { idx = f + 1; break; }
                    y += rh;
                    idx = f + 1;
                }
                idx
            }
        } else {
            0
        };
        Some(feld.min(FELD_ANZAHL - 1))
    } else {
        None
    };
    (geklicktes_feld, karte_rect)
}
