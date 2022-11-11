use std::cell::RefCell;
use std::rc::Rc;
use druid::text::{RichTextBuilder, RichText};
use druid::{AppLauncher, WindowDesc, Widget, PlatformError, Data, Lens, WidgetExt, Color, FontWeight, FontFamily, im};
use druid::widget::{Flex, Button, TextBox, Controller, Scroll};
use leda::gemini;

const DEFAULT_URL: &str = "gemini://gemini.circumlunar.space/";

#[derive(Clone, Data, Lens)]
struct CastorState {
    url: String,
    body_content: RichText,
    gemini_client: Rc<RefCell<gemini::Client>>,
    history: im::Vector<String>,
    history_index: usize,
}

impl CastorState {
    pub fn new(start_url: String) -> Self {
        let mut ret = Self {
            url: String::from(start_url.clone()),
            body_content: druid::text::RichTextBuilder::new().build(),
            gemini_client: Rc::new(RefCell::new(gemini::Client::with_timeout(Some(std::time::Duration::new(5, 0)))
                .expect("Failed to create gemini client"))),
            history: im::Vector::from(vec![start_url.clone()]),
            history_index: 0
        };

        ret.load_page(start_url.clone());
        ret
    }

    // adds to the history vector at the current history_index+1
    pub fn push_to_history(&mut self, url: String) {
        if self.history_index == self.history.len()-1 {
            self.history.push_back(url.clone());
            self.url = url;
            self.history_index += 1;
        }
        else {
            self.history[self.history_index+1] = url.clone();
            self.url = url;
            self.history_index += 1;
            if self.history_index != self.history.len()-1 {
                self.history.split_off(self.history_index);
            }
        }
    }

    pub fn load_current_page(&mut self) {
        self.load_page(self.history[self.history_index].clone())
    }

    pub fn load_page(&mut self, url: String) {
        let result = self.gemini_client.as_ref().borrow_mut().request(url.clone())
            .expect("Failed to send request");
        if let  gemini::header::StatusCode::Success = result.header.status {
            if result.header.meta.starts_with("text/gemini") {
                let body_str = std::str::from_utf8(result.body.as_ref().unwrap())
                    .expect("Failed to parse result as utf8").to_string();
                let gemtext = gemini::Gemtext::new(&body_str)
                    .expect("Failed to parse body as gemtext");

                let mut rich_text_builder = RichTextBuilder::new();
                for element in gemtext.elements {
                    match element {
                        gemini::gemtext::Element::Text(text) => {
                            rich_text_builder.push(&(text+"\n"));
                        },
                        gemini::gemtext::Element::Link(_link, text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .underline(true)
                                .text_color(Color::AQUA);
                        },
                        gemini::gemtext::Element::Heading(text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .size(28.0);
                        },
                        gemini::gemtext::Element::Subheading(text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .size(24.0);
                        },
                        gemini::gemtext::Element::Subsubheading(text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .size(20.0);
                        },
                        gemini::gemtext::Element::UnorderedList(items) => {
                            for item in items {
                                rich_text_builder.push(&(String::from(&(String::from("•")+&item+"\n"))));
                            }
                        },
                        gemini::gemtext::Element::BlockQuote(text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .underline(true)
                                .weight(FontWeight::BOLD);
                        },
                        gemini::gemtext::Element::Preformatted(_alt_text, text) => {
                            rich_text_builder.push(&(text+"\n"))
                                .font_family(FontFamily::MONOSPACE);
                        },
                    };
                }

                self.body_content = rich_text_builder.build();
                self.url = url;
            }
        }
    }
}

struct InputWrapper<F: Fn(&druid::Event, &mut CastorState)> {
    on_submit: F
}

impl<W: Widget<CastorState>, F: Fn(&druid::Event, &mut CastorState)> Controller<CastorState, W> for InputWrapper<F> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut CastorState,
        env: &druid::Env,
    ) {
        if let druid::Event::KeyUp(druid::KeyEvent{key: druid::KbKey::Enter, ..}) = event {
            (self.on_submit)(event, data);
        }
        // Always pass on the event!
        child.event(ctx, event, data, env)
    }
}

fn build_ui() -> impl Widget<CastorState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("←")
                        .on_click(|_, castor_state: &mut CastorState, _| {
                            castor_state.history_index -= 1;
                            castor_state.load_current_page();
                        })
                        .disabled_if(|castor_state, _| {
                            castor_state.history_index == 0
                        })
                )
                .with_child(
                    Button::new("→")
                        .on_click(|_, castor_state: &mut CastorState, _| {
                            castor_state.history_index += 1;
                            castor_state.load_current_page()
                        })
                        .disabled_if(|castor_state, _| {
                            castor_state.history_index == castor_state.history.len()-1
                        })
                )
                .with_child(
                    Button::new("⟳")
                )
                .with_flex_child(
                    TextBox::new()
                        .expand_width()
                        .lens(CastorState::url)
                        .controller(InputWrapper{ on_submit: |_, castor_state| {
                            castor_state.load_page(castor_state.url.clone());
                            castor_state.push_to_history(castor_state.url.clone());
                            println!("{:?}", castor_state.history);
                        }})
                        .padding(5.0),
                        1.0
                )
                .padding(5.0)
        )
        .with_flex_child(
            Scroll::new(druid::widget::RawLabel::new().lens(CastorState::body_content))
                .expand(),
                1.0
        )
}

fn main() -> Result<(), PlatformError> {
    let initial_state = CastorState::new(DEFAULT_URL.to_string());
    let window_desc = WindowDesc::new(build_ui())
        .title("Castor")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(window_desc).launch(initial_state)?;
    Ok(())
}