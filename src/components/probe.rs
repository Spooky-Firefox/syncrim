use crate::{
    common::{Component, Input, OutputType, Ports},
    gui_vizia::{popup::NewPopup, tooltip::new_component_tooltip, GuiData},
};
use serde::{Deserialize, Serialize};
use vizia::{
    prelude::*,
    vg::{Paint, Path},
};

#[derive(Serialize, Deserialize)]
pub struct Probe {
    pub id: String,
    pub pos: (f32, f32),
    pub input: Input,
}

#[typetag::serde]
impl Component for Probe {
    fn to_(&self) {
        println!("Probe");
    }

    fn get_id_ports(&self) -> (String, Ports) {
        (
            self.id.clone(),
            Ports {
                // Probes take one input
                inputs: vec![self.input.clone()],
                out_type: OutputType::Combinatorial,
                // No output value
                outputs: vec![],
            },
        )
    }

    // create view
    fn view(&self, cx: &mut Context) {
        println!("---- Create Probe View");
        View::build(ProbeView {}, cx, |cx| {
            let input = self.input.clone();

            Binding::new(cx, crate::gui_vizia::GuiData::clock, move |cx, _| {
                Label::new(cx, {
                    let simulator = GuiData::simulator.get(cx);
                    &format!(" {:?}", simulator.get_input_val(&input))
                })
                .hoverable(false);
            });
            NewPopup::new(cx, self.get_id_ports()).position_type(PositionType::SelfDirected);
        })
        .position_type(PositionType::SelfDirected)
        .left(Pixels(self.pos.0 - 10.0))
        .top(Pixels(self.pos.1 - 10.0))
        .width(Pixels(20.0))
        .height(Pixels(20.0))
        .on_press(|ex| ex.emit(PopupEvent::Switch))
        .tooltip(|cx| new_component_tooltip(cx, self));
    }
}

pub struct ProbeView {}

impl View for ProbeView {
    fn element(&self) -> Option<&'static str> {
        Some("Probe")
    }

    fn draw(&self, cx: &mut DrawContext<'_>, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        // println!("Probe draw {:?}", bounds);

        let mut path = Path::new();
        let mut paint = Paint::color(vizia::vg::Color::rgbf(0.0, 1.0, 1.0));
        paint.set_line_width(cx.logical_to_physical(1.0));

        path.move_to(bounds.left() + 0.5, bounds.top() + 0.5);
        path.line_to(bounds.right() + 0.5, bounds.top() + 0.5);
        path.line_to(bounds.right() + 0.5, bounds.bottom() + 0.5);
        path.line_to(bounds.left() + 0.5, bounds.bottom() + 0.5);
        path.line_to(bounds.left() + 0.5, bounds.top() + 0.5);

        canvas.fill_path(&path, &paint);
    }
}
