use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};

use crate::{
    action::Action,
    config,
    events::{Event, Events},
    serde_helper::keybindings::key_event_to_string,
    tui::Tui,
    widgets::{
        help::{Help, HelpWidget},
        popup_message::{PopupMessageState, PopupMessageWidget},
        search_filter_prompt::SearchFilterPromptWidget,
        search_page::SearchPage,
        search_page::SearchPageWidget,
        status_bar::StatusBarWidget,
        summary::{Summary, SummaryWidget},
        tabs::SelectedTab,
    },
};

#[derive(
    Default, Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIs,
)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Common,
    #[default]
    Summary,
    PickerShowCrateInfo,
    PickerHideCrateInfo,
    Search,
    Filter,
    Popup,
    Help,
    Quit,
}

impl Mode {
    pub fn is_prompt(&self) -> bool {
        self.is_search() || self.is_filter()
    }

    pub fn is_picker(&self) -> bool {
        self.is_picker_hide_crate_info() || self.is_picker_show_crate_info()
    }
}

struct AppWidget;

#[derive(Debug)]
pub struct App {
    /// Receiver end of an asynchronous channel for actions that the app needs
    /// to process.
    rx: UnboundedReceiver<Action>,

    /// Sender end of an asynchronous channel for dispatching actions from
    /// various parts of the app to be handled by the event loop.
    tx: UnboundedSender<Action>,

    /// A thread-safe indicator of whether data is currently being loaded,
    /// allowing different parts of the app to know if it's in a loading state.
    loading_status: Arc<AtomicBool>,

    /// The active mode of the application, which could change how user inputs
    /// and commands are interpreted.
    mode: Mode,

    /// The active mode of the application, which could change how user inputs
    /// and commands are interpreted.
    last_mode: Mode,

    /// A list of key events that have been held since the last tick, useful for
    /// interpreting sequences of key presses.
    last_tick_key_events: Vec<KeyEvent>,

    /// frame counter
    frame_count: usize,

    summary: Summary,
    search: SearchPage,
    popup: Option<(PopupMessageWidget, PopupMessageState)>,
    help: Help,
    selected_tab: SelectedTab,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let loading_status = Arc::new(AtomicBool::default());
        let search = SearchPage::new(tx.clone(), loading_status.clone());
        let summary = Summary::new(tx.clone(), loading_status.clone());
        Self {
            rx,
            tx,
            mode: Mode::default(),
            last_mode: Mode::default(),
            loading_status,
            search,
            summary,
            popup: Default::default(),
            last_tick_key_events: Default::default(),
            frame_count: Default::default(),
            help: Default::default(),
            selected_tab: Default::default(),
        }
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(
        &mut self,
        mut tui: Tui,
        mut events: Events,
        query: Option<String>,
    ) -> Result<()> {
        // uncomment to test error handling
        // panic!("test panic");
        // Err(color_eyre::eyre::eyre!("Error"))?;
        self.tx.send(Action::Init { query })?;

        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?.map(|action| self.tx.send(action));
            }
            while let Ok(action) = self.rx.try_recv() {
                self.handle_action(action.clone())?;
                if matches!(action, Action::Resize(_, _) | Action::Render) {
                    self.draw(&mut tui)?;
                }
            }
            if self.should_quit() {
                break;
            }
        }
        Ok(())
    }

    /// Handles an event by producing an optional `Action` that the application
    /// should perform in response.
    ///
    /// This method maps incoming events from the terminal user interface to
    /// specific `Action` that represents tasks or operations the
    /// application needs to carry out.
    fn handle_event(&mut self, e: Event) -> Result<Option<Action>> {
        let maybe_action = match e {
            Event::Quit => Some(Action::Quit),
            Event::Tick => Some(Action::Tick),
            Event::KeyRefresh => Some(Action::KeyRefresh),
            Event::Render => Some(Action::Render),
            Event::Crossterm(CrosstermEvent::Resize(x, y)) => Some(Action::Resize(x, y)),
            Event::Crossterm(CrosstermEvent::Key(key)) => self.handle_key_event(key)?,
            _ => None,
        };
        Ok(maybe_action)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        debug!("Received key {:?}", key);
        match self.mode {
            Mode::Search => {
                self.search.handle_key(key);
            }
            Mode::Filter => {
                self.search.handle_key(key);
                self.search.handle_filter_prompt_change();
            }
            _ => (),
        };
        Ok(self.handle_key_events_from_config(key))
    }

    /// Evaluates a sequence of key events against user-configured key bindings
    /// to determine if an `Action` should be triggered.
    ///
    /// This method supports user-configurable key sequences by collecting key
    /// events over time and then translating them into actions according to the
    /// current mode.
    fn handle_key_events_from_config(&mut self, key: KeyEvent) -> Option<Action> {
        self.last_tick_key_events.push(key);
        let config = config::get();
        config
            .key_bindings
            .event_to_command(self.mode, &self.last_tick_key_events)
            .or_else(|| {
                config
                    .key_bindings
                    .event_to_command(Mode::Common, &self.last_tick_key_events)
            })
            .map(|command| config.key_bindings.command_to_action(command))
    }

    /// Performs the `Action` by calling on a respective app method.
    ///
    /// Upon receiving an action, this function updates the application state, performs necessary
    /// operations like drawing or resizing the view, or changing the mode. Actions that affect the
    /// navigation within the application, are also handled. Certain actions generate a follow-up
    /// action which will be to be processed in the next iteration of the main event loop.
    fn handle_action(&mut self, action: Action) -> Result<()> {
        if action != Action::Tick && action != Action::Render && action != Action::KeyRefresh {
            info!("{action:?}");
        }
        match action {
            Action::Quit => self.quit(),
            Action::KeyRefresh => self.key_refresh_tick(),
            Action::Init { ref query } => self.init(query)?,
            Action::Tick => self.tick(),
            Action::StoreTotalNumberOfCrates(n) => self.store_total_number_of_crates(n),
            Action::ScrollUp => self.scroll_up(),
            Action::ScrollDown => self.scroll_down(),

            Action::ScrollTop
            | Action::ScrollBottom
            | Action::ScrollSearchResultsUp
            | Action::ScrollSearchResultsDown => self.search.handle_action(action.clone()),

            Action::ScrollCrateInfoUp => self.search.crate_info.scroll_previous(),
            Action::ScrollCrateInfoDown => self.search.crate_info.scroll_next(),
            Action::ReloadData => self.search.reload_data(),
            Action::IncrementPage => self.search.increment_page(),
            Action::DecrementPage => self.search.decrement_page(),
            Action::NextSummaryMode => self.summary.next_mode(),
            Action::PreviousSummaryMode => self.summary.previous_mode(),
            Action::NextTab => self.goto_next_tab(),
            Action::PreviousTab => self.goto_previous_tab(),
            Action::SwitchMode(mode) => self.switch_mode(mode),
            Action::SwitchToLastMode => self.switch_to_last_mode(),
            Action::SubmitSearch => self.search.submit_query(),
            Action::ToggleShowCrateInfo => self.search.toggle_show_crate_info(),
            Action::UpdateCurrentSelectionCrateInfo => self.update_current_selection_crate_info(),
            Action::UpdateSearchTableResults => self.search.update_search_table_results(),
            Action::UpdateSummary => self.summary.update(),
            Action::ShowFullCrateInfo => self.show_full_crate_details(),
            Action::ShowErrorPopup(ref err) => self.show_error_popup(err.clone()),
            Action::ShowInfoPopup(ref info) => self.show_info_popup(info.clone()),
            Action::ClosePopup => self.close_popup(),
            Action::ToggleSortBy { reload, forward } => {
                self.search.toggle_sort_by(reload, forward)?
            }
            Action::ClearTaskDetailsHandle(ref id) => self
                .search
                .clear_task_details_handle(uuid::Uuid::parse_str(id)?)?,
            Action::OpenDocsUrlInBrowser => self.open_docs_url_in_browser()?,
            Action::OpenCratesIOUrlInBrowser if self.mode.is_summary() => {
                self.open_summary_url_in_browser()?
            }
            Action::OpenCratesIOUrlInBrowser => self.open_crates_io_url_in_browser()?,
            Action::CopyCargoAddCommandToClipboard => self.copy_cargo_add_command_to_clipboard()?,
            _ => {}
        }
        match action {
            Action::ScrollUp | Action::ScrollDown | Action::ScrollTop | Action::ScrollBottom
                if self.mode.is_prompt() || self.mode.is_picker() =>
            {
                let _ = self.tx.send(Action::UpdateCurrentSelectionCrateInfo);
            }
            Action::SubmitSearch => {
                let _ = self.tx.send(Action::ReloadData);
            }
            _ => {}
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), self);
            self.update_frame_count(frame);
            self.update_cursor(frame);
        })?;
        Ok(())
    }
}

impl App {
    fn tick(&mut self) {
        self.search.update_search_table_results();
    }

    fn init(&mut self, query: &Option<String>) -> Result<()> {
        if let Some(query) = query {
            self.search.search = query.clone();
            let _ = self.tx.send(Action::SwitchMode(Mode::Search));
            let _ = self.tx.send(Action::SubmitSearch);
        } else {
            self.summary.request()?;
        }
        Ok(())
    }

    fn key_refresh_tick(&mut self) {
        self.last_tick_key_events.drain(..);
    }

    fn should_quit(&self) -> bool {
        self.mode == Mode::Quit
    }

    fn quit(&mut self) {
        self.mode = Mode::Quit
    }

    fn scroll_up(&mut self) {
        match self.mode {
            Mode::Popup => {
                if let Some((_, popup_state)) = &mut self.popup {
                    popup_state.scroll_up();
                }
            }
            Mode::Summary => self.summary.scroll_previous(),
            Mode::Help => self.help.scroll_up(),
            _ => self.search.scroll_up(),
        }
    }

    fn scroll_down(&mut self) {
        match self.mode {
            Mode::Popup => {
                if let Some((_, popup_state)) = &mut self.popup {
                    popup_state.scroll_down();
                }
            }
            Mode::Summary => self.summary.scroll_next(),
            Mode::Help => self.help.scroll_down(),
            _ => self.search.scroll_down(),
        }
    }

    fn switch_mode(&mut self, mode: Mode) {
        self.last_mode = self.mode;
        self.mode = mode;
        self.search.mode = mode;
        match self.mode {
            Mode::Search => {
                self.selected_tab.select(SelectedTab::Search);
                self.search.enter_search_insert_mode();
            }
            Mode::Filter => {
                self.selected_tab.select(SelectedTab::Search);
                self.search.enter_filter_insert_mode();
            }
            Mode::Summary => {
                self.search.enter_normal_mode();
                self.selected_tab.select(SelectedTab::Summary);
            }
            Mode::Help => {
                self.search.enter_normal_mode();
                self.help.mode = Some(self.last_mode);
                self.selected_tab.select(SelectedTab::None)
            }
            Mode::PickerShowCrateInfo | Mode::PickerHideCrateInfo => {
                self.search.enter_normal_mode();
                self.selected_tab.select(SelectedTab::Search)
            }
            _ => {
                self.search.enter_normal_mode();
                self.selected_tab.select(SelectedTab::None)
            }
        }
    }

    fn switch_to_last_mode(&mut self) {
        self.switch_mode(self.last_mode);
    }

    fn goto_next_tab(&mut self) {
        match self.mode {
            Mode::Summary => self.switch_mode(Mode::Search),
            Mode::Search => self.switch_mode(Mode::Summary),
            _ => self.switch_mode(Mode::Summary),
        }
    }

    fn goto_previous_tab(&mut self) {
        match self.mode {
            Mode::Summary => self.switch_mode(Mode::Search),
            Mode::Search => self.switch_mode(Mode::Summary),
            _ => self.switch_mode(Mode::Summary),
        }
    }

    fn show_error_popup(&mut self, message: String) {
        error!("Error: {message}");
        self.popup = Some((
            PopupMessageWidget::new("Error".into(), message),
            PopupMessageState::default(),
        ));
        self.switch_mode(Mode::Popup);
    }

    fn show_info_popup(&mut self, info: String) {
        info!("Info: {info}");
        self.popup = Some((
            PopupMessageWidget::new("Info".into(), info),
            PopupMessageState::default(),
        ));
        self.switch_mode(Mode::Popup);
    }

    fn close_popup(&mut self) {
        self.popup = None;
        if self.last_mode.is_popup() {
            self.switch_mode(Mode::Search);
        } else {
            self.switch_mode(self.last_mode);
        }
    }

    fn update_current_selection_crate_info(&mut self) {
        self.search.clear_all_previous_task_details_handles();
        self.search.request_crate_details();
    }

    fn show_full_crate_details(&mut self) {
        self.search.clear_all_previous_task_details_handles();
        self.search.request_full_crate_details();
    }

    fn store_total_number_of_crates(&mut self, n: u64) {
        self.search.total_num_crates = Some(n)
    }

    fn open_docs_url_in_browser(&self) -> Result<()> {
        if let Some(crate_response) = self.search.crate_response.lock().unwrap().clone() {
            let name = crate_response.crate_data.name;
            webbrowser::open(&format!("https://docs.rs/{name}/latest"))?;
        }
        Ok(())
    }

    fn open_summary_url_in_browser(&self) -> Result<()> {
        if let Some(url) = self.summary.url() {
            webbrowser::open(&url)?;
        } else {
            let _ = self.tx.send(Action::ShowErrorPopup(
                "Unable to open URL in browser: No summary data loaded".into(),
            ));
        }
        Ok(())
    }

    fn open_crates_io_url_in_browser(&self) -> Result<()> {
        if let Some(crate_response) = self.search.crate_response.lock().unwrap().clone() {
            let name = crate_response.crate_data.name;
            webbrowser::open(&format!("https://crates.io/crates/{name}"))?;
        }
        Ok(())
    }

    fn copy_cargo_add_command_to_clipboard(&self) -> Result<()> {
        use copypasta::ClipboardProvider;
        match copypasta::ClipboardContext::new() {
            Ok(mut ctx) => {
                if let Some(crate_response) = self.search.crate_response.lock().unwrap().clone() {
                    let msg = format!("cargo add {}", crate_response.crate_data.name);
                    let _ = match ctx.set_contents(msg.clone()).ok() {
                        Some(_) => self.tx.send(Action::ShowInfoPopup(format!(
                            "Copied to clipboard: `{msg}`"
                        ))),
                        None => self.tx.send(Action::ShowErrorPopup(format!(
                            "Unable to copied to clipboard: `{msg}`"
                        ))),
                    };
                } else {
                    let _ = self
                        .tx
                        .send(Action::ShowErrorPopup("No selection made to copy".into()));
                }
            }
            Err(err) => {
                let _ = self.tx.send(Action::ShowErrorPopup(format!(
                    "Unable to create ClipboardContext: {}",
                    err
                )));
            }
        }
        Ok(())
    }

    // Sets the frame count
    fn update_frame_count(&mut self, frame: &mut Frame<'_>) {
        self.frame_count = frame.count();
    }

    // Sets cursor for the prompt
    fn update_cursor(&mut self, frame: &mut Frame<'_>) {
        if self.mode.is_prompt() {
            if let Some(cursor_position) = self.search.cursor_position() {
                frame.set_cursor_position(cursor_position);
            }
        }
    }

    fn events_widget(&self) -> Option<Block> {
        if self.last_tick_key_events.is_empty() {
            return None;
        }

        let title = format!(
            "{:?}",
            self.last_tick_key_events
                .iter()
                .map(key_event_to_string)
                .collect::<Vec<_>>()
        );
        Some(
            Block::default()
                .title(title)
                .title_position(ratatui::widgets::block::Position::Top)
                .title_alignment(ratatui::layout::Alignment::Right),
        )
    }

    fn loading(&self) -> bool {
        self.loading_status.load(Ordering::SeqCst)
    }
}

impl StatefulWidget for AppWidget {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Background color
        Block::default()
            .bg(config::get().color.base00)
            .render(area, buf);

        use Constraint::*;
        let [header, main] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        let [tabs, events] = Layout::horizontal([Min(15), Fill(1)]).areas(header);

        state.render_tabs(tabs, buf);
        state.events_widget().render(events, buf);

        let mode = if matches!(state.mode, Mode::Popup | Mode::Quit) {
            state.last_mode
        } else {
            state.mode
        };
        match mode {
            Mode::Summary => state.render_summary(main, buf),
            Mode::Help => state.render_help(main, buf),

            Mode::Search => state.render_search(main, buf),
            Mode::Filter => state.render_search(main, buf),
            Mode::PickerShowCrateInfo => state.render_search(main, buf),
            Mode::PickerHideCrateInfo => state.render_search(main, buf),

            Mode::Common => {}
            Mode::Popup => {}
            Mode::Quit => {}
        };

        if state.loading() {
            Line::from(state.spinner())
                .right_aligned()
                .render(main, buf);
        }

        if let Some((popup, popup_state)) = &mut state.popup {
            popup.render(area, buf, popup_state);
        }
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        use strum::IntoEnumIterator;
        let titles = SelectedTab::iter().map(|tab| tab.title());
        let highlight_style = SelectedTab::highlight_style();

        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    fn render_summary(&mut self, area: Rect, buf: &mut Buffer) {
        let [main, status_bar] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(area);
        SummaryWidget.render(main, buf, &mut self.summary);
        self.render_status_bar(status_bar, buf);
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
        let [main, status_bar] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(area);
        HelpWidget.render(main, buf, &mut self.help);
        self.render_status_bar(status_bar, buf);
    }

    fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
        let prompt_height = if self.mode.is_prompt() && self.search.is_prompt() {
            5
        } else {
            0
        };
        let [main, prompt, status_bar] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(prompt_height),
            Constraint::Length(1),
        ])
        .areas(area);

        SearchPageWidget.render(main, buf, &mut self.search);

        self.render_prompt(prompt, buf);
        self.render_status_bar(status_bar, buf);
    }

    fn render_prompt(&mut self, area: Rect, buf: &mut Buffer) {
        let p = SearchFilterPromptWidget::new(
            self.mode,
            self.search.sort.clone(),
            &self.search.input,
            self.search.search_mode,
        );
        p.render(area, buf, &mut self.search.prompt);
    }

    fn render_status_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let s = StatusBarWidget::new(
            self.mode,
            self.search.sort.clone(),
            self.search.input.value().to_string(),
        );
        s.render(area, buf);
    }

    fn spinner(&self) -> String {
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let index = self.frame_count % spinner.len();
        let symbol = spinner[index];
        symbol.into()
    }
}
