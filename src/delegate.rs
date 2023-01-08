use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Selector, Target};

use crate::data::CastorState;

pub const LINK_CLICKED: Selector<String> = Selector::<String>::new("link_clicked");
pub const PAGE_LOADED: Selector = Selector::new("page_loaded");

pub struct Delegate;

impl AppDelegate<CastorState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut CastorState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(LINK_CLICKED) {
            let url = cmd.get(LINK_CLICKED).unwrap().clone();
            let url = match url::Url::parse(&url) {
                Ok(url) => url,
                Err(url::ParseError::RelativeUrlWithoutBase) => {
                    let base =
                        url::Url::parse(&data.get_current_url()).expect("Failed to parse base url");
                    base.join(url.as_str())
                        .expect("Failed to combine relative url with base url")
                }
                _ => panic!("Unexpected return value when parsing url"),
            };

            data.load_page(url.as_str().to_string());
            data.push_to_history(url.as_str().to_string());
            ctx.submit_command(PAGE_LOADED);
            Handled::Yes
        }
        // We don't want to handle PAGE_LOADED, we want it to propagate down the ui tree
        else {
            Handled::No
        }
    }
}
