const DEFAULT_URL: &str = "gemini://gemini.circumlunar.space/";

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use glib::{clone, MainContext, Sender, PRIORITY_DEFAULT};
use gtk::{
    prelude::*, Button, ButtonsType, Entry, MessageDialog, Orientation, ScrolledWindow, TextBuffer,
    TextChildAnchor, TextTag, TextTagTable, TextView,
};
use gtk::{Application, ApplicationWindow};
use gtk4 as gtk;
use leda::gemini::{self, gemtext, Gemtext};
use percent_encoding::utf8_percent_encode;

// program state
#[derive(Clone)]
struct Castor {
    current_url: String,
    history: Vec<String>,
    history_index: usize,
}

impl Castor {
    pub fn new() -> Castor {
        Castor {
            current_url: String::from(DEFAULT_URL),
            history: vec![String::from(DEFAULT_URL)],
            history_index: 0,
        }
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.github.maebee-cm.dioscuri.castor")
        .build();

    app.connect_activate(|app| {
        let window = match build_ui(&app) {
            Ok(window) => window,
            Err(e) => {
                eprintln!("Error occurred while creating ui: {}", e);
                return;
            }
        };
        window.show();
    });

    app.run();
}

fn build_ui(app: &Application) -> Result<ApplicationWindow> {
    let client = Rc::new(RefCell::new(
        gemini::Client::new().context("Failed to create gemini client")?,
    ));
    let castor_state = Rc::new(RefCell::new(Castor::new()));

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .title("Castor")
        .build();

    let window_content = gtk::Box::new(Orientation::Vertical, 0);
    let control_bar = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .margin_start(5)
        .margin_end(5)
        .margin_top(5)
        .margin_bottom(5)
        .vexpand(false)
        .build();

    let back_button = Button::builder().label("←").sensitive(false).build();
    let forward_button = Button::builder().label("→").sensitive(false).build();
    let refresh_button = Button::with_label("⟳");
    let url_bar = Entry::builder().text(DEFAULT_URL).hexpand(true).build();

    control_bar.append(&back_button);
    control_bar.append(&forward_button);
    control_bar.append(&refresh_button);
    control_bar.append(&url_bar);

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let page_content = TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .margin_start(5)
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .build();

    scroll.set_child(Some(&page_content));

    window_content.append(&control_bar);
    window_content.append(&scroll);

    window.set_child(Some(&window_content));

    let tag_table = TextTagTable::new();
    tag_table.add(&TextTag::builder().name("plaintext").build());
    tag_table.add(
        &TextTag::builder()
            .name("link")
            .foreground("blue")
            .underline(gtk::pango::Underline::Single)
            .build(),
    );
    tag_table.add(&TextTag::builder().name("header").size_points(20.0).build());
    tag_table.add(
        &TextTag::builder()
            .name("subheader")
            .size_points(16.0)
            .build(),
    );
    tag_table.add(
        &TextTag::builder()
            .name("subsubheader")
            .size_points(12.0)
            .build(),
    );
    tag_table.add(&TextTag::builder().name("preformatted").build());
    let buffer = TextBuffer::new(Some(&tag_table));
    page_content.set_buffer(Some(&buffer));

    // we'll use this when the user clicks on a links
    let (tx, rx) = MainContext::channel::<String>(PRIORITY_DEFAULT);

    window.connect_show(clone!(@strong client, @strong castor_state, @weak page_content, @weak window, @strong tx => move |_w| {
        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@weak client, @strong castor_state, @weak page_content, @strong tx => async move {
            load_page(&mut client.borrow_mut(), castor_state.borrow_mut().current_url.clone(), String::from(DEFAULT_URL), &page_content, &window, tx.clone()).await;
        }));
    }));

    url_bar.connect_activate(clone!(@strong castor_state, @strong client, @weak page_content, @weak window,
        @weak forward_button, @weak back_button, @strong tx => move |entry| {
        let url = entry.buffer().text().to_string();
        forward_button.set_sensitive(false);
        back_button.set_sensitive(true);

        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@strong castor_state, @strong client, @weak page_content,
            @weak window, @strong tx, @weak entry => async move {
            let ret = load_page(&mut client.borrow_mut(), castor_state.borrow_mut().current_url.clone(), url, &page_content, &window, tx.clone()).await;
            if let Some(url) = ret{
                castor_state.borrow_mut().current_url = url;
                entry.set_text(&castor_state.borrow().current_url);
            }
            let mut state = castor_state.borrow_mut();
            state.history_index += 1;
            // make the borrow checker happy
            let index = state.history_index;
            state.history.insert(index, entry.text().to_string());
            state.history.truncate(index+1);
        }));
    }));

    back_button.connect_clicked(clone!(@strong castor_state, @strong client, @weak page_content,
        @weak window, @weak forward_button, @weak url_bar, @strong tx => move |button| {
        let mut state = castor_state.borrow_mut();
        state.history_index -= 1;
        if state.history_index == 0 {
            button.set_sensitive(false);
        }
        forward_button.set_sensitive(true);
        let url = state.history[state.history_index].clone();

        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@strong castor_state, @strong client, @weak page_content,
            @weak window, @strong tx, @weak url_bar => async move {
            let ret = load_page(&mut client.borrow_mut(), castor_state.borrow_mut().current_url.clone(), url, &page_content, &window, tx.clone()).await;
            if let Some(url) = ret{
                castor_state.borrow_mut().current_url = url;
                url_bar.set_text(&castor_state.borrow().current_url);
            }
        }));
    }));

    forward_button.connect_clicked(clone!(@strong castor_state, @strong client, @weak page_content,
        @weak window, @weak back_button, @weak url_bar, @strong tx => move |button| {
        let mut state = castor_state.borrow_mut();
        state.history_index += 1;
        if state.history_index == state.history.len()-1 {
            button.set_sensitive(false);
        }
        back_button.set_sensitive(true);
        let url = state.history[state.history_index].clone();

        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@strong castor_state, @strong client, @weak page_content,
            @weak window, @strong tx, @weak url_bar => async move {
            let ret = load_page(&mut client.borrow_mut(), castor_state.borrow_mut().current_url.clone(), url, &page_content, &window, tx.clone()).await;
            if let Some(url) = ret{
                castor_state.borrow_mut().current_url = url;
                url_bar.set_text(&castor_state.borrow().current_url);
            }
        }));
    }));

    refresh_button.connect_clicked(clone!(@strong castor_state, @strong client, @weak page_content,
        @weak window, @strong tx =>  move |_| {
        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@strong castor_state, @strong client, @weak page_content, @weak window, @strong tx => async move {
            let current_url = castor_state.borrow().current_url.clone();
            load_page(&mut client.borrow_mut(), current_url.clone(), current_url, &page_content, &window, tx.clone()).await;
        }));
    }));

    rx.attach(None, clone!(@weak forward_button, @weak back_button, @strong client, @strong castor_state,
        @weak window, @strong tx, @weak url_bar => @default-return Continue(false), move |url| {
        forward_button.set_sensitive(false);
        back_button.set_sensitive(true);

        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@strong castor_state, @strong client, @weak page_content,
            @weak window, @strong tx, @weak url_bar => async move {
            let ret = load_page(&mut client.borrow_mut(), castor_state.borrow_mut().current_url.clone(), url, &page_content, &window, tx.clone()).await;
            if let Some(url) = ret{
                castor_state.borrow_mut().current_url = url;
                url_bar.set_text(&castor_state.borrow().current_url);
            }
            let mut state = castor_state.borrow_mut();
            state.history_index += 1;
            // make the borrow checker happy
            let index = state.history_index;
            state.history.insert(index, url_bar.text().to_string());
            state.history.truncate(index+1);
        }));
        Continue(true)
    }));

    Ok(window)
}

fn gemtext_to_text_buffer(gemtext: Gemtext, text_view: &TextView, link_tx: Sender<String>) {
    let buffer = text_view.buffer();
    for element in gemtext.elements {
        match element {
            gemtext::Element::Text(mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["plaintext"]);
            }
            gemtext::Element::Link(url, text) => {
                let link = Button::builder().label(&text).tooltip_text(&url).build();
                let anchor = TextChildAnchor::new();
                buffer.insert_child_anchor(&mut buffer.end_iter(), &anchor);
                text_view.add_child_at_anchor(&link, &anchor);
                buffer.insert(&mut buffer.end_iter(), "\n");

                link.connect_clicked(clone!(@strong link_tx => move |button| {
                    link_tx.send(button.tooltip_text().unwrap().to_string())
                        .expect("Failed to send url upon click");
                }));
            }
            gemtext::Element::Heading(mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["header"]);
            }
            gemtext::Element::Subheading(mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["subheader"]);
            }
            gemtext::Element::Subsubheading(mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["subsubheader"]);
            }
            gemtext::Element::UnorderedList(items) => {
                for mut text in items {
                    text.insert_str(0, "•");
                    text += "\n";
                    buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["plaintext"]);
                }
            }
            gemtext::Element::BlockQuote(mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["plaintext"]);
            }
            gemtext::Element::Preformatted(_alt_text, mut text) => {
                text += "\n";
                buffer.insert_with_tags_by_name(&mut buffer.end_iter(), &text, &["preformatted"]);
            }
        }
    }
}

enum LoadPageError {
    RequestFailure(gemini::Error),
    EmptyBody(gemini::Response),
    NotGemtext(gemini::Response),
    GemtextParsing(gemini::Error, gemini::Response),
    InvalidUrl(url::ParseError),
    FailTemporary(gemini::header::FailTemporaryCode),
    FailPermanent(gemini::header::FailPermanentCode),
    CertFail(gemini::header::CertFailCode),
}

impl std::fmt::Display for LoadPageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_print = match self {
            LoadPageError::RequestFailure(err) => {
                format!("Page request failed with error: {err}")
            }
            LoadPageError::EmptyBody(response) => {
                match response.header.status {
                    // Input, and Redirect shouldn't be reachable.
                    // they are not errors and therefor load_page wouldn't create
                    // an error for them. However Succeess can result in an error
                    // if the body is empty.
                    gemini::header::StatusCode::Input(_)
                    | gemini::header::StatusCode::Redirect(_) => unreachable!(),
                    gemini::header::StatusCode::Success => {
                        String::from("Success, but empty response body")
                    }
                    gemini::header::StatusCode::FailTemporary(_) => {
                        format!("Temporary failure: {}", response.header.status)
                    }
                    gemini::header::StatusCode::FailPermanent(_) => {
                        format!("Permanent failure: {}", response.header.status)
                    }
                    gemini::header::StatusCode::CertFail(_) => {
                        format!("Certificate failure: {}", response.header.status)
                    }
                }
            }
            LoadPageError::NotGemtext(response) => {
                format!("Response wasn't gemtext. Meta: {}", response.header.meta)
            }
            LoadPageError::GemtextParsing(err, _) => {
                if let gemini::Error::GemtextFormat(_) = err {
                    format!("Gemtext parsing error: {err}")
                } else {
                    unreachable!()
                }
            }
            LoadPageError::InvalidUrl(err) => {
                format!("Failed to parse url: {err}")
            }
            LoadPageError::FailTemporary(code) => {
                format!("Temporary failure: {code}")
            }
            LoadPageError::FailPermanent(code) => {
                format!("Permanent failure: {code}")
            }
            LoadPageError::CertFail(code) => {
                format!("Certificate failure: {code}\nCertificates are currently not supported")
            }
        };
        write!(f, "{}", to_print)
    }
}

// Returns the url of the page if loaded with no errors, otherwise returns none
#[async_recursion(?Send)]
async fn load_page(
    client: &mut gemini::Client,
    current_url: String,
    mut url: String,
    text_view: &TextView,
    window: &ApplicationWindow,
    link_tx: Sender<String>,
) -> Option<String> {
    if let Err(err) = url::Url::parse(&url) {
        let mut new_url = None;
        if matches!(err, url::ParseError::RelativeUrlWithoutBase) {
            let to_join = url::Url::parse(&current_url).unwrap();
            if let Ok(joined) = to_join.join(&url) {
                new_url = Some(joined.to_string())
            };
        }

        if new_url.is_some() {
            url = new_url.unwrap();
        }
        else {
            load_page_error_modal(&window, LoadPageError::InvalidUrl(err)).await;
            return None;
        }
    }

    let old_buffer = text_view.buffer();
    let buffer = TextBuffer::new(Some(&old_buffer.tag_table()));
    text_view.set_buffer(Some(&buffer));
    let result = client.async_request(url.clone()).await;
    match result {
        Ok(response) => match response.header.status {
            gemini::header::StatusCode::Input(code) => {
                let entry_dialog = MessageDialog::builder()
                    .transient_for(window)
                    .buttons(ButtonsType::OkCancel)
                    .text(&response.header.meta)
                    .build();
                let entry = Entry::new();
                match code {
                    gemini::header::InputCode::Input => {
                        entry_dialog.content_area().append(&entry);
                    }
                    gemini::header::InputCode::Sensitive => {
                        entry.set_visibility(false);
                        entry.set_invisible_char(Some('*'));
                        entry_dialog.content_area().append(&entry);
                    }
                }
                let url = url.clone()
                    + "?"
                    + &match entry_dialog.run_future().await {
                        gtk::ResponseType::Ok => entry.text().to_string(),
                        gtk::ResponseType::Cancel => return None,
                        _ => unreachable!(),
                    };
                let url = utf8_percent_encode(&url, percent_encoding::NON_ALPHANUMERIC).to_string();
                load_page(client, current_url, url, text_view, window, link_tx).await
            }
            gemini::header::StatusCode::Success => {
                if response.header.meta.starts_with("text/plaintext") {
                    match &response.body {
                        Some(body) => {
                            let text = String::from_utf8_lossy(body);
                            buffer.set_text("");
                            buffer.insert_with_tags_by_name(
                                &mut buffer.end_iter(),
                                &text,
                                &["plaintext"],
                            );
                            Some(url)
                        }
                        None => {
                            load_page_error_modal(window, LoadPageError::EmptyBody(response)).await;
                            None
                        }
                    }
                } else if response.header.meta.starts_with("text/gemini")
                    || response.header.meta.is_empty()
                {
                    match &response.body {
                        Some(body) => {
                            let text = String::from_utf8_lossy(&body);
                            match Gemtext::new(&text) {
                                Ok(gemtext) => {
                                    gemtext_to_text_buffer(gemtext, &text_view, link_tx);
                                    Some(url)
                                }
                                Err(err) => {
                                    load_page_error_modal(
                                        window,
                                        LoadPageError::GemtextParsing(err, response),
                                    )
                                    .await;
                                    None
                                }
                            }
                        }
                        None => {
                            load_page_error_modal(window, LoadPageError::EmptyBody(response)).await;
                            None
                        }
                    }
                } else {
                    load_page_error_modal(window, LoadPageError::NotGemtext(response)).await;
                    None
                }
            }
            gemini::header::StatusCode::Redirect(code) => {
                let redirect_dialog_builder = MessageDialog::builder()
                    .transient_for(window)
                    .modal(true)
                    .buttons(ButtonsType::YesNo);
                let user_response = match code {
                    gemini::header::RedirectCode::Temporary => {
                        let redirect_dialog = redirect_dialog_builder
                            .text(&format!("This website has a temporary redirect to {}\nWould you like to continue?", response.header.meta))
                            .build();
                        let user_response = redirect_dialog.run_future().await;
                        redirect_dialog.close();
                        user_response
                    }
                    gemini::header::RedirectCode::Permanent => {
                        let redirect_dialog = redirect_dialog_builder
                            .text(&format!("This website has a permanent redirect to {}\nWould you like to continue?", response.header.meta))
                            .build();
                        let user_response = redirect_dialog.run_future().await;
                        redirect_dialog.close();
                        user_response
                    }
                };
                if matches!(user_response, gtk::ResponseType::Yes) {
                    load_page(
                        client,
                        current_url,
                        response.header.meta,
                        text_view,
                        window,
                        link_tx,
                    )
                    .await
                } else {
                    None
                }
            }
            gemini::header::StatusCode::FailTemporary(code) => {
                load_page_error_modal(window, LoadPageError::FailTemporary(code)).await;
                None
            }
            gemini::header::StatusCode::FailPermanent(code) => {
                load_page_error_modal(window, LoadPageError::FailPermanent(code)).await;
                None
            }
            gemini::header::StatusCode::CertFail(code) => {
                load_page_error_modal(window, LoadPageError::CertFail(code)).await;
                None
            }
        },
        Err(e) => {
            load_page_error_modal(window, LoadPageError::RequestFailure(e)).await;
            None
        }
    }
}

async fn load_page_error_modal(window: &ApplicationWindow, err: LoadPageError) {
    let error_dialog = MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .text(&format!("{err}"))
        .build();
    error_dialog.run_future().await;
    error_dialog.close();
}
