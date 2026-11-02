use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use typemark::{FontBook, ParseOptions};

const USAGE: &str = "\
Usage: typemark <command> [options]

Commands:
  build <input.md> [options]   Compile Markdown to PDF

Options for build:
  -o, --output <file>   Output path (default: input with .pdf extension)
  --css <file>          Stylesheet (parsed in a later version; ignored for now)
  --font <file>         Font file or collection to use (repeatable)
  --svg <dir>           Also write one SVG per page into <dir>
  --commonmark-breaks   Join single newlines with a space, as in CommonMark
                        (by default every newline breaks the line)
";

/// Fonts tried when no `--font` is given, in order.
const DEFAULT_FONTS: &[&str] = &[
    "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "C:\\Windows\\Fonts\\malgun.ttf",
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("build") => build(&args[1..]),
        Some("-V" | "--version") => {
            println!("typemark {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("typemark: {message}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Default)]
struct BuildArgs {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    css: Option<PathBuf>,
    fonts: Vec<PathBuf>,
    svg_dir: Option<PathBuf>,
    commonmark_breaks: bool,
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut parsed = BuildArgs::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let mut value = |name: &str| {
            iter.next().map(PathBuf::from).ok_or_else(|| format!("{name} needs a value"))
        };
        match arg.as_str() {
            "-o" | "--output" => parsed.output = Some(value(arg)?),
            "--css" => parsed.css = Some(value(arg)?),
            "--font" => parsed.fonts.push(value(arg)?),
            "--svg" => parsed.svg_dir = Some(value(arg)?),
            "--commonmark-breaks" => parsed.commonmark_breaks = true,
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag}\n\n{USAGE}")),
            _ if parsed.input.is_none() => parsed.input = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument {arg}")),
        }
    }
    Ok(parsed)
}

fn build(args: &[String]) -> Result<(), String> {
    let args = parse_build_args(args)?;
    let input = args.input.ok_or_else(|| format!("missing input file\n\n{USAGE}"))?;
    let output = args.output.unwrap_or_else(|| input.with_extension("pdf"));

    let markdown = read_to_string(&input)?;
    let css = match &args.css {
        Some(path) => {
            eprintln!("typemark: warning: CSS is not supported yet; using the default theme");
            read_to_string(path)?
        }
        None => String::new(),
    };
    let fonts = load_fonts(&args.fonts)?;

    let start = Instant::now();
    let options = ParseOptions { newline_breaks: !args.commonmark_breaks };
    let paged = typemark::compile_with(&markdown, &css, &fonts, options).map_err(|e| e.to_string())?;
    let bytes = typemark::pdf(&paged, &fonts).map_err(|e| e.to_string())?;
    let elapsed = start.elapsed();

    std::fs::write(&output, bytes).map_err(|e| format!("cannot write {}: {e}", output.display()))?;

    if let Some(dir) = &args.svg_dir {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        for (i, svg) in typemark::svg(&paged, &fonts).iter().enumerate() {
            let path = dir.join(format!("page-{}.svg", i + 1));
            std::fs::write(&path, svg).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
    }

    eprintln!(
        "wrote {} ({} page{}, {:.1} ms)",
        output.display(),
        paged.pages.len(),
        if paged.pages.len() == 1 { "" } else { "s" },
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}

fn load_fonts(paths: &[PathBuf]) -> Result<FontBook, String> {
    let mut book = FontBook::new();
    if paths.is_empty() {
        for path in DEFAULT_FONTS {
            if let Ok(data) = std::fs::read(path) {
                book.add(data);
                break;
            }
        }
        if book.is_empty() {
            return Err("no default font found; pass one with --font <file>".into());
        }
    }
    for path in paths {
        let data = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        if book.add(data) == 0 {
            return Err(format!("{} is not a supported font file", path.display()));
        }
    }
    Ok(book)
}

fn read_to_string(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}
