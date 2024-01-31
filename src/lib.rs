use valence::{
    graph::{VData, VNode, VNodeCtx, VTransform},
    Frame, Result, VGraph, VNodeRef,
};

use serde::Serialize;

use handlebars::{Handlebars, Template};

pub trait HandlebarsNodeOps<I, O: VData + Serialize> {
    fn handlebars(&self, g: &mut VGraph, template: impl AsRef<str>) -> Result<VNodeRef<O, String>>;
}

impl<I, O: VData + Serialize> HandlebarsNodeOps<I, O> for VNodeRef<I, O> {
    fn handlebars(&self, g: &mut VGraph, template: impl AsRef<str>) -> Result<VNodeRef<O, String>> {
        let hbs = HbsTemplateXform::new(template)?;
        Ok(self.transform(g, hbs))
    }
}

/// Takes an input and applies it to a Handlebars template
pub struct HbsTemplateXform<I>
where
    I: VData + Serialize,
{
    handlebars: handlebars::Handlebars<'static>,
    _marker: std::marker::PhantomData<I>,
}

impl<I> HbsTemplateXform<I>
where
    I: VData + Serialize,
{
    pub fn new(template: impl AsRef<str>) -> Result<Self> {
        let template = Template::compile(template.as_ref())?;
        let mut handlebars = Handlebars::new();

        handlebars.set_strict_mode(true);
        handlebars.register_template("main", template);

        Ok(Self {
            handlebars,
            _marker: std::default::Default::default(),
        })
    }
}

impl<I> VNode for HbsTemplateXform<I>
where
    I: VData + Serialize,
{
    fn tick(&mut self, ctx: &mut VNodeCtx) -> Result<()> {
        if ctx.recv_finished() {
            self.send(ctx, Frame::End)?;
        } else {
            if let Some(next) = self.recv(ctx) {
                if let Frame::Data(data) = next {
                    let output = self.handlebars.render("main", &data)?;
                    self.send(ctx, Frame::Data(output))?;
                }
            }
        }

        Ok(())
    }

    fn default_label(&self) -> Option<String> {
        Some("Handlebars".to_owned())
    }
}

impl<I> VTransform for HbsTemplateXform<I>
where
    I: VData + Serialize,
{
    type Input = I;
    type Output = String;
}
