// === debugger.rs ===
// Interactive bytecode debugger and inspection engine for Guppty.
//
// Supports:
//   - Line and function breakpoints with hit counting
//   - Step into (step), step over (next), step out (finish), and step instruction (stepi)
//   - Variable and environment inspection (print, locals, globals)
//   - Call stack backtraces with parameter display
//   - Source code listing with context and position indicators
//   - Bytecode chunk disassembly with current instruction pointer
//   - Scriptable / non-interactive command execution via --commands or piped stdin
//   - Post-mortem state inspection on runtime errors

use std::collections::VecDeque;
use std::io::{BufRead, Write};

use crate::error::{GupError, Span};
use crate::value::Value;
use crate::vm::Vm;

/// Signals what the VM should do after a debugger hook check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    Continue,
    Abort,
}

/// Interface between the VM execution loop and an attached debugger or tracer.
pub trait DebugHook {
    /// Called immediately before executing an instruction at `span` in `function_name`.
    fn before_instruction(
        &mut self,
        vm: &Vm,
        ip: usize,
        span: Span,
        function_name: Option<&str>,
    ) -> Result<DebugAction, GupError>;

    /// Called when an unhandled runtime error is raised.
    fn on_error(&mut self, vm: &Vm, error: &GupError);
}

/// Target location for setting a breakpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakpointTarget {
    Line(usize),
    Function(String),
}

/// One registered breakpoint.
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: usize,
    pub target: BreakpointTarget,
    pub hit_count: usize,
    pub enabled: bool,
}

/// Stepping state tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteppingState {
    None,
    StepInto {
        start_line: usize,
        start_depth: usize,
    },
    StepOver {
        start_line: usize,
        start_depth: usize,
    },
    StepOut {
        target_depth: usize,
    },
    StepInstruction,
}

/// The interactive or scriptable debugger session.
pub struct Debugger<R, W> {
    source_filename: String,
    source_lines: Vec<String>,
    breakpoints: Vec<Breakpoint>,
    next_bp_id: usize,
    stepping: SteppingState,
    last_line: Option<usize>,
    first_step: bool,
    command_queue: VecDeque<String>,
    last_command: Option<String>,
    input: R,
    output: W,
    is_interactive: bool,
    post_mortem: bool,
    batch_mode: bool,
}

impl<R: BufRead, W: Write> Debugger<R, W> {
    pub fn new(
        source_filename: String,
        source_text: String,
        input: R,
        output: W,
        is_interactive: bool,
    ) -> Self {
        let source_lines = source_text.lines().map(String::from).collect();
        Debugger {
            source_filename,
            source_lines,
            breakpoints: Vec::new(),
            next_bp_id: 1,
            stepping: SteppingState::None,
            last_line: None,
            first_step: true,
            command_queue: VecDeque::new(),
            last_command: None,
            input,
            output,
            is_interactive,
            post_mortem: false,
            batch_mode: false,
        }
    }

    /// Queue commands passed via CLI (e.g. semicolon-separated).
    pub fn queue_commands(&mut self, commands_str: &str) {
        self.batch_mode = true;
        for cmd in commands_str.split(';').map(str::trim) {
            if !cmd.is_empty() {
                self.command_queue.push_back(cmd.to_string());
            }
        }
    }

    /// Add a line breakpoint.
    pub fn set_line_breakpoint(&mut self, line: usize) -> usize {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        self.breakpoints.push(Breakpoint {
            id,
            target: BreakpointTarget::Line(line),
            hit_count: 0,
            enabled: true,
        });
        id
    }

    /// Add a function breakpoint.
    pub fn set_function_breakpoint(&mut self, func_name: String) -> usize {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        self.breakpoints.push(Breakpoint {
            id,
            target: BreakpointTarget::Function(func_name),
            hit_count: 0,
            enabled: true,
        });
        id
    }

    /// Delete a breakpoint by ID.
    pub fn delete_breakpoint(&mut self, id: usize) -> bool {
        if let Some(pos) = self.breakpoints.iter().position(|b| b.id == id) {
            self.breakpoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all breakpoints.
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Format a Value for debugger inspection.
    pub fn format_value(value: &Value) -> String {
        match value {
            Value::GuppyString(s) => format!("\"{s}\""),
            Value::GuppyChar(c) => format!("'{c}'"),
            Value::Nothing => "nil".to_string(),
            Value::GuppyClosure(c) => {
                let name = c
                    .function
                    .borrow()
                    .name
                    .clone()
                    .unwrap_or_else(|| "<anonymous>".to_string());
                format!("<closure {name}>")
            }
            Value::GuppyFunction(f) => {
                format!("<function({})>", f.params.join(", "))
            }
            _ => value.to_display_string(),
        }
    }

    /// Print source code lines around `target_line` with `->` marker.
    pub fn show_source_listing(&mut self, target_line: usize, radius: usize) {
        if self.source_lines.is_empty() {
            let _ = writeln!(self.output, "  (source listing unavailable)");
            return;
        }

        let start = target_line.saturating_sub(radius).max(1);
        let end = (target_line + radius).min(self.source_lines.len());

        for line_num in start..=end {
            let marker = if line_num == target_line {
                "->"
            } else {
                "  "
            };
            let content = &self.source_lines[line_num - 1];
            let _ = writeln!(self.output, "{marker} {:4} | {}", line_num, content);
        }
    }

    /// Read next command string either from queue or input stream.
    fn read_command(&mut self) -> Option<String> {
        if let Some(cmd) = self.command_queue.pop_front() {
            let _ = writeln!(self.output, "(guppty-debug) {}", cmd);
            return Some(cmd);
        }

        if self.batch_mode {
            return None;
        }

        if self.is_interactive {
            let _ = write!(self.output, "(guppty-debug) ");
            let _ = self.output.flush();
        }

        let mut line = String::new();
        match self.input.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    self.last_command.clone()
                } else {
                    self.last_command = Some(trimmed.clone());
                    Some(trimmed)
                }
            }
            Err(_) => None,
        }
    }

    /// Run the interactive command loop until execution should continue.
    fn command_loop(
        &mut self,
        vm: &Vm,
        ip: usize,
        span: Span,
        function_name: Option<&str>,
    ) -> Result<DebugAction, GupError> {
        loop {
            let Some(raw_cmd) = self.read_command() else {
                if self.batch_mode {
                    self.clear_breakpoints();
                    return Ok(DebugAction::Continue);
                }
                // On EOF, continue to end or abort if post-mortem
                return Ok(if self.post_mortem {
                    DebugAction::Abort
                } else {
                    DebugAction::Continue
                });
            };

            let parts: Vec<&str> = raw_cmd.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "help" | "?" => {
                    let _ = writeln!(
                        self.output,
                        "Guppty Debugger Commands:\n\
                           break, b <line|func>   Set breakpoint at line number or function name\n\
                           delete, d [id]         Delete breakpoint by ID (or all if omitted)\n\
                           breakpoints, info b    List all breakpoints\n\
                           step, s                Step into next source line\n\
                           next, n                Step over next source line\n\
                           finish, f              Step out of current function\n\
                           stepi, si              Step single bytecode instruction\n\
                           continue, c            Continue running until next breakpoint or end\n\
                           print, p <var>         Print value of a variable\n\
                           locals                 List local variables in current call frame\n\
                           globals                List all global variables\n\
                           stack                  Inspect VM operand stack\n\
                           backtrace, bt          Print call stack backtrace\n\
                           list, l [line] [rad]   Show source code listing\n\
                           disasm, dis            Disassemble bytecode of current function\n\
                           help, ?                Display this help message\n\
                           quit, q                Quit the debugger"
                    );
                }

                "break" | "b" => {
                    if parts.len() < 2 {
                        let _ = writeln!(self.output, "Usage: break <line_number | function_name>");
                    } else if let Ok(line) = parts[1].parse::<usize>() {
                        let id = self.set_line_breakpoint(line);
                        let _ = writeln!(
                            self.output,
                            "Breakpoint {} set at {}:{}",
                            id, self.source_filename, line
                        );
                    } else {
                        let fn_name = parts[1].to_string();
                        let id = self.set_function_breakpoint(fn_name.clone());
                        let _ = writeln!(
                            self.output,
                            "Breakpoint {} set for function '{}'",
                            id, fn_name
                        );
                    }
                }

                "delete" | "d" => {
                    if parts.len() >= 2 {
                        if let Ok(id) = parts[1].parse::<usize>() {
                            if self.delete_breakpoint(id) {
                                let _ = writeln!(self.output, "Breakpoint {} deleted.", id);
                            } else {
                                let _ = writeln!(self.output, "No breakpoint with ID {}.", id);
                            }
                        } else {
                            let _ = writeln!(self.output, "Usage: delete [id]");
                        }
                    } else {
                        self.clear_breakpoints();
                        let _ = writeln!(self.output, "All breakpoints deleted.");
                    }
                }

                "breakpoints" | "info" => {
                    if self.breakpoints.is_empty() {
                        let _ = writeln!(self.output, "No breakpoints set.");
                    } else {
                        let _ = writeln!(self.output, "Breakpoints:");
                        for b in &self.breakpoints {
                            let target_desc = match &b.target {
                                BreakpointTarget::Line(l) => format!("line {l}"),
                                BreakpointTarget::Function(f) => format!("function '{f}'"),
                            };
                            let status = if b.enabled { "enabled" } else { "disabled" };
                            let _ = writeln!(
                                self.output,
                                "  #{}: {} ({}, hit {} time{})",
                                b.id,
                                target_desc,
                                status,
                                b.hit_count,
                                if b.hit_count == 1 { "" } else { "s" }
                            );
                        }
                    }
                }

                "step" | "s" => {
                    self.stepping = SteppingState::StepInto {
                        start_line: span.line,
                        start_depth: vm.frames().len(),
                    };
                    return Ok(DebugAction::Continue);
                }

                "next" | "n" => {
                    self.stepping = SteppingState::StepOver {
                        start_line: span.line,
                        start_depth: vm.frames().len(),
                    };
                    return Ok(DebugAction::Continue);
                }

                "finish" | "f" => {
                    self.stepping = SteppingState::StepOut {
                        target_depth: vm.frames().len().saturating_sub(1),
                    };
                    return Ok(DebugAction::Continue);
                }

                "stepi" | "si" => {
                    self.stepping = SteppingState::StepInstruction;
                    return Ok(DebugAction::Continue);
                }

                "continue" | "c" => {
                    self.stepping = SteppingState::None;
                    return Ok(DebugAction::Continue);
                }

                "print" | "p" => {
                    if parts.len() < 2 {
                        let _ = writeln!(self.output, "Usage: print <variable_name>");
                    } else {
                        let var_name = parts[1];
                        match vm.get_variable(var_name) {
                            Ok(val) => {
                                let _ = writeln!(
                                    self.output,
                                    "{} = {}",
                                    var_name,
                                    Self::format_value(&val)
                                );
                            }
                            Err(_) => {
                                let _ = writeln!(
                                    self.output,
                                    "Undefined variable: '{}'",
                                    var_name
                                );
                            }
                        }
                    }
                }

                "locals" => {
                    let frame_idx = vm.frames().len().saturating_sub(1);
                    let locals = vm.get_frame_locals(frame_idx);
                    if locals.is_empty() {
                        let _ = writeln!(self.output, "  (no local variables in current frame)");
                    } else {
                        let _ = writeln!(self.output, "Local variables:");
                        for (name, val) in locals {
                            let _ = writeln!(
                                self.output,
                                "  {} = {}",
                                name,
                                Self::format_value(&val)
                            );
                        }
                    }
                }

                "globals" => {
                    let globals = vm.globals();
                    if globals.is_empty() {
                        let _ = writeln!(self.output, "  (no global variables)");
                    } else {
                        let mut sorted: Vec<_> = globals.iter().collect();
                        sorted.sort_by_key(|(k, _)| (*k).clone());
                        let _ = writeln!(self.output, "Global variables:");
                        for (name, val) in sorted {
                            let _ = writeln!(
                                self.output,
                                "  {} = {}",
                                name,
                                Self::format_value(val)
                            );
                        }
                    }
                }

                "stack" => {
                    let stack = vm.stack();
                    if stack.is_empty() {
                        let _ = writeln!(self.output, "  (operand stack is empty)");
                    } else {
                        let _ = writeln!(
                            self.output,
                            "Operand stack ({} value{}):",
                            stack.len(),
                            if stack.len() == 1 { "" } else { "s" }
                        );
                        for (i, val) in stack.iter().enumerate() {
                            let is_top = if i == stack.len() - 1 { " (top)" } else { "" };
                            let _ = writeln!(
                                self.output,
                                "  [{}] {}{}",
                                i,
                                Self::format_value(val),
                                is_top
                            );
                        }
                    }
                }

                "backtrace" | "bt" => {
                    let bt = vm.format_backtrace(&self.source_filename);
                    if bt.is_empty() {
                        let _ = writeln!(self.output, "  (empty call stack)");
                    } else {
                        let _ = writeln!(self.output, "Call stack:");
                        for line in bt {
                            let _ = writeln!(self.output, "  {}", line);
                        }
                    }
                }

                "list" | "l" => {
                    let target_line = if parts.len() >= 2 {
                        parts[1].parse::<usize>().unwrap_or(span.line)
                    } else {
                        span.line
                    };
                    let radius = if parts.len() >= 3 {
                        parts[2].parse::<usize>().unwrap_or(5)
                    } else {
                        5
                    };
                    self.show_source_listing(target_line, radius);
                }

                "disasm" | "dis" => {
                    if let Some(frame) = vm.frames().last() {
                        let fn_borrow = frame.function.borrow();
                        let name = function_name.unwrap_or("<main>");
                        let dis = fn_borrow.chunk.disassemble(
                            name,
                            &fn_borrow.local_names,
                            Some(ip),
                        );
                        let _ = write!(self.output, "{}", dis);
                    }
                }

                "quit" | "q" => {
                    let _ = writeln!(self.output, "Debugger session ended.");
                    return Ok(DebugAction::Abort);
                }

                unknown => {
                    let _ = writeln!(
                        self.output,
                        "Unknown command: '{}'. Type 'help' for available commands.",
                        unknown
                    );
                }
            }
        }
    }
}

impl<R: BufRead, W: Write> DebugHook for Debugger<R, W> {
    fn before_instruction(
        &mut self,
        vm: &Vm,
        ip: usize,
        span: Span,
        function_name: Option<&str>,
    ) -> Result<DebugAction, GupError> {
        let current_line = span.line;
        let line_changed = self.last_line != Some(current_line);
        let mut pause_reason = None;

        // 1. Initial entry point pause
        if self.first_step {
            self.first_step = false;
            pause_reason = Some("Stopped at program entry".to_string());
        }

        // 2. Line breakpoint check (triggers upon entering a line)
        if pause_reason.is_none() && line_changed {
            for bp in &mut self.breakpoints {
                if bp.enabled && bp.target == BreakpointTarget::Line(current_line) {
                    bp.hit_count += 1;
                    pause_reason = Some(format!(
                        "Hit breakpoint {} at {}:{} in {}",
                        bp.id,
                        self.source_filename,
                        current_line,
                        function_name.unwrap_or("<main>")
                    ));
                    break;
                }
            }
        }

        // 3. Function breakpoint check (triggers at instruction 0 of callee)
        if pause_reason.is_none() && ip == 0 {
            if let Some(name) = function_name {
                for bp in &mut self.breakpoints {
                    if bp.enabled && bp.target == BreakpointTarget::Function(name.to_string()) {
                        bp.hit_count += 1;
                        pause_reason = Some(format!(
                            "Hit breakpoint {} entering function '{}'",
                            bp.id, name
                        ));
                        break;
                    }
                }
            }
        }

        // 4. Stepping state evaluation
        if pause_reason.is_none() {
            match self.stepping {
                SteppingState::StepInstruction => {
                    pause_reason = Some("Single instruction step".to_string());
                }
                SteppingState::StepInto {
                    start_line,
                    start_depth,
                } => {
                    let depth = vm.frames().len();
                    if depth != start_depth
                        || (line_changed && current_line != start_line && current_line > 0)
                    {
                        pause_reason = Some("Step".to_string());
                    }
                }
                SteppingState::StepOver {
                    start_line,
                    start_depth,
                } => {
                    let depth = vm.frames().len();
                    if depth < start_depth
                        || (depth == start_depth
                            && line_changed
                            && current_line != start_line
                            && current_line > 0)
                    {
                        pause_reason = Some("Next".to_string());
                    }
                }
                SteppingState::StepOut { target_depth } => {
                    if vm.frames().len() <= target_depth {
                        pause_reason = Some("Finish".to_string());
                    }
                }
                SteppingState::None => {}
            }
        }

        if let Some(reason) = pause_reason {
            self.stepping = SteppingState::None;
            let _ = writeln!(
                self.output,
                "[{}] {}:{}: in {}",
                reason,
                self.source_filename,
                current_line,
                function_name.unwrap_or("<main>")
            );
            self.show_source_listing(current_line, 2);
            let action = self.command_loop(vm, ip, span, function_name)?;
            self.last_line = Some(current_line);
            Ok(action)
        } else {
            self.last_line = Some(current_line);
            Ok(DebugAction::Continue)
        }
    }

    fn on_error(&mut self, vm: &Vm, error: &GupError) {
        self.post_mortem = true;
        let _ = writeln!(
            self.output,
            "\n*** Unhandled Runtime Error ***\n{}\n\
             Entering post-mortem inspection. Type 'bt', 'locals', 'stack', or 'quit'.",
            error.render(&self.source_filename, &self.source_lines.join("\n"))
        );
        let frame_ip = vm.frames().last().map(|f| f.ip).unwrap_or(0);
        let span = vm.current_span();
        let fn_name = vm
            .frames()
            .last()
            .and_then(|f| f.function.borrow().name.clone());
        let _ = self.command_loop(vm, frame_ip, span, fn_name.as_deref());
    }
}
