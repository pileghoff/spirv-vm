mod execution_context;
mod id_types;
mod instructions;
mod memory_store;
mod parse;
mod program;
mod run;
mod types;

use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, KeyCode};
use miette::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{Event, KeyEvent},
    },
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    execution_context::{ExecutionContex, ExecutionNext},
    instructions::Instruction,
};

/// Spirv emu
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to spirv
    #[arg(short, long)]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("----------- Parsing --------------");
    let prog = parse::parse(&args.path)?;
    println!("----------------------------------");

    let mut context = ExecutionContex::new(prog);
    ratatui::run(|terminal| {
        loop {
            {
                let context = context.clone();
                terminal
                    .draw(|frame| {
                        render_source(frame, &context);
                        render_next_instruction(frame, &context);
                    })
                    .unwrap();
            }
            if let Some(key) = event::read().unwrap().as_key_press_event() {
                match key.code {
                    KeyCode::Esc => return,
                    KeyCode::Enter => {
                        context.step();
                        while let Some(ExecutionNext::Instruction(Instruction::Line(_))) =
                            context.peek_next_instuction()
                        {
                            context.step();
                        }
                    }
                    _ => {}
                };
            }

            if context.stopped() {
                return;
            }
        }
    });

    Ok(())
}

fn render_next_instruction(frame: &mut Frame, context: &ExecutionContex) {
    let rect = Rect::new(frame.area().width / 2, 0, frame.area().width / 2, 3);

    let next_inst = match context.peek_next_instuction() {
        Some(ExecutionNext::Instruction(i)) => format!("{:?}", i),
        Some(ExecutionNext::Terminator(t)) => format!("{:?}", t),
        None => format!(""),
    };

    let paragraph = Paragraph::new(next_inst).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, rect);
}

fn render_source(frame: &mut Frame, context: &ExecutionContex) {
    let source_rect = Rect::new(0, 0, frame.area().width / 2, frame.area().height);
    let source: String = context
        .program
        .source
        .clone()
        .unwrap()
        .lines()
        .enumerate()
        .map(|e| {
            let l = e.1.replace("\t", "    ");
            if Some(e.0 as u32) == context.current_line {
                format!("> {}", l)
            } else {
                format!("  {}", l)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    let paragraph = Paragraph::new(source)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, source_rect);
}
