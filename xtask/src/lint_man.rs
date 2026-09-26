//! Checks the `-sys` crates against the section 9 manual pages of a gate.
//!
//! Two things are verified. First, every doc comment of the form
//! `` `name(9F)`. <level>. `` names a page that exists under
//! `usr/src/man/man9{e,f,s,p}` and whose interface level is not marked
//! obsolete. Second, with `--coverage`, every function the manual documents
//! in section 9F is compared with the `pub fn` declarations of the `-sys`
//! crates, so that the gap between "the DDI" and "what is bound" is a list
//! rather than a feeling. Pages whose interface level is obsolete are left
//! out of the comparison, and pages that belong to a subsystem with its own
//! kernel module are reported separately, because they are planned as their
//! own crates. With `--mapfile-gaps`, the documented functions of the core
//! crate that `ddi.mapfile` does not list are printed; they are what a
//! driver linked with `-z defs` against that mapfile trips over, and the
//! input for an upstream mapfile patch.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// A section 9 manual page as the lint sees it.
struct Page {
    /// The names the page documents, from its NAME section and from the
    /// function prototypes in its SYNOPSIS.
    names: Vec<String>,
    /// The names that carry a prototype in the SYNOPSIS. Coverage counts
    /// these, because a NAME line also lists page titles and aliases.
    funcs: Vec<String>,
    /// The whole INTERFACE LEVEL section as one line of text, or empty.
    level: String,
    /// The Interface Stability attribute, used when there is no level.
    stability: String,
    /// The first header the SYNOPSIS includes, or empty.
    header: String,
}

impl Page {
    /// The text a doc comment must quote after `` `name(9X)`. `` for this
    /// page and name. Pages that give each function its own level, such as
    /// `esballoc(9F)`, are handled by picking the entry for `name`.
    fn canonical(&self, name: &str) -> String {
        if !self.level.is_empty() {
            let tag = format!("{name}():");
            if let Some(i) = self.level.find(&tag) {
                let rest = self.level[i + tag.len()..].trim_start();
                let end = next_entry(rest).unwrap_or(rest.len());
                return rest[..end].trim().trim_end_matches('.').to_string();
            }
            return first_sentence(&self.level).to_string();
        }
        if !self.stability.is_empty() {
            return format!("Interface stability: {}", self.stability);
        }
        "No interface level given".to_string()
    }

    /// Whether the page marks `name` obsolete or private, either for the
    /// whole page or in a later sentence. A later sentence that names
    /// functions applies to those; one that names none, such as "These
    /// interfaces are obsolete", applies to the whole page.
    fn obsolete(&self, name: &str) -> bool {
        let own = self.canonical(name).to_ascii_lowercase();
        if own.contains("obsolete") || is_private(&own) {
            return true;
        }
        if self.level.is_empty() && self.stability.eq_ignore_ascii_case("obsolete") {
            return true;
        }
        let call = format!("{name}()");
        sentences(&self.level).skip(1).any(|s| {
            let l = s.to_ascii_lowercase();
            (l.contains("obsolete") || is_private(&l)) && (s.contains(&call) || !s.contains("()"))
        })
    }
}

/// Whether a level text marks an interface as private to the kernel. Pages
/// such as `usba_hcdi(9E)` say "Volatile - illumos USB HCD private" and that
/// the interface is not part of the stable DDI; they document it for host
/// controller drivers at the Volatile level, so they are bound. Only the
/// phrase "private DDI function", used by `devfs_clean(9F)` and the
/// `esballoc_wait` variants, withdraws an interface from drivers.
fn is_private(level_lower: &str) -> bool {
    level_lower.contains("private ddi function") || level_lower.trim() == "interface stability: private"
}

/// The first sentence of a level text, without its full stop.
fn first_sentence(s: &str) -> &str {
    sentences(s).next().unwrap_or("").trim_end_matches('.')
}

fn sentences(s: &str) -> impl Iterator<Item = &str> {
    s.split(". ").map(str::trim)
}

/// Finds where the next `other():` entry starts in a per-function level list.
fn next_entry(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    while let Some(off) = s[i..].find("():") {
        let colon = i + off;
        let mut j = colon;
        while j > 0 && (b[j - 1].is_ascii_alphanumeric() || b[j - 1] == b'_') {
            j -= 1;
        }
        if j < colon && (j == 0 || b[j - 1] == b' ') {
            return Some(j);
        }
        i = colon + 3;
    }
    None
}

/// Prefixes of function families that resolve from a kernel module other
/// than `genunix` or `unix`, and therefore belong to a crate of their own.
const SUBSYSTEM_PREFIXES: &[(&str, &str)] = &[
    ("scsi_", "ddi-scsa-sys (misc/scsi)"),
    ("usb_", "ddi-usba-sys (misc/usba)"),
    ("usba_", "ddi-usba-sys (misc/usba)"),
    ("mac_", "ddi-mac-sys (misc/mac)"),
    ("net_", "ddi-neti-sys (misc/neti)"),
    ("hook_", "ddi-neti-sys (misc/hook)"),
    ("csx_", "ddi-pcmcia-sys (genunix)"),
    ("ksensor_", "ddi-ksensor-sys (misc/ksensor)"),
    ("ktest_", "ddi-ktest-sys (drv/ktest)"),
    ("sas_phymap_", "SCSA phymap, Volatile, in ddi-scsa-sys"),
    ("gld", "ddi-gld-sys (misc/gld)"),
    ("dl", "DLPI acknowledgements, in ddi-mac-sys"),
];

/// Documented names that are C macros with no exported symbol, and why a
/// `-sys` crate leaves them out. Each is either replaced by a Rust language
/// feature or provided by the safe crate on top of the functions it expands
/// to.
const MACROS: &[(&str, &str)] = &[
    ("ASSERT", "use assert! and debug_assert!"),
    ("STRUCT_", "the ioctl data-model helpers, ddi::ioctl::IoctlArg::model"),
    ("SIZEOF_", "the ioctl data-model helpers, ddi::ioctl::IoctlArg::model"),
    ("AVL_NEXT", "expands to avl_walk, which is bound"),
    ("AVL_PREV", "expands to avl_walk, which is bound"),
    ("CRED", "expands to ddi_get_cred, which is bound"),
    ("va_", "C varargs; Rust callers use core::fmt"),
    ("stdc_", "the type-generic forms; Rust has leading_zeros and friends on every integer"),
    ("KT_", "ktest assertion macros"),
];

/// Names a page documents that the running kernel does not export at all.
/// They are manual bugs rather than binding gaps.
/// `min` and `max` are defined in `subr.c` but absent from the symbol table
/// of the kernel `xtask kernel-check` was run against.
const NOT_IN_KERNEL: &[&str] = &[
    "atomic_inc_ptr",
    "atomic_inc_ptr_nv",
    "atomic_dec_ptr",
    "atomic_dec_ptr_nv",
    "id_allocff_specific_nosleep",
    "min",
    "max",
    "net_netidtozonid",
    "mac_prop_info_set_default_uint16",
];

pub fn run(args: &[String]) -> Result<(), String> {
    let mut gate = std::env::var("ILLUMOS_GATE").ok().map(PathBuf::from);
    let mut coverage = false;
    let mut markdown = false;
    let mut fix = false;
    let mut mapfile_gaps = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--gate" => {
                i += 1;
                gate = Some(PathBuf::from(args.get(i).ok_or("--gate needs a path")?));
            }
            "--coverage" => coverage = true,
            "--fix" => fix = true,
            "--mapfile-gaps" => mapfile_gaps = true,
            "--markdown" => {
                coverage = true;
                markdown = true;
            }
            a => return Err(format!("unknown argument {a}")),
        }
        i += 1;
    }
    let gate = gate.ok_or("set ILLUMOS_GATE or pass --gate <path to illumos-gate>")?;
    let man = gate.join("usr/src/man");
    if !man.is_dir() {
        return Err(format!("{} is not a manual directory", man.display()));
    }
    let root = super::workspace_root();

    let mut pages: BTreeMap<(String, String), Page> = BTreeMap::new();
    for sec in ["9e", "9f", "9s", "9p"] {
        let dir = man.join(format!("man{sec}"));
        for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            if path.extension().and_then(|e| e.to_str()) != Some(sec) {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            pages.insert((sec.to_uppercase(), stem.to_string()), parse_page(&text));
        }
    }

    // Every name a page documents, so that `kmem_free(9F)` resolves to the
    // `kmem_alloc` page and `mutex_enter(9F)` to `mutex`.
    let mut index: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
    for ((sec, stem), p) in &pages {
        index.entry((sec.clone(), stem.clone())).or_insert((sec.clone(), stem.clone()));
        for n in &p.names {
            index.entry((sec.clone(), n.clone())).or_insert((sec.clone(), stem.clone()));
        }
    }

    // The Manlink files map further names to pages, one page per line
    // followed by its links indented with a tab.
    for sec in ["9e", "9f", "9s", "9p"] {
        let Ok(text) = fs::read_to_string(man.join(format!("man{sec}/Manlink"))) else { continue };
        let upper = sec.to_uppercase();
        let mut page: Option<String> = None;
        for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let stem = line.trim().trim_end_matches(&format!(".{sec}")).to_string();
            if line.starts_with('\t') || line.starts_with(' ') {
                if let Some(pg) = &page {
                    index.entry((upper.clone(), stem)).or_insert((upper.clone(), pg.clone()));
                }
            } else {
                page = Some(stem);
            }
        }
    }

    let mut errors = 0;
    let mut referenced: BTreeSet<(String, String)> = BTreeSet::new();
    let mut bound: BTreeSet<String> = BTreeSet::new();
    // Functions of the core crate that cite a section 9 page, for the
    // comparison with `ddi.mapfile`.
    let mut core_documented: BTreeSet<String> = BTreeSet::new();
    let mut fixed = 0;
    for src in sys_sources(&root)? {
        let text = fs::read_to_string(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        let rel = src.strip_prefix(&root).unwrap_or(&src).display().to_string();
        // The first doc line of the block being read, if it cites a page.
        let mut pending: Option<(usize, String, String, String)> = None;
        let mut prev_doc = false;
        for i in 0..lines.len() {
            let line = lines[i].clone();
            let t = line.trim_start();
            let is_doc = t.starts_with("///");
            if is_doc && !prev_doc {
                pending = doc_reference(t).map(|(n, sec, q)| (i, n.to_string(), sec.to_string(), q.to_string()));
            }
            prev_doc = is_doc;
            if is_doc || t.starts_with("#[") {
                continue;
            }
            let item = item_name(t);
            if let Some(name) = item.filter(|_| t.contains("fn ")) {
                bound.insert(name.to_string());
            }
            let Some((at, cited, sec, quoted)) = pending.take() else { continue };
            if src.components().any(|c| c.as_os_str() == "ddi-sys") && t.contains("fn ") {
                core_documented.insert(item.unwrap_or(&cited).to_string());
            }
            let item = item.unwrap_or(&cited);
            let key = (sec.clone(), cited.clone());
            referenced.insert(key.clone());
            let where_ = format!("{rel}:{}", at + 1);
            let Some(p) = index.get(&key).and_then(|k| pages.get(k)) else {
                errors += 1;
                eprintln!("{where_}: no manual page documents {cited}({sec})");
                continue;
            };
            // Functions are judged by their own name, so that a page which
            // obsoletes some of its functions does not condemn the others.
            let subject = if p.names.iter().any(|n| n == item) { item } else { cited.as_str() };
            if p.obsolete(subject) {
                errors += 1;
                eprintln!("{where_}: {subject} is marked obsolete by {cited}({sec}): {}", p.level);
                continue;
            }
            let want = p.canonical(subject);
            if normalise(&quoted) != normalise(&want) {
                if fix {
                    let indent: String = lines[at].chars().take_while(|c| c.is_whitespace()).collect();
                    lines[at] = format!("{indent}/// `{cited}({sec})`. {want}.");
                    fixed += 1;
                } else {
                    errors += 1;
                    eprintln!("{where_}: {cited}({sec}) quotes \"{quoted}\" but the page says \"{want}\"");
                }
            }
        }
        if fix {
            let mut out = lines.join("\n");
            out.push('\n');
            if out != text {
                fs::write(&src, out).map_err(|e| format!("{}: {e}", src.display()))?;
            }
        }
    }
    if fix {
        println!("lint-man: rewrote {fixed} first lines to the canonical level text");
    }
    println!(
        "lint-man: {} doc references checked, {} bound functions, {} errors",
        referenced.len(),
        bound.len(),
        errors
    );

    if coverage {
        report_coverage(&pages, &bound, markdown);
    }
    if mapfile_gaps {
        let mapfile = gate.join("usr/src/uts/common/mapfiles/ddi.mapfile");
        let text = fs::read_to_string(&mapfile).map_err(|e| format!("{}: {e}", mapfile.display()))?;
        let listed: BTreeSet<&str> = text
            .lines()
            .filter_map(|l| l.trim().split_once('{').map(|(n, _)| n.trim()))
            .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .collect();
        let gaps: Vec<&String> = core_documented.iter().filter(|n| !listed.contains(n.as_str())).collect();
        println!("documented DDI functions bound in ddi-sys but absent from ddi.mapfile: {}", gaps.len());
        for chunk in gaps.chunks(8) {
            println!("  {}", chunk.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));
        }
    }
    if errors > 0 { Err(format!("{errors} manual reference errors")) } else { Ok(()) }
}

/// Splits a first doc line of the form `` /// `name(9F)`. Text. `` into
/// the name, the section and the quoted text without its final full stop.
fn doc_reference(t: &str) -> Option<(&str, &str, &str)> {
    let rest = t.strip_prefix("/// `")?;
    let (name, rest) = rest.split_once('(')?;
    let (sec, rest) = rest.split_once(")`")?;
    if !matches!(sec, "9E" | "9F" | "9S" | "9P") || name.contains(' ') {
        return None;
    }
    let quoted = rest.strip_prefix('.').unwrap_or(rest).trim();
    Some((name, sec, quoted.trim_end_matches('.')))
}

/// The name an item line declares, for `fn`, `struct`, `union`, `type`,
/// `const` and `static` items.
fn item_name(t: &str) -> Option<&str> {
    let t = t.strip_prefix("pub ").unwrap_or(t);
    let t = t.strip_prefix("const fn ").or_else(|| t.strip_prefix("unsafe fn ")).unwrap_or(t);
    let rest = ["fn ", "struct ", "union ", "type ", "const ", "static mut ", "static ", "enum "]
        .iter()
        .find_map(|kw| t.strip_prefix(kw))
        .or_else(|| t.strip_prefix("const fn "))?;
    let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").trim_end_matches('.').to_string()
}

fn sys_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let crates = root.join("crates");
    for entry in fs::read_dir(&crates).map_err(|e| format!("{}: {e}", crates.display()))? {
        let dir = entry.map_err(|e| e.to_string())?.path();
        let Some(name) = dir.file_name().and_then(|s| s.to_str()) else { continue };
        if !name.ends_with("-sys") {
            continue;
        }
        let src = dir.join("src");
        for f in fs::read_dir(&src).map_err(|e| format!("{}: {e}", src.display()))? {
            let p = f.map_err(|e| e.to_string())?.path();
            if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Removes the font escapes of `man(7)` source.
fn clean(s: &str) -> String {
    s.replace("\\fB", "")
        .replace("\\fR", "")
        .replace("\\fI", "")
        .replace("\\fP", "")
        .replace("\\-", "-")
        .replace("\\&", "")
}

/// Turns one line of an `mdoc(7)` or `man(7)` section body into text, or
/// returns `None` for a line that is pure formatting.
fn body_text(line: &str) -> Option<String> {
    if !line.starts_with('.') {
        let c = clean(line);
        return (!c.trim().is_empty()).then(|| c.trim().to_string());
    }
    let (mac, rest) = line[1..].split_once(' ').unwrap_or((&line[1..], ""));
    let rest = rest.trim();
    let (body, punct) = match rest.rsplit_once(' ') {
        Some((b, p)) if !p.is_empty() && p.chars().all(|c| ".,;:)".contains(c)) => (b, p),
        _ => (rest, ""),
    };
    let body = body.trim_matches('"');
    match mac {
        "Sy" | "Nm" | "Em" | "Ql" | "Dv" | "Ar" | "Li" | "B" | "I" | "BR" | "IR" | "RB" | "RI" => {
            Some(format!("{}{punct}", clean(body)))
        }
        "Pq" => Some(format!("({}){punct}", clean(body))),
        "Fn" => Some(format!("{}(){punct}", body.split_whitespace().next().unwrap_or(""))),
        "Xr" => {
            let mut w = body.split_whitespace();
            let n = w.next().unwrap_or("");
            let s = w.next().unwrap_or("");
            Some(format!("{n}({s}){punct}"))
        }
        _ => None,
    }
}

/// Reads the names, interface level, stability and first header from a
/// page written in either the `man(7)` or the `mdoc(7)` macro set.
fn parse_page(text: &str) -> Page {
    let mut names = Vec::new();
    let mut funcs: Vec<String> = Vec::new();
    let mut name_done = false;
    let mut level_parts: Vec<String> = Vec::new();
    let mut stability = String::new();
    let mut header = String::new();
    let mut section = "";
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(rest) = line.strip_prefix(".Sh ").or_else(|| line.strip_prefix(".SH ")) {
            section = match rest.trim().trim_matches('"') {
                "NAME" => "NAME",
                "SYNOPSIS" => "SYNOPSIS",
                "INTERFACE LEVEL" => "LEVEL",
                "INTERFACE STABILITY" => "STABILITY",
                "ATTRIBUTES" => "ATTRIBUTES",
                _ => "",
            };
            continue;
        }
        match section {
            "NAME" => {
                if let Some(rest) = line.strip_prefix(".Nm ") {
                    names.extend(rest.split([',', ' ']).filter(|w| !w.is_empty()).map(str::to_string));
                } else if !line.starts_with('.') && !name_done {
                    let c = clean(line);
                    let (before, dash) = match c.split_once(" - ") {
                        Some((b, _)) => (b.to_string(), true),
                        None => (c.clone(), false),
                    };
                    for n in before.split(',') {
                        let n = n.trim();
                        if !n.is_empty() && !n.contains(' ') {
                            names.push(n.to_string());
                        }
                    }
                    name_done = dash;
                }
            }
            "SYNOPSIS" => {
                if header.is_empty() {
                    if let Some(h) = line.strip_prefix(".In ") {
                        header = h.trim().to_string();
                    } else if let Some(h) = clean(line).trim().strip_prefix("#include <") {
                        header = h.trim_end_matches('>').to_string();
                    }
                }
                if let Some(f) = line.strip_prefix(".Fo ").or_else(|| line.strip_prefix(".Fn ")) {
                    funcs.push(f.split_whitespace().next().unwrap_or("").trim_matches('"').to_string());
                } else if !line.starts_with('.') && !line.contains("#include") {
                    // A man(7) prototype: the identifier directly before the
                    // first '(', which rules out `void (*fn)(...)` and prose.
                    let c = clean(line);
                    if let Some(open) = c.find('(') {
                        let head = &c[..open];
                        let ident: String =
                            head.chars().rev().take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_').collect();
                        let ident: String = ident.chars().rev().collect();
                        const NOT_NAMES: &[&str] = &["void", "int", "sizeof", "return", "if", "while", "for", "struct"];
                        // A few pages glue the return type to the name, as in
                        // `uint_t\\fBddi_model_convert_from`; split those.
                        let ident = ["uint_t", "void", "int"]
                            .iter()
                            .find_map(|t| ident.strip_prefix(t).filter(|r| r.len() > 3 && !r.starts_with('_')))
                            .map(str::to_string)
                            .unwrap_or(ident);
                        let starts_ok = ident.chars().next().is_some_and(|ch| !ch.is_ascii_digit());
                        let whole_word = head.len() == ident.len()
                            || head[..head.len() - ident.len()].ends_with([' ', '\t', '*', 't', 'd'])
                                && head.ends_with(ident.as_str());
                        if starts_ok && whole_word && !NOT_NAMES.contains(&ident.as_str()) {
                            funcs.push(ident);
                        }
                    }
                }
            }
            // Some pages forget the PARAMETERS heading, so a list or a
            // subsection ends the level text as well.
            "LEVEL" if [".Bl", ".It", ".Ss", ".SS", ".TP", ".IP"].iter().any(|m| line.starts_with(m)) => section = "",
            // A paragraph break inside the level ends a sentence, even when
            // the page forgot the full stop.
            "LEVEL" if [".Pp", ".PP", ".LP", ".P", ".sp"].contains(&line.trim()) => {
                if let Some(last) = level_parts.last_mut()
                    && !last.ends_with('.')
                {
                    last.push('.');
                }
            }
            "LEVEL" => {
                if let Some(t) = body_text(line) {
                    level_parts.push(t);
                }
            }
            "STABILITY" if stability.is_empty() => {
                if let Some(t) = body_text(line) {
                    // `.Sy Uncommitted -` followed by an explanation: keep
                    // only the stability word.
                    stability = t.trim_end_matches('.').trim_end_matches(" -").trim().to_string();
                }
            }
            "ATTRIBUTES" if stability.is_empty() => {
                let c = clean(line);
                if let Some(v) = c.trim().strip_prefix("Interface Stability") {
                    stability = v.trim_matches(|ch: char| ch.is_whitespace() || ch == '|').to_string();
                }
            }
            _ => {}
        }
    }
    funcs.retain(|n| !n.is_empty());
    funcs.sort();
    funcs.dedup();
    names.extend(funcs.iter().cloned());
    names.retain(|n| !n.is_empty());
    names.sort();
    names.dedup();
    let level = level_parts.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
    Page { names, funcs, level, stability, header }
}

fn report_coverage(pages: &BTreeMap<(String, String), Page>, bound: &BTreeSet<String>, markdown: bool) {
    let mut core_missing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut sub_missing: BTreeMap<&str, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut total = 0;
    let mut covered = 0;
    let mut obsolete_pages = 0;
    let mut macros = 0;
    let mut absent = 0;
    for ((sec, stem), p) in pages {
        if sec != "9F" || matches!(stem.as_str(), "Intro" | "Makefile" | "Manlink") {
            continue;
        }
        let counted = if p.funcs.is_empty() { &p.names } else { &p.funcs };
        if counted.iter().all(|n| p.obsolete(n)) {
            obsolete_pages += 1;
            continue;
        }
        if p.funcs.is_empty() {
            // A page without prototypes describes a concept, not functions.
            continue;
        }
        for n in counted {
            if n.chars().any(|c| c == '\\' || c == '(') || p.obsolete(n) {
                continue;
            }
            if MACROS.iter().any(|(pre, _)| n.starts_with(pre) && (pre.ends_with('_') || n == pre))
                && !bound.contains(n)
            {
                macros += 1;
                continue;
            }
            if NOT_IN_KERNEL.contains(&n.as_str()) {
                absent += 1;
                continue;
            }
            total += 1;
            if bound.contains(n) {
                covered += 1;
                continue;
            }
            match SUBSYSTEM_PREFIXES.iter().find(|(pre, _)| n.starts_with(pre)) {
                Some((_, crate_)) => {
                    sub_missing.entry(crate_).or_default().entry(stem.clone()).or_default().push(n.clone());
                }
                None => core_missing.entry(stem.clone()).or_default().push(n.clone()),
            }
        }
    }
    let core_gap: usize = core_missing.values().map(Vec::len).sum();
    let sub_gap: usize = sub_missing.values().flat_map(|m| m.values()).map(Vec::len).sum();
    if markdown {
        println!("| Measure | Count |");
        println!("| --- | --- |");
        println!("| 9F names on non-obsolete pages | {total} |");
        println!("| bound in the `-sys` crates | {covered} |");
        println!("| unbound, core families | {core_gap} |");
        println!("| unbound, planned subsystem crates | {sub_gap} |");
        println!("| obsolete or private pages skipped | {obsolete_pages} |");
        println!("| C macros without a symbol | {macros} |");
        println!("| documented but not exported by the kernel | {absent} |");
        println!();
        if !core_missing.is_empty() {
            println!("| Page | Unbound names | Header |");
            println!("| --- | --- | --- |");
            for (stem, names) in &core_missing {
                let h = pages.get(&("9F".to_string(), stem.clone())).map(|p| p.header.as_str()).unwrap_or("");
                println!("| `{stem}(9F)` | {} | `{h}` |", names.join(", "));
            }
            println!();
        }
        for (crate_, m) in &sub_missing {
            let n: usize = m.values().map(Vec::len).sum();
            println!("`{crate_}`: {n} names on {} pages.", m.len());
        }
    } else {
        println!(
            "coverage: {covered}/{total} 9F names bound; {core_gap} unbound in core, {sub_gap} in subsystem families; {obsolete_pages} obsolete or private pages, {macros} macros and {absent} missing symbols skipped"
        );
        for (stem, names) in &core_missing {
            println!("  {stem}(9F): {}", names.join(" "));
        }
        for (crate_, m) in &sub_missing {
            let n: usize = m.values().map(Vec::len).sum();
            println!("  [{crate_}] {n} names on {} pages", m.len());
        }
    }
}
