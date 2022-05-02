use jellyfish_lang::{CompiledProgram, ErrorReporter, Source, VM};

fn main() {
    let cmd = std::env::args().next().unwrap();

    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("USAGE: {} <input_file>", cmd);
            return;
        }
    };

    let source = match std::fs::read_to_string(path.clone()) {
        Ok(source) => Source::new(path, source),
        Err(_) => {
            eprintln!("ERROR: couldn't open file");
            return;
        }
    };

    let mut diagnostics = ErrorReporter::default();

    let compile_result = CompiledProgram::compile(&source, &mut diagnostics);
    diagnostics.print(&source);

    let program = match compile_result {
        Ok(module) => module,
        Err(_) => {
            eprintln!("exiting with errors");
            return;
        }
    };

    println!("CHUNK\n=====");
    for opcode in &program.chunk.code {
        println!(
            "{}\t{}",
            opcode.byte(),
            opcode
                .instr_safe()
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "--".to_string())
        );
    }
    println!();

    let mut vm = VM::new();
    if let Err(err) = vm.run(program) {
        // TODO: print runtime errors nicely
        eprintln!("RUNTIME ERROR: {:#?}", err);
    }
}
