use clap::crate_version;
use infini::*;
use ratatui::backend::CrosstermBackend;
use ratatui::text::Line;
use ratatui::Terminal;
use std::sync::Arc;
use std::{env, io};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> AppResult<()> {
    setup_logging()?;
    infini::cli().version(crate_version!()).get_matches();
    let config = Arc::new(Config::load());
    let formatter = setup_formatter(&config)?;

    let mut app = App::new(config.clone(), &formatter);

    let llm = Arc::new(Mutex::new(
        LLMModel::init(&config.llm, config.clone()).await,
    ));

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    main_loop(&mut app, &llm, &mut tui, &formatter).await?;

    tui.exit()?;
    Ok(())
}

fn setup_logging() -> AppResult<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "infini-tui.log");
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_writer(file_appender)
        .with_level(true)
        .init();
    Ok(())
}

fn setup_formatter(config: &Arc<Config>) -> AppResult<Formatter> {
    let (formatter_config, formatter_assets) = Formatter::init();
    let formatter = Formatter::new(
        Box::leak(Box::new(formatter_config)),
        Box::leak(Box::new(formatter_assets)),
    );
    Ok(formatter)
}

async fn main_loop(
    app: &mut App<'_>,
    llm: &Arc<Mutex<Box<dyn LLM>>>,
    tui: &mut Tui<CrosstermBackend<std::io::Stderr>>,
    formatter: &Formatter<'_>,
) -> AppResult<()> {
    while app.running {
        tui.draw(app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                handle_key_events(key_event, app, llm.clone(), tui.events.sender.clone()).await?;
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::LLMEvent(llm_event) => {
                handle_llm_event(llm_event, app, llm.clone(), formatter).await?
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
        }
    }
    Ok(())
}

async fn handle_llm_event(
    llm_event: LLMAnswer,
    app: &mut App<'_>,
    llm: Arc<Mutex<Box<dyn LLM>>>,
    formatter: &Formatter<'_>,
) -> AppResult<()> {
    match llm_event {
        LLMAnswer::Answer(answer) => {
            app.chat.handle_answer(LLMAnswer::Answer(answer), formatter);
        }
        LLMAnswer::EndAnswer => {
            {
                let mut llm = llm.lock().await;
                llm.append_chat_msg(app.chat.answer.plain_answer.clone(), LLMRole::ASSISTANT);
            }
            app.chat.handle_answer(LLMAnswer::EndAnswer, formatter);
            app.terminate_response_signal
                .store(false, std::sync::atomic::Ordering::Relaxed);
            app.chat
                .formatted_chat
                .lines
                .push(Line::raw(format!("🤖: End of Answer.")));
        }
        LLMAnswer::StartAnswer => {
            app.spinner.active = false;
            app.chat.handle_answer(LLMAnswer::StartAnswer, formatter);
        }
    }
    Ok(())
}
