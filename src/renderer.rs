use crate::prelude::*;

pub trait Renderer {
    type Target;
    type Data;

    fn view<'a>(
        &self,
        target: &Self::Target,
        ctx: &mut RenderContext<'a>,
        sender: Sender<Box<dyn Messenger<Target = Self::Data>>>,
    ) -> Node<'a>;
}

pub trait ContextRenderer<A> {}

impl<A, T> ContextRenderer<A> for T where T: Renderer<Target = A, Data = A> {}

impl<T> Renderer for dyn ContextRenderer<T> {
    type Target = Container<T>;
    type Data = Container<T>;

    fn view<'a>(
        &self,
        target: &Self::Target,
        ctx: &mut RenderContext<'a>,
        sender: Sender<Box<dyn Messenger<Target = Self::Data>>>,
    ) -> Node<'a> {
        let bump = ctx.bump;
        if let Some(data) = target.data.try_lock() {
            target.renderer.view(&*data, ctx, target.sender.clone())
        } else {
            dodrio!(bump, <template></template>)
        }
    }
}

impl<'a, T> dodrio::Render<'a> for Container<T> {
    fn render(&self, cx: &mut RenderContext<'a>) -> Node<'a> {
        let bump = cx.bump;
        if let Some(data) = self.data.try_lock() {
            self.renderer.view(&*data, cx, self.sender.clone())
        } else {
            dodrio!(bump, <template></template>)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub enum Device {
        pc,
        mobile,
    }

    pub struct Data {
        state: i32,
    }

    impl Renderer for Device {
        type Target = Data;
        type Data = Data;

        fn view<'a>(
            &self,
            target: &Self::Target,
            ctx: &mut RenderContext<'a>,
            sender: Sender<Box<dyn crate::messenger::Messenger<Target = Self::Data>>>,
        ) -> Node<'a> {
            let bump = ctx.bump;
            let state = bf!(in bump, "{}", &target.state).into_bump_str();

            match self {
                Device::pc => dodrio!(bump, <div class={state}>"I'm on pc"</div>),
                Device::mobile => dodrio!(bump, <div class={state}>"I'm on mobile"</div>),
            }
        }
    }
}
