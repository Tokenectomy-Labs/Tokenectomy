mod cache;
mod cli;
mod config;
mod extractor;
mod git;
mod mcp;
mod provider;
mod proxy;
mod redact;
mod search;
mod formatter;

use clap::Parser;
use std::io::{self, Read};
use cli::{Cli, ProviderChoice};
use config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    
    if args.mcp {
        return mcp::run_server().await;
    }

    if args.proxy {
        return proxy::run_reverse_proxy(&args.proxy_bind, &args.upstream_url).await;
    }



    let app_config = AppConfig::load();

    // Merge Config & Args (Args override Config)
    let context_lines = args.context_lines.unwrap_or(app_config.context_lines.unwrap_or(10));
    let local_only = args.local_only || app_config.local_only.unwrap_or(false);
    let provider_choice = args.provider.unwrap_or(app_config.default_provider.clone().unwrap_or(ProviderChoice::Ollama));

use std::io::IsTerminal;

    let mut log = String::new();
    if let Some(file_path) = &args.file {
        log = std::fs::read_to_string(file_path)?;
    } else {
        if std::io::stdin().is_terminal() {
            use colored::*;
            use std::io::Write;

            // 1. Font Slant (Modern) & Yellow
            let banner = r#"
  ______     __                           __                  
 /_  __/___ / /_____  ____  ___  _____/ /_____  ____ ___  __  __
  / / / __ \/ //_/ _ \/ __ \/ _ \/ ___/ __/ __ \/ __ `__ \/ / / /
 / / / /_/ / ,< /  __/ / / /  __/ /__/ /_/ /_/ / / / / / / /_/ / 
/_/  \____/_/|_|\___/_/ /_/\___/\___/\__/\____/_/ /_/ /_/\__, /  
                                                        /____/   "#;
            println!("{}", banner.yellow().bold());
            println!("{:>70}", "v1.0.0 | AI-Powered Debugger".bright_black());
            
            // 2. Dashboard with Bug ASCII Art & Solid Background Colors
            let prov_val = format!("{:?}", app_config.default_provider.clone().unwrap_or(ProviderChoice::Ollama)).to_lowercase();
            let ctx_val = format!("{} lines", context_lines);
            
            let title = "   TOKENECTOMY AGENT v1.0.0 (AI DEBUGGER)   ";
            println!("╭{}╮", "─".repeat(70).yellow());
            println!("│{:^70}│", title.black().on_yellow().bold());
            println!("├{}┤", "─".repeat(70).yellow());
            
            println!("│  {}       {}│", "     ".green(), format!("{:<54}", "Available Providers").cyan().bold());
            println!("│  {}       {}│", "\\  / ".green(), format!("{:<54}", "openai, anthropic, ollama, mock").white());
            println!("│  {}                                                            │", "(oo) ".green());
            println!("│  {}       {}│", "/  \\/  \\".green(), format!("{:<54}", "Active Configuration").cyan().bold());
            println!("│  {}       Provider: {}│", "| |    | |".green(), format!("{:<44}", prov_val).white());
            println!("│  {}       Context : {}│", "\\/\\__/\\/".green(), format!("{:<44}", ctx_val).white());
            println!("│                                                                      │");
            println!("│             {}│", format!("{:<54}", "Features Activated").cyan().bold());
            println!("│             {}│", format!("{:<54}", "Smart Filter, StackOverflow, Secret Redaction").white());
            
            let footer = "   READY FOR LOG INPUT   ";
            println!("├{}┤", "─".repeat(70).yellow());
            println!("│{:^70}│", footer.black().on_green().bold());
            println!("╰{}╯\n", "─".repeat(70).yellow());

            // 3. Tip of the day
            let tips = [
                "Combine with `|` to pipe errors directly: `npm run dev 2>&1 | tkmy`",
                "Use `/help` in the REPL to see all available slash commands.",
                "Set `--yes` to auto-approve reading files outside the directory.",
                "Ensure your Ollama daemon is running if you use local mode."
            ];
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let tip = tips[(now as usize) % tips.len()];
            println!("{} {}\n", "✦ Tip:".bright_black().bold(), tip.bright_black());

            println!("Welcome to Tokenectomy! Paste your error log or type {} for options.", "/help".cyan());
            
            let mut line = String::new();
            loop {
                print!("tkmy> ");
                std::io::stdout().flush().unwrap();
                let n = io::stdin().read_line(&mut line)?;
                if n == 0 { break; } // Ctrl+D
                
                let trimmed = line.trim();
                if trimmed == "/" {
                    println!("\nAvailable Commands:");
                    println!("  /help    Show full CLI documentation");
                    println!("  /clear   Clear currently pasted log");
                    println!("  /submit  Submit the log to AI");
                    println!("  /exit    Exit the application\n");
                } else if trimmed == "/help" {
                    use clap::CommandFactory;
                    let mut cmd = Cli::command();
                    println!("\n");
                    cmd.print_help().unwrap();
                    println!("\n(You are still in REPL mode. Paste log or type /exit)\n");
                } else if trimmed == "/clear" {
                    log.clear();
                    println!("Log cleared.\n");
                } else if trimmed == "/submit" {
                    break;
                } else if trimmed == "/exit" {
                    println!("Exiting.");
                    return Ok(());
                } else {
                    const MAX_LOG_SIZE: usize = 10 * 1024 * 1024; // 10MB
                    if log.len() + line.len() > MAX_LOG_SIZE {
                        println!("⚠️  Input limit reached (10MB). Type /submit to send or /clear to reset.");
                    } else {
                        log.push_str(&line);
                    }
                }
                line.clear();
            }
        } else {
            io::stdin().take(10 * 1024 * 1024).read_to_string(&mut log)?;
        }
    }

    if log.trim().is_empty() {
        println!("Error: No log provided. Exiting.");
        return Ok(());
    }



    let (mut context, extracted_files) = extractor::extract_context(&log, context_lines, false);

    let yes_flag = args.yes || app_config.yes.unwrap_or(false);
    if !yes_flag && !local_only && provider_choice != ProviderChoice::Ollama {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let mut unsafe_files = Vec::new();
        for file in extracted_files {
            let path = std::path::Path::new(&file);
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                current_dir.join(path)
            };
            if !abs_path.starts_with(&current_dir) {
                unsafe_files.push(file);
            }
        }

        if !unsafe_files.is_empty() {
            println!("🔒 Security Alert: Error log points to files outside the current active directory:");
            for f in unsafe_files {
                println!("  - {}", f);
            }
            if std::io::stdin().is_terminal() {
                println!("\nAre you sure you want to send this file snippet to the Cloud AI? [y/N]");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    println!("Aborted for security reasons.");
                    return Ok(());
                }
            } else {
                println!("\nError: Aborted. Use the `--yes` flag to approve sending out-of-bounds files to the Cloud.");
                return Ok(());
            }
        }
    }

    let max_chars = args.max_context_chars.unwrap_or(app_config.max_context_chars.unwrap_or(10_000));
    if context.len() > max_chars {
        context.truncate(max_chars);
        context.push_str("\n... (context truncated due to limit)\n");
    }
    
    let mut combined = format!("Log:\n{}\nContext:\n{}", log, context);

    if let Some(git_diff) = git::get_recent_changes() {
        combined.push_str(&format!("\n\nRecent Git Changes:\n{}", git_diff));
    }

    if let Some(so_results) = search::search_stackoverflow(&log).await {
        combined.push_str(&format!("\n\nStack Overflow References (context for AI):\n{}", so_results));
    }

    let safe_payload = redact::redact_secrets(&combined);

    if let Some(cached) = cache::get_cached_response(&safe_payload) {
        if !args.no_color {
            use colored::*;
            println!("{}", "--- ⚡ Serving Answer from Local Cache ⚡ ---".yellow());
        } else {
            println!("--- ⚡ Serving Answer from Local Cache ⚡ ---");
        }
        formatter::print_explanation(&cached, args.no_color);
        return Ok(());
    }

    log::debug!("Payload size: {} bytes", safe_payload.len());

    let provider = provider::get_provider(provider_choice, local_only, &app_config)?;
    
    let pb = indicatif::ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""])
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Analyzing error with AI...");

    let explanation_result = provider.explain(&log, &context).await;
    
    pb.finish_and_clear();

    let explanation = explanation_result?;

    // Simpan ke cache untuk query masa depan
    cache::save_cached_response(&safe_payload, &explanation);

    formatter::print_explanation(&explanation, args.no_color);

    Ok(())
}
