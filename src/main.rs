mod app;
mod config;
mod tray;

use app::{AppEvent, AppState, MenuCommand};
use config::Config;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray::Tray;
use tray_icon::menu::MenuEvent;

fn main() {
    let config = Config::load();
    let mut state = AppState::new(config);

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let _proxy = event_loop.create_proxy();

    let menu_channel = MenuEvent::receiver();
    let mut tray = Tray::new(&state);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(menu_event) = menu_channel.try_recv()
            && let Some(app_event) = tray.handle_menu_event(&menu_event, &state)
        {
            handle_event(app_event, &mut state, &mut tray, control_flow);
        }

        if let Event::UserEvent(app_event) = event {
            handle_event(app_event, &mut state, &mut tray, control_flow);
        }
    });
}

fn handle_event(
    event: AppEvent,
    state: &mut AppState,
    tray: &mut Tray,
    control_flow: &mut ControlFlow,
) {
    match event {
        AppEvent::Menu(cmd) => handle_menu_command(cmd, state, tray),
        AppEvent::Quit => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {} // phases 2-5 will handle remaining events
    }
}

fn handle_menu_command(cmd: MenuCommand, state: &mut AppState, tray: &mut Tray) {
    match cmd {
        MenuCommand::SetOutputMode(mode) => {
            state.config.output_mode = mode;
            state.config.save();
            tray.rebuild(state);
        }
        MenuCommand::SetHotkey(hotkey) => {
            state.config.hotkey = hotkey;
            state.config.save();
            tray.rebuild(state);
        }
        MenuCommand::SelectModel(model) => {
            state.config.selected_model = model;
            state.config.save();
            tray.rebuild(state);
        }
        MenuCommand::DownloadModel(_model) => {
            // phase 5: model download
        }
        MenuCommand::Quit => {}
    }
}
