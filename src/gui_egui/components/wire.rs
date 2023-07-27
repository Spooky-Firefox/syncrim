use crate::common::{EguiComponent, Ports, SignalUnsigned, Simulator};
use crate::components::Wire;
use crate::gui_egui::component_ui::*;
use crate::gui_egui::editor::{EditorMode, EditorRenderReturn, SnapPriority};
use crate::gui_egui::gui::EguiExtra;
use crate::gui_egui::helper::offset_helper;
use egui::{Color32, PointerButton, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2};

#[typetag::serde]
impl EguiComponent for Wire {
    fn render(
        &self,
        ui: &mut Ui,
        _context: &mut EguiExtra,
        simulator: Option<&mut Simulator>,
        offset: Vec2,
        scale: f32,
        clip_rect: Rect,
        editor_mode: EditorMode,
    ) -> Option<Vec<Response>> {
        let oh: fn((f32, f32), f32, Vec2) -> Pos2 = offset_helper;
        let offset = offset;
        let s = scale;
        let o = offset;
        let mut line_vec = vec![];
        for pos in self.pos.clone() {
            line_vec.push(oh(pos, s, o));
        }

        ui.painter().add(Shape::line(
            line_vec.clone(),
            Stroke {
                width: scale,
                color: Color32::BLACK,
            },
        ));
        let mut r_vec = vec![];

        for (i, _) in line_vec[1..].iter().enumerate() {
            let (line_top, line_bottom) = if line_vec[i].x > line_vec[i + 1].x {
                (line_vec[i + 1].x, line_vec[i].x)
            } else {
                (line_vec[i].x, line_vec[i + 1].x)
            };
            let (line_left, line_right) = if line_vec[i].y > line_vec[i + 1].y {
                (line_vec[i + 1].y, line_vec[i].y)
            } else {
                (line_vec[i].y, line_vec[i + 1].y)
            };
            let rect = Rect {
                min: Pos2::new(line_top, line_left),
                max: Pos2::new(line_bottom, line_right),
            };

            let r = rect_with_hover(rect, clip_rect, editor_mode, ui, self.id.clone(), |ui| {
                ui.label(format!("Id: {}", self.id.clone()));
                if let Some(s) = &simulator {
                    ui.label({
                        let r: Result<SignalUnsigned, String> =
                            s.get_input_val(&self.input).try_into();
                        match r {
                            Ok(data) => format!("{:#x}", data),
                            _ => format!("{:?}", r),
                        }
                    });
                }
            });
            r_vec.push(r);
        }

        Some(r_vec)
    }

    fn render_editor(
        &mut self,
        ui: &mut Ui,
        context: &mut EguiExtra,
        simulator: Option<&mut Simulator>,
        offset: Vec2,
        scale: f32,
        clip_rect: Rect,
        id_ports: &[(crate::common::Id, Ports)],
        editor_mode: EditorMode,
    ) -> EditorRenderReturn {
        let mut delete = false;
        let r_vec = Wire::render(
            self,
            ui,
            context,
            simulator,
            offset,
            scale,
            clip_rect,
            editor_mode,
        )
        .unwrap();

        for (i, resp) in r_vec.iter().enumerate() {
            if resp.dragged_by(PointerButton::Primary) {
                let delta = resp.drag_delta() / scale;
                self.pos[i] = (self.pos[i].0 + delta.x, self.pos[i].1 + delta.y);
                self.pos[i + 1] = (self.pos[i + 1].0 + delta.x, self.pos[i + 1].1 + delta.y);
            }
            if resp.drag_released_by(PointerButton::Primary)
                && resp.interact_pointer_pos().unwrap().x < offset.x
            {
                delete = true;
            }
            properties_window(
                ui,
                self.id.clone(),
                resp,
                &mut context.properties_window,
                |ui| {
                    let mut clicked_dropdown = false;
                    input_change_id(ui, &mut context.id_tmp, &mut self.id, id_ports);
                    pos_slider(ui, &mut self.pos[i]);
                    pos_slider(ui, &mut self.pos[i + 1]);
                    clicked_dropdown |= input_selector(
                        ui,
                        &mut self.input,
                        crate::components::WIRE_INPUT_ID.to_string(),
                        id_ports,
                        self.id.clone(),
                    );
                    clicked_dropdown
                },
            );
        }

        EditorRenderReturn {
            delete,
            resp: Some(r_vec),
        }
    }

    fn ports_location(&self) -> Vec<(crate::common::Id, Pos2)> {
        let mut vec: Vec<(crate::common::Id, Pos2)> = vec![];
        for (i, pos) in self.pos.iter().enumerate() {
            vec.push((
                format!("{}-{}", crate::components::WIRE_INPUT_ID, i),
                Pos2 { x: pos.0, y: pos.1 },
            ));
        }
        vec
    }

    fn snap_priority(&self) -> SnapPriority {
        SnapPriority::Wire
    }
}
