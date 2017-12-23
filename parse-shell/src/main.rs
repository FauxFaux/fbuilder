extern crate conch_parser;
#[macro_use]
extern crate error_chain;
extern crate void;

use std::io;
use std::io::Read;

use conch_parser::ast;
use conch_parser::lexer;
use conch_parser::parse;

use errors::*;

fn run() -> Result<()> {
    let mut buf = String::with_capacity(10_000);
    io::stdin().read_to_string(&mut buf).unwrap();

    scan_top_levels(
        parse::DefaultParser::new(lexer::Lexer::new(buf.chars()))
            .into_iter()
            .map(|r| r.map_err(|e| Error::with_chain(e, "parsing")))
            .collect::<Result<Vec<ast::TopLevelCommand<String>>>>()?
            .iter(),
    )?;

    Ok(())
}

fn scan_top_levels<'i, I: Iterator<Item = &'i ast::TopLevelCommand<String>>>(it: I) -> Result<()> {
    for item in it {
        println!("state: {:?}", scan_top_level(item)?);
    }
    Ok(())
}

fn scan_top_level(item: &ast::TopLevelCommand<String>) -> Result<Exit> {
    let and_or = match item.0 {
        ast::Command::Job(_) => bail!("launches job; unsupported"),
        ast::Command::List(ref l) => l,
    };

    scan_and_or(&and_or).chain_err(|| format!("processing {:?}", and_or))
}

#[derive(Debug)]
enum Effect {
    ChangeSettings,
    ComparisonResult(Computed),
    RunsLdConfig,
}

#[derive(Debug, Default)]
struct Exit {
    code: u8,
    effects: Vec<Effect>,
}

fn scan_and_or(and_or: &ast::DefaultAndOrList) -> Result<Exit> {
    ensure!(
        and_or.rest.is_empty(),
        "top-level and-or expression; unsupported"
    );

    let mut exit = Exit::default();

    let pipeline = match and_or.first {
        ast::ListableCommand::Pipe(_, ref cmds) => cmds.clone(),
        ast::ListableCommand::Single(ref cmd) => vec![cmd.clone()],
    };

    for command in pipeline {
        match command {
            ast::PipeableCommand::Compound(comp) => {
                ensure!(comp.io.is_empty(), "compound command does io");
                match comp.kind {
                    ast::CompoundCommandKind::If {
                        ref conditionals,
                        ref else_branch,
                    } => {
                        scan_if(conditionals.as_slice(), else_branch)?;
                    }
                    _ => bail!("unsupported command kind"),
                }
            }
            ast::PipeableCommand::FunctionDef(_, _) => bail!("unsupported function def"),
            ast::PipeableCommand::Simple(cmd) => {
                let cmd: Box<ast::DefaultSimpleCommand> = cmd;

                ensure!(
                    cmd.redirects_or_env_vars.is_empty(),
                    "redirects not supported"
                );

                let cmd = cmd.redirects_or_cmd_words
                    .into_iter()
                    .map(scan_word)
                    .collect::<Result<Vec<ast::DefaultSimpleWord>>>()?;

                match cmd[0] {
                    ast::SimpleWord::Literal(ref string) => match string.as_ref() {
                        "set" => {
                            ensure!(2 == cmd.len(), "set must have one argument");
                            ensure!(
                                ast::SimpleWord::Literal("-e".to_string()) == cmd[1],
                                "set -e only"
                            );
                            exit.effects.push(Effect::ChangeSettings);
                        }
                        "ldconfig" => {
                            ensure!(1 == cmd.len(), "ldconfig doesn't have args");
                            exit.effects.push(Effect::RunsLdConfig);
                        }
                        _ => bail!("unsupported command: {:?}", cmd),
                    },
                    ast::SimpleWord::SquareOpen => {
                        ensure!(
                            ast::SimpleWord::SquareClose == cmd[cmd.len() - 1],
                            "square open without close"
                        );
                        let parts: Vec<ast::DefaultSimpleWord> = cmd[1..cmd.len() - 1].to_vec();
                        ensure!(3 == parts.len(), "triplet comparison please");
                        match parts[1] {
                            ast::SimpleWord::Literal(ref token) if token == "=" => {
                                exit.effects
                                    .push(Effect::ComparisonResult(if word_to_string(&parts[0])?
                                        == word_to_string(&parts[2])?
                                    {
                                        Computed::True
                                    } else {
                                        Computed::False
                                    }));
                            }
                            ref other => bail!("unsupported comparator: {:?}", other),
                        }
                    }
                    _ => bail!("unsupported command type: {:?}", cmd),
                }
            }
        }
    }

    Ok(exit)
}

fn scan_if(
    conditionals: &[ast::GuardBodyPair<ast::TopLevelCommand<String>>],
    else_branch: &Option<Vec<ast::TopLevelCommand<String>>>,
) -> Result<()> {
    for cond in conditionals {
        match scan_guard(&cond.guard).chain_err(|| format!("processing guard: {:?}", cond.guard))? {
            Computed::True => {
                scan_top_levels(cond.body.iter())?;
                return Ok(());
            }
            Computed::False => continue,
            _ => bail!("unexpected eval result"),
        }
    }

    ensure!(else_branch.is_none(), "else unsupported");

    Ok(())
}

#[derive(Debug)]
enum Computed {
    True,
    False,
    Filesystem,
    Command,
}

fn scan_guard(guard: &[ast::TopLevelCommand<String>]) -> Result<Computed> {
    ensure!(1 == guard.len(), "more than one guard?");
    let item: &ast::TopLevelCommand<String> = &guard[0];

    let and_or: &ast::DefaultAndOrList = match item.0 {
        ast::Command::Job(_) => bail!("launches job; unsupported"),
        ast::Command::List(ref l) => l,
    };

    let scanned = scan_and_or(and_or)?;

    ensure!(1 == scanned.effects.len(), "only one effect");
    match scanned.effects.into_iter().next().unwrap() {
        Effect::ComparisonResult(computed) => Ok(computed),
        _ => bail!("unexpected effect"),
    }
}

fn scan_word(
    word: ast::RedirectOrCmdWord<ast::DefaultRedirect, ast::TopLevelWord<String>>,
) -> Result<ast::DefaultSimpleWord> {
    let word: ast::TopLevelWord<String> = match word {
        ast::RedirectOrCmdWord::Redirect(_) => bail!("redirect in command"),
        ast::RedirectOrCmdWord::CmdWord(word) => word,
    };

    let word: ast::DefaultWord = match word.0 {
        ast::ComplexWord::Concat(_) => bail!("concat?"),
        ast::ComplexWord::Single(word) => word,
    };

    let word = match word {
        ast::Word::Simple(word) => word,
        ast::Word::DoubleQuoted(word) => {
            ensure!(
                1 == word.len(),
                "double-quoted words can't be multiple words"
            );
            word[0].clone()
        }
        ast::Word::SingleQuoted(_) => bail!("single-quoted word"),
    };

    Ok(word)
}

fn word_to_string(word: &ast::DefaultSimpleWord) -> Result<String> {
    Ok(match word {
        &ast::SimpleWord::Literal(ref string) => string.to_string(),
        &ast::SimpleWord::Param(ast::Parameter::Positional(1)) => "configure".to_string(),
        _ => bail!("unsupported word type"),
    })
}

quick_main!(run);

mod errors {
    error_chain!{
        foreign_links {
            Conch(::conch_parser::parse::ParseError<::void::Void>);
        }
    }
}
