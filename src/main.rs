use clap::Parser;
use walkdir::WalkDir;
use globset::{Glob, GlobMatcher};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::path::Path;
use colored::Colorize;
use serde::Serialize;

/// Простая утилита поиска файлов и папок (sf) v0.3.0
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Шаблон или подстрока для поиска
    pattern: String,

    /// Искать папки вместо файлов
    #[arg(short = 'd', long)]
    dir: bool,

    /// Игнорировать регистр
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Папка для начала поиска (по умолчанию ".")
    #[arg(short = 's', long, default_value = ".")]
    start: String,

    /// Максимальная глубина рекурсии (-1 — без ограничений)
    #[arg(short = 'r', long, default_value = "-1")]
    max_depth: i32,

    /// Использовать glob-шаблон ( * ? [] )
    #[arg(short = 'g', long)]
    glob: bool,

    /// Фильтр по размеру: +100M, -1G, 500K (>, < или точно)
    #[arg(long, allow_hyphen_values = true)]
    size: Option<String>,

    /// Фильтр по дате изменения: -7 (последние 7 дней), +30 (старше 30 дней)
    #[arg(long, allow_hyphen_values = true)]
    mtime: Option<String>,

    /// Исключить пути, содержащие эту подстроку (можно указывать несколько раз)
    #[arg(long)]
    exclude: Vec<String>,

    /// Вывод в формате JSON
    #[arg(long)]
    json: bool,

    /// Показать статистику (кол-во файлов, общий размер)
    #[arg(long)]
    stats: bool,
}

// ── Парсинг фильтра размера ──────────────────────────────────────────────────

enum SizeFilter {
    GreaterThan(u64),
    LessThan(u64),
    Exactly(u64),
}

fn parse_size(s: &str) -> Option<SizeFilter> {
    let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
        ('+', r)
    } else if let Some(r) = s.strip_prefix('-') {
        ('-', r)
    } else {
        ('=', s)
    };

    let (num_str, multiplier) = if let Some(n) = rest.strip_suffix("GB").or(rest.strip_suffix("G")) {
        (n, 1_073_741_824u64)
    } else if let Some(n) = rest.strip_suffix("MB").or(rest.strip_suffix("M")) {
        (n, 1_048_576u64)
    } else if let Some(n) = rest.strip_suffix("KB").or(rest.strip_suffix("K")) {
        (n, 1_024u64)
    } else {
        (rest, 1u64)
    };

    let n: u64 = num_str.parse().ok()?;
    let bytes = n * multiplier;

    Some(match sign {
        '+' => SizeFilter::GreaterThan(bytes),
        '-' => SizeFilter::LessThan(bytes),
        _   => SizeFilter::Exactly(bytes),
    })
}

fn size_matches(file_size: u64, filter: &SizeFilter) -> bool {
    match filter {
        SizeFilter::GreaterThan(n) => file_size > *n,
        SizeFilter::LessThan(n)   => file_size < *n,
        SizeFilter::Exactly(n)    => file_size == *n,
    }
}

// ── Парсинг фильтра даты ─────────────────────────────────────────────────────

enum MtimeFilter {
    Within(i64),   // -N  → изменён не позже N дней назад
    OlderThan(i64), // +N  → изменён более N дней назад
}

fn parse_mtime(s: &str) -> Option<MtimeFilter> {
    if let Some(r) = s.strip_prefix('+') {
        r.parse().ok().map(MtimeFilter::OlderThan)
    } else if let Some(r) = s.strip_prefix('-') {
        r.parse().ok().map(MtimeFilter::Within)
    } else {
        s.parse().ok().map(MtimeFilter::Within)
    }
}

fn mtime_matches(modified: SystemTime, filter: &MtimeFilter) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mtime = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff_days = (now - mtime) / 86_400;

    match filter {
        MtimeFilter::Within(n)    => diff_days <= *n,
        MtimeFilter::OlderThan(n) => diff_days > *n,
    }
}

// ── Форматирование размера ───────────────────────────────────────────────────

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ── Совпадение по имени ──────────────────────────────────────────────────────

fn name_matches(name: &str, args: &Args, matcher: &Option<GlobMatcher>) -> bool {
    if let Some(m) = matcher {
        m.is_match(name)
    } else if args.ignore_case {
        name.to_lowercase().contains(&args.pattern.to_lowercase())
    } else {
        name.contains(&args.pattern)
    }
}

// ── JSON-структура ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct FileEntry {
    path: String,
    size: Option<u64>,
}

#[derive(Serialize)]
struct JsonOutput {
    results: Vec<FileEntry>,
    count: usize,
    total_size: u64,
    elapsed_ms: u128,
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    // Включаем ANSI-цвета на Windows
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    let start_time = Instant::now();
    let args = Args::parse();

    // Компилируем glob один раз
    let matcher: Option<GlobMatcher> = if args.glob {
        let g = Glob::new(&args.pattern).expect("Неверный glob-шаблон");
        Some(g.compile_matcher())
    } else {
        None
    };

    // Парсим фильтры
    let size_filter: Option<SizeFilter> = args.size.as_deref().and_then(parse_size);
    let mtime_filter: Option<MtimeFilter> = args.mtime.as_deref().and_then(parse_mtime);

    let start_path = Path::new(&args.start)
        .canonicalize()
        .expect("Не удалось найти стартовую директорию");

    let mut results: Vec<FileEntry> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in WalkDir::new(&start_path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        // Глубина
        let depth = entry.depth() as i32;
        if args.max_depth >= 0 && depth > args.max_depth {
            continue;
        }

        let file_type = entry.file_type();

        // Тип: файл или папка
        if args.dir && !file_type.is_dir() { continue; }
        if !args.dir && !file_type.is_file() { continue; }

        // Исключения
        let path_str = entry.path().display().to_string();
        if args.exclude.iter().any(|ex| path_str.contains(ex.as_str())) {
            continue;
        }

        // Имя
        let name = entry.file_name().to_string_lossy();
        if !name_matches(&name, &args, &matcher) {
            continue;
        }

        // Метаданные (размер + дата)
        let meta = entry.metadata().ok();
        let file_size = meta.as_ref().map(|m| m.len());

        // Фильтр по размеру
        if let Some(ref sf) = size_filter {
            match file_size {
                Some(sz) if size_matches(sz, sf) => {}
                _ => continue,
            }
        }

        // Фильтр по дате
        if let Some(ref mf) = mtime_filter {
            match meta.as_ref().and_then(|m| m.modified().ok()) {
                Some(mt) if mtime_matches(mt, mf) => {}
                _ => continue,
            }
        }

        // Убираем \\?\ на Windows
        let clean_path = path_str
            .strip_prefix(r"\\?\")
            .unwrap_or(&path_str)
            .to_string();

        total_size += file_size.unwrap_or(0);
        results.push(FileEntry { path: clean_path, size: file_size });
    }

    let elapsed = start_time.elapsed();

    // ── Вывод ────────────────────────────────────────────────────────────────

    if args.json {
        let output = JsonOutput {
            count: results.len(),
            total_size,
            elapsed_ms: elapsed.as_millis(),
            results,
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    // Обычный вывод
    for entry in &results {
        let line = if args.stats {
            let sz = entry.size.map(|s| format!(" ({})", format_size(s))).unwrap_or_default();
            format!("{}{}", entry.path, sz)
        } else {
            entry.path.clone()
        };
        println!("{}", line.blue().bold());
    }

    // Итоговая строка
    let count = results.len();
    if args.stats {
        println!(
            "\n{} {} {} {:.2?} | {} {}",
            "Found:".green().bold(),
            count.to_string().yellow().bold(),
            "files in".green(),
            elapsed,
            "Total size:".green().bold(),
            format_size(total_size).yellow().bold(),
        );
    } else {
        println!(
            "\n{} {} {} {:.2?}",
            "Found:".green().bold(),
            count.to_string().yellow().bold(),
            "in".green(),
            elapsed,
        );
    }
}