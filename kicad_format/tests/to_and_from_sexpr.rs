use std::path::Path;

use kicad_format::{
    convert::{FromSexpr, Parser, ToSexpr},
    footprint_library::FootprintLibraryFile,
    netlist::NetlistFile,
    pcb::PcbFile,
    schematic::SchematicFile,
    symbol_library::SymbolLibraryFile,
};
use kicad_sexpr::Sexpr;

fn assert_sexprs_eq(input_sexpr: Sexpr, output_sexpr: Sexpr) {
    if input_sexpr == output_sexpr {
        return;
    }

    let mut output = String::new();
    let context_lines = 3; // Number of lines to show before and after each diff

    // Collect all diffs with their line numbers
    let input_str = format!("{input_sexpr}");
    let output_str = format!("{output_sexpr}");
    let diffs: Vec<_> = diff::lines(&input_str, &output_str)
        .into_iter()
        .enumerate()
        .collect();

    // Find all lines that have changes
    let changed_lines: Vec<usize> = diffs
        .iter()
        .filter_map(|(idx, diff)| match diff {
            diff::Result::Left(_) | diff::Result::Right(_) => Some(*idx),
            diff::Result::Both(_, _) => None,
        })
        .collect();

    if changed_lines.is_empty() {
        panic!("No differences found but sexprs are not equal");
    }

    // Create ranges of lines to display (with context)
    let mut ranges_to_display = Vec::new();
    let mut current_start = changed_lines[0].saturating_sub(context_lines);
    let mut current_end = changed_lines[0] + context_lines;

    for &line in &changed_lines[1..] {
        let potential_start = line.saturating_sub(context_lines);
        let potential_end = line + context_lines;

        // If ranges overlap, merge them
        if potential_start <= current_end + 1 {
            current_end = potential_end;
        } else {
            // Save the current range and start a new one
            ranges_to_display.push((current_start, current_end));
            current_start = potential_start;
            current_end = potential_end;
        }
    }
    ranges_to_display.push((current_start, current_end));

    // Display the diffs with context
    for (range_idx, (start, end)) in ranges_to_display.iter().enumerate() {
        if range_idx > 0 {
            output.push_str("\n...\n\n");
        }

        for idx in *start..=(*end).min(diffs.len() - 1) {
            if let Some((_, diff)) = diffs.get(idx) {
                match diff {
                    diff::Result::Left(l) => output.push_str(&format!(
                        "{}",
                        ansi_term::Color::Red.paint(format!("-{}\n", l))
                    )),
                    diff::Result::Both(l, _) => output.push_str(&format!(" {}\n", l)),
                    diff::Result::Right(r) => output.push_str(&format!(
                        "{}",
                        ansi_term::Color::Green.paint(format!("+{}\n", r))
                    )),
                }
            }
        }
    }

    panic!("input sexpr (red) did not match output sexpr (green): \n{output}");
}

fn assert_in_out_eq<T: FromSexpr + ToSexpr>(input: &str, path: &Path) {
    let input_sexpr = kicad_sexpr::from_str(input).unwrap();

    let parser = Parser::new(input_sexpr.as_list().unwrap().clone());
    let pcb = T::from_sexpr(parser)
        .unwrap_or_else(|e| panic!("Failed to parse file: {}\n{e}\n{e:?}", path.display()));

    let output_sexpr = pcb.to_sexpr();

    assert_sexprs_eq(input_sexpr, output_sexpr);
}

fn test_files_in_dir<T: FromSexpr + ToSexpr, P: AsRef<Path>>(directory: P) {
    let files = std::fs::read_dir(directory)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    files.iter().for_each(|file| {
        if file.metadata().unwrap().is_dir() {
            return;
        }

        let input = std::fs::read_to_string(file.path()).unwrap();

        assert_in_out_eq::<T>(&input, &file.path());
    });
}

#[test]
#[ignore]
fn test_footprint_library() {
    test_files_in_dir::<FootprintLibraryFile, _>("./tests/footprint_library")
}

#[test]
fn test_symbol_library() {
    test_files_in_dir::<SymbolLibraryFile, _>("./tests/symbol_library")
}

#[test]
fn test_schematic() {
    test_files_in_dir::<SchematicFile, _>("./tests/schematic")
}

#[test]
#[ignore]
fn test_pcb() {
    test_files_in_dir::<PcbFile, _>("./tests/pcb")
}

#[test]
#[ignore]
fn test_netlist() {
    test_files_in_dir::<NetlistFile, _>("./tests/netlist")
}
