use crate::tui::app::{
    App, SelectionStep, TimePeriod, spawn_fetch_app_stats, spawn_fetch_network_stats,
};
use chrono::{Days, Months};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum StatsTarget {
    Applications,
    Network,
}

pub fn get_period_string(period: TimePeriod, app: &App) -> String {
    match period {
        TimePeriod::Today => "today".to_string(),
        TimePeriod::Yesterday => "yesterday".to_string(),
        TimePeriod::Week => "week".to_string(),
        TimePeriod::Month => "month".to_string(),
        TimePeriod::Year => "year".to_string(),
        TimePeriod::All => "all".to_string(),
        TimePeriod::Custom => {
            if let (Some(start), Some(end)) = (app.date_picker.start_date, app.date_picker.end_date)
            {
                format!("custom:{}:{}", start, end)
            } else {
                "all".to_string()
            }
        }
    }
}

pub fn get_display_period(period: TimePeriod) -> &'static str {
    match period {
        TimePeriod::Today => "Today",
        TimePeriod::Yesterday => "Yesterday",
        TimePeriod::Week => "Week",
        TimePeriod::Month => "Month",
        TimePeriod::Year => "Year",
        TimePeriod::All => "All Time",
        TimePeriod::Custom => "Custom",
    }
}

pub fn cycle_period_next(p: TimePeriod) -> TimePeriod {
    match p {
        TimePeriod::Today => TimePeriod::Yesterday,
        TimePeriod::Yesterday => TimePeriod::Week,
        TimePeriod::Week => TimePeriod::Month,
        TimePeriod::Month => TimePeriod::Year,
        TimePeriod::Year => TimePeriod::All,
        TimePeriod::All => TimePeriod::Custom,
        TimePeriod::Custom => TimePeriod::Today,
    }
}

pub fn cycle_period_prev(p: TimePeriod) -> TimePeriod {
    match p {
        TimePeriod::Today => TimePeriod::Custom,
        TimePeriod::Yesterday => TimePeriod::Today,
        TimePeriod::Week => TimePeriod::Yesterday,
        TimePeriod::Month => TimePeriod::Week,
        TimePeriod::Year => TimePeriod::Month,
        TimePeriod::All => TimePeriod::Year,
        TimePeriod::Custom => TimePeriod::All,
    }
}

pub fn fetch_stats(app: &App, target: StatsTarget) {
    let period = match target {
        StatsTarget::Applications => app.apps.period,
        StatsTarget::Network => app.network.period,
    };
    let period_str = get_period_string(period, app);
    match target {
        StatsTarget::Applications => spawn_fetch_app_stats(app.tx.clone(), &period_str),
        StatsTarget::Network => spawn_fetch_network_stats(app.tx.clone(), &period_str),
    }
}

pub fn handle_date_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.date_picker.open = false;
        }
        KeyCode::Left => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_days(Days::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Right => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_days(Days::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Up => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_days(Days::new(7))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Down => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_days(Days::new(7))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageUp => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_months(Months::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageDown => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_months(Months::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Enter => match app.date_picker.selection_step {
            SelectionStep::Start => {
                app.date_picker.start_date = Some(app.date_picker.current_selection);
                app.date_picker.selection_step = SelectionStep::End;

                app.date_picker.current_selection = app
                    .date_picker
                    .current_selection
                    .checked_add_days(Days::new(1))
                    .unwrap_or(app.date_picker.current_selection);
            }
            SelectionStep::End => {
                let end = app.date_picker.current_selection;
                if let Some(start) = app.date_picker.start_date {
                    if end >= start {
                        app.date_picker.end_date = Some(end);
                        app.date_picker.open = false;
                    } else {
                        app.date_picker.start_date = Some(end);
                        app.date_picker.end_date = Some(start);
                        app.date_picker.open = false;
                    }
                } else {
                    app.date_picker.start_date = Some(end);
                    app.date_picker.selection_step = SelectionStep::End;
                }
            }
        },
        _ => {}
    }
}

pub fn handle_period_nav(app: &mut App, key: KeyEvent, target: StatsTarget) -> bool {
    if app.date_picker.open {
        handle_date_picker_key(app, key);
        if !app.date_picker.open {
            fetch_stats(app, target);
        }
        return true;
    }

    match key.code {
        KeyCode::Char('h') => {
            let current = match target {
                StatsTarget::Applications => app.apps.period,
                StatsTarget::Network => app.network.period,
            };
            let new_period = cycle_period_prev(current);
            match target {
                StatsTarget::Applications => app.apps.period = new_period,
                StatsTarget::Network => app.network.period = new_period,
            };

            if new_period != TimePeriod::Custom {
                fetch_stats(app, target);
            }
            true
        }
        KeyCode::Char('l') => {
            let current = match target {
                StatsTarget::Applications => app.apps.period,
                StatsTarget::Network => app.network.period,
            };
            let new_period = cycle_period_next(current);
            match target {
                StatsTarget::Applications => app.apps.period = new_period,
                StatsTarget::Network => app.network.period = new_period,
            };

            if new_period != TimePeriod::Custom {
                fetch_stats(app, target);
            }
            true
        }
        KeyCode::Char('/') => {
            match target {
                StatsTarget::Applications => app.apps.period = TimePeriod::Custom,
                StatsTarget::Network => app.network.period = TimePeriod::Custom,
            };
            app.date_picker.open = true;
            app.date_picker.selection_step = SelectionStep::Start;
            if app.date_picker.start_date.is_none() {
                app.date_picker.current_selection = chrono::Local::now().date_naive();
            }
            true
        }
        _ => false,
    }
}
