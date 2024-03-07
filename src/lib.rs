use mixlayer::{
    graph::{MxlData, MxlNode, MxlNodeCtx, MxlTransform},
    Frame, MxlGraph, MxlNodeRef,
};

use anyhow::Result;
use serde::Serialize;

use handlebars::{Handlebars, Template};

pub trait HandlebarsNodeOps<I, O: MxlData + Serialize> {
    fn handlebars(
        &self,
        g: &mut MxlGraph,
        template: impl AsRef<str>,
    ) -> Result<MxlNodeRef<O, String>>;
}

impl<I, O: MxlData + Serialize> HandlebarsNodeOps<I, O> for MxlNodeRef<I, O> {
    fn handlebars(
        &self,
        g: &mut MxlGraph,
        template: impl AsRef<str>,
    ) -> Result<MxlNodeRef<O, String>> {
        let hbs = HbsTemplateXform::new(template)?;
        Ok(self.transform(g, hbs))
    }
}

/// Takes an input and applies it to a Handlebars template
pub struct HbsTemplateXform<I>
where
    I: MxlData + Serialize,
{
    handlebars: handlebars::Handlebars<'static>,
    _marker: std::marker::PhantomData<I>,
}

impl<I> HbsTemplateXform<I>
where
    I: MxlData + Serialize,
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

impl<I> MxlNode for HbsTemplateXform<I>
where
    I: MxlData + Serialize,
{
    fn tick(&mut self, ctx: &mut MxlNodeCtx) -> Result<()> {
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

impl<I> MxlTransform for HbsTemplateXform<I>
where
    I: MxlData + Serialize,
{
    type Input = I;
    type Output = String;
}
