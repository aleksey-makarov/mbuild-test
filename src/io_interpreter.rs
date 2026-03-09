use nickel_lang_core::{
    eval::{cache::CacheImpl, Closure, value::NickelValue},
    identifier::LocIdent,
    mk_app,
    program::Program,
    term::{UnaryOp, make as mk_term},
};
use std::io::BufRead;

/// Interpret an IO action tree produced by a Nickel program.
///
/// The `value` is an evaluated Nickel value representing an IO action
/// (an enum variant like `'Return x`, `'Bind {action, cont}`, etc.).
pub fn interpret_io(
    program: &mut Program<CacheImpl>,
    value: NickelValue,
) -> Result<NickelValue, String> {
    // Evaluate to WHNF
    let value = program
        .eval_closure(Closure::from(value))
        .map_err(|e| format!("Eval error: {e:?}"))?;

    if let Some(ev) = value.as_enum_variant() {
        let tag = ev.tag.label();
        match tag {
            "Return" => {
                Ok(ev.arg.clone().unwrap_or_else(NickelValue::null))
            }

            "Bind" => {
                let record_arg = ev.arg.clone()
                    .ok_or("'Bind requires an argument")?;

                // Build field access as a term (don't evaluate yet — preserves environments)
                let action_term = mk_term::op1(
                    UnaryOp::RecordAccess(LocIdent::from("action")),
                    record_arg.clone(),
                );
                let result = interpret_io(program, action_term)?;

                // Build cont(result) as a single term so the evaluator resolves
                // the continuation's closure environment (which contains bindings
                // like `io`) atomically.
                let cont_term = mk_term::op1(
                    UnaryOp::RecordAccess(LocIdent::from("cont")),
                    record_arg,
                );
                let next = mk_app!(cont_term, result);
                interpret_io(program, next)
            }

            "PrintLine" => {
                let s = eval_to_string(program, ev.arg.clone()
                    .ok_or("'PrintLine requires an argument")?)?;
                println!("{s}");
                Ok(NickelValue::null())
            }

            "Print" => {
                let s = eval_to_string(program, ev.arg.clone()
                    .ok_or("'Print requires an argument")?)?;
                print!("{s}");
                Ok(NickelValue::null())
            }

            "ReadLine" => {
                let stdin = std::io::stdin();
                let line = stdin
                    .lock()
                    .lines()
                    .next()
                    .ok_or("EOF on stdin")?
                    .map_err(|e| format!("IO error: {e}"))?;
                Ok(NickelValue::string_posless(line))
            }

            "ReadFile" => {
                let path = eval_to_string(program, ev.arg.clone()
                    .ok_or("'ReadFile requires an argument")?)?;
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Cannot read file '{path}': {e}"))?;
                Ok(NickelValue::string_posless(content))
            }

            "WriteFile" => {
                let record_arg = ev.arg.clone()
                    .ok_or("'WriteFile requires an argument")?;

                let path_term = mk_term::op1(
                    UnaryOp::RecordAccess(LocIdent::from("path")),
                    record_arg.clone(),
                );
                let content_term = mk_term::op1(
                    UnaryOp::RecordAccess(LocIdent::from("content")),
                    record_arg,
                );

                let path = eval_to_string(program, path_term)?;
                let content = eval_to_string(program, content_term)?;

                std::fs::write(&path, &content)
                    .map_err(|e| format!("Cannot write file '{path}': {e}"))?;
                Ok(NickelValue::null())
            }

            "GetEnv" => {
                let name = eval_to_string(program, ev.arg.clone()
                    .ok_or("'GetEnv requires an argument")?)?;
                let val = std::env::var(&name).unwrap_or_default();
                Ok(NickelValue::string_posless(val))
            }

            "Args" => {
                let args: Vec<NickelValue> = std::env::args()
                    .skip(1) // skip binary name
                    .map(NickelValue::string_posless)
                    .collect();
                let array = args.into_iter().collect();
                Ok(NickelValue::array_posless(array, Vec::new()))
            }

            other => Err(format!("Unknown IO action: '{other}'")),
        }
    } else {
        Err(format!("Expected an enum variant (IO action), got: {value:?}"))
    }
}

/// Evaluate a NickelValue and extract it as a Rust String.
fn eval_to_string(
    program: &mut Program<CacheImpl>,
    value: NickelValue,
) -> Result<String, String> {
    let evaled = program
        .eval_closure(Closure::from(value))
        .map_err(|e| format!("Eval error (string): {e:?}"))?;

    evaled
        .as_string()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Expected a string, got: {evaled:?}"))
}
