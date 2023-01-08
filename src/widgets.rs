use std::ops::Deref;

use druid::{Data, Widget};

use crate::delegate::PAGE_LOADED;

pub struct Scroll<T: Data, W: Widget<T>> {
    scroll: druid::widget::Scroll<T, W>,
}

impl<T: Data, W: Widget<T>> Scroll<T, W> {
    pub fn new(child: W) -> Self {
        Scroll {
            scroll: druid::widget::Scroll::new(child),
        }
    }
}

impl<T: Data, W: Widget<T>> Deref for Scroll<T, W> {
    type Target = druid::widget::Scroll<T, W>;

    fn deref(&self) -> &Self::Target {
        &self.scroll
    }
}

impl<T: Data, W: Widget<T>> Widget<T> for Scroll<T, W> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        if let druid::Event::Command(cmd) = event {
            if cmd.is(PAGE_LOADED) {
                self.scroll.scroll_to(druid::Rect::new(0.0, 0.0, 1.0, 1.0));
            }
        }

        self.scroll.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.scroll.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.scroll.update(ctx, old_data, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.scroll.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.scroll.paint(ctx, data, env)
    }
}
