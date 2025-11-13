use clap::Parser;
use walkdir::WalkDir;
use globset::{Glob, GlobMatcher};
use std::time::Instant;
use colored::Colorize;

//Простая утилита поиска файлов и папок (sf)
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Шаблон или подстрока для поиска
    pattern: String,

    //Искать папки вместо файлов
    #[arg(short = 'd', long)]
    dir: bool,

    //Игнорировать регистр
    #[arg(short = 'i', long)]
    ignore_case: bool,

    //Папка для начала поиска (по умолчанию)
    #[arg(short = 's', long, default_value = ".")]
    start: String,

    //Максимальная глубина рекурсии (-1 - без ограничений)
    #[arg(short = 'r', long, default_value = "-1")]
    max_depth: i32,

    // Использовать glob-шаблон ( * ? [])
    #[arg(short = 'g', long)]
    glob: bool,
}

fn matches(name: &str, args: &Args, matcher: &Option<GlobMatcher>) -> bool {
    if let Some(m) = matcher {
        m.is_match(name)
    } else {
        if args.ignore_case {
            name.to_lowercase().contains(&args.pattern.to_lowercase())
        } else {
            name.contains(&args.pattern)
        }
    }
}

fn main() {
    // Включаем поддержку ANSI цветов
    #[cfg(windows)]
    {
        if !colored::control::set_virtual_terminal(true).is_ok() {
            colored::control::set_override(true);
        }
    }

    let start_time = Instant::now();
    let args = Args::parse();
    let mut count = 0;

    let matcher: Option<GlobMatcher> = if args.glob {
        let g = Glob::new(&args.pattern).expect("Неверный шаблон glob");
        Some(g.compile_matcher())
    } else {
        None
    };

    let start_path = std::path::Path::new(&args.start).canonicalize().unwrap();

    for entry in WalkDir::new(start_path).follow_links(false).into_iter().filter_map(Result::ok) {
        let depth = entry.depth() as i32;
        if args.max_depth >= 0 && depth > args.max_depth {
            continue;
        }

        let file_type = entry.file_type();

        if args.dir && !file_type.is_dir() {
            continue;
        }
        if !args.dir && !file_type.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy();

        if matches(&name, &args, &matcher) {
            // Убираем \\?\ префикс для Windows
            let path_str = entry.path().display().to_string();
            let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
            println!("{}", clean_path.blue().bold());
            count += 1;
        }
    }

    let elapsed = start_time.elapsed();
    println!("{} {} {} {:.2?}", "\nFound:".green(), count, "in".green(), elapsed);
}