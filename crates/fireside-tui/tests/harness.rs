use std::path::{Path, PathBuf};

use fireside_engine::{PresentationSession, load_graph};
use fireside_tui::{Action, App, Theme};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

pub struct AppHarness {
    pub app: App,
    width: u16,
    height: u16,
}

impl AppHarness {
    pub fn for_graph(graph_path: &Path, start_index: usize) -> Self {
        let graph = load_graph(graph_path).expect("graph fixture should load");
        let session = PresentationSession::new(graph, start_index);
        let mut app = App::new(session, Theme::default());
        let width = 100;
        let height = 28;
        app.update(Action::Resize(width, height));
        Self { app, width, height }
    }

    pub fn for_hello(start_index: usize) -> Self {
        Self::for_graph(&hello_path(), start_index)
    }

    pub fn press(&mut self, action: Action) {
        self.app.update(action);
    }

    pub fn current_node_id(&self) -> String {
        self.app
            .session
            .current_node()
            .id
            .as_deref()
            .unwrap_or("(no-id)")
            .to_string()
    }

    pub fn render_text(&mut self) -> String {
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| self.app.view(frame))
            .expect("frame should render");

        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            while out.ends_with(' ') {
                out.pop();
            }
            out.push('\n');
        }
        out
    }
}

pub fn hello_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/hello.json")
}
