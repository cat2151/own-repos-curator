use ratatui::{backend::TestBackend, Terminal};

pub(crate) fn render_overlay_text(
    width: u16,
    height: u16,
    render: impl FnOnce(&mut ratatui::Frame),
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(render).expect("draw");

    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
