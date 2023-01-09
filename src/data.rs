use std::{cell::RefCell, rc::Rc};

use druid::{
    im,
    text::{RichText, RichTextBuilder},
    widget::{Button, Flex, TextBox},
    Color, Data, FontFamily, FontWeight, Lens, Widget, WidgetExt,
};
use leda::gemini;

use crate::{
    controllers::InputWrapper,
    delegate::{self, PAGE_LOADED},
    widgets::Scroll,
};

#[derive(Clone, Data, Lens)]
pub struct CastorState {
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
            gemini_client: Rc::new(RefCell::new(
                gemini::Client::with_timeout(Some(std::time::Duration::new(5, 0)))
                    .expect("Failed to create gemini client"),
            )),
            history: im::Vector::from(vec![start_url.clone()]),
            history_index: 0,
        };

        ret.load_page(start_url.clone());
        ret
    }

    pub fn get_current_url(&self) -> &str {
        &self.history[self.history_index]
    }

    /// Pushes a url to the history vector, erasing everything after it.
    /// This will also increment the `history_index`
    pub fn push_to_history(&mut self, url: String) {
        if self.history_index == self.history.len() - 1 {
            self.history.push_back(url.clone());
            self.url = url;
            self.history_index += 1;
        } else {
            self.history[self.history_index + 1] = url.clone();
            self.url = url;
            self.history_index += 1;
            if self.history_index != self.history.len() - 1 {
                self.history.split_off(self.history_index);
            }
        }
    }

    pub fn load_current_page(&mut self) {
        self.load_page(self.history[self.history_index].clone())
    }

    pub fn load_page(&mut self, url: String) {
        let result = self
            .gemini_client
            .borrow_mut()
            .request(url.clone())
            .expect("Failed to send request");
        match result.header.status {
            gemini::header::StatusCode::Input(code) => {
                match code {
                    gemini::header::InputCode::Input => {},
                    gemini::header::InputCode::Sensitive => todo!(),
                }
            },
            gemini::header::StatusCode::Success => {
                if result.header.meta.starts_with("text/gemini") {
                    let body_str = std::str::from_utf8(result.body.as_ref().unwrap())
                        .expect("Failed to parse result as utf8")
                        .to_string();
                    let gemtext = gemini::Gemtext::new(&body_str)
                        .expect("Failed to parse body as gemtext");
    
                    self.body_content = Self::gemtext_to_rich_text(gemtext);
                    self.url = url;
                }
            },
            gemini::header::StatusCode::Redirect(_) => todo!(),
            gemini::header::StatusCode::FailTemporary(_) => todo!(),
            gemini::header::StatusCode::FailPermanent(_) => todo!(),
            gemini::header::StatusCode::CertFail(_) => todo!(),
        }
    }

    fn gemtext_to_rich_text(gemtext: gemini::Gemtext) -> RichText {
        let mut rich_text_builder = RichTextBuilder::new();
        for element in gemtext.elements {
            match element {
                gemini::gemtext::Element::Text(text) => {
                    rich_text_builder.push(&(text + "\n"));
                }
                gemini::gemtext::Element::Link(link, text) => {
                    rich_text_builder
                        .push(&(text + "\n"))
                        .link(delegate::LINK_CLICKED.with(link))
                        .underline(true)
                        .text_color(Color::AQUA);
                }
                gemini::gemtext::Element::Heading(text) => {
                    rich_text_builder.push(&(text + "\n")).size(28.0);
                }
                gemini::gemtext::Element::Subheading(text) => {
                    rich_text_builder.push(&(text + "\n")).size(24.0);
                }
                gemini::gemtext::Element::Subsubheading(text) => {
                    rich_text_builder.push(&(text + "\n")).size(20.0);
                }
                gemini::gemtext::Element::UnorderedList(items) => {
                    for item in items {
                        rich_text_builder
                            .push(&(String::from(&(String::from("•") + &item + "\n"))));
                    }
                }
                gemini::gemtext::Element::BlockQuote(text) => {
                    rich_text_builder
                        .push(&(text + "\n"))
                        .underline(true)
                        .weight(FontWeight::BOLD);
                }
                gemini::gemtext::Element::Preformatted(_alt_text, text) => {
                    rich_text_builder
                        .push(&(text + "\n"))
                        .font_family(FontFamily::MONOSPACE);
                }
            };
        }
        rich_text_builder.build()
    }
}

pub fn build_ui() -> impl Widget<CastorState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("←")
                        .on_click(|ctx, castor_state: &mut CastorState, _| {
                            castor_state.history_index -= 1;
                            castor_state.load_current_page();
                            ctx.submit_command(PAGE_LOADED);
                        })
                        .disabled_if(|castor_state, _| castor_state.history_index == 0),
                )
                .with_child(
                    Button::new("→")
                        .on_click(|ctx, castor_state: &mut CastorState, _| {
                            castor_state.history_index += 1;
                            castor_state.load_current_page();
                            ctx.submit_command(PAGE_LOADED);
                        })
                        .disabled_if(|castor_state, _| {
                            castor_state.history_index == castor_state.history.len() - 1
                        }),
                )
                .with_child(
                    Button::new("⟳").on_click(|ctx, castor_state: &mut CastorState, _| {
                        castor_state.load_current_page();
                        ctx.submit_command(PAGE_LOADED);
                    }),
                )
                .with_flex_child(
                    TextBox::new()
                        .expand_width()
                        .lens(CastorState::url)
                        .controller(InputWrapper::new(|ctx, castor_state: &mut CastorState| {
                            castor_state.load_page(castor_state.url.clone());
                            castor_state.push_to_history(castor_state.url.clone());
                            ctx.submit_command(PAGE_LOADED);
                        }))
                        .padding(5.0),
                    1.0,
                )
                .padding(5.0),
        )
        .with_flex_child(
            Scroll::new(druid::widget::RawLabel::new().lens(CastorState::body_content)).expand(),
            1.0,
        )
}
