use std::marker::PhantomData;

use druid::{widget::Controller, Widget};

/// wraps a [TextBox] widget to add an on_submit event watcher
pub struct InputWrapper<T, F: Fn(&mut druid::EventCtx, &mut T)> {
    on_submit: F,
    _phantom_data: PhantomData<T>,
}

impl<T, F: Fn(&mut druid::EventCtx, &mut T)> InputWrapper<T, F> {
    pub fn new(func: F) -> InputWrapper<T, F> {
        InputWrapper {
            on_submit: func,
            _phantom_data: PhantomData,
        }
    }
}

impl<T, F: Fn(&mut druid::EventCtx, &mut T), W: Widget<T>> Controller<T, W> for InputWrapper<T, F> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        if let druid::Event::KeyUp(druid::KeyEvent {
            key: druid::KbKey::Enter,
            ..
        }) = event
        {
            (self.on_submit)(ctx, data);
        }
        // Always pass on the event!
        child.event(ctx, event, data, env)
    }
}
