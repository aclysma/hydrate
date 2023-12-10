use egui::*;
use hydrate_model::AssetId;
use std::sync::Mutex;

#[derive(Clone)]
pub enum DragDropPayload {
    AssetReference(AssetId),
}

lazy_static::lazy_static! {
    static ref DRAG_DROP_PAYLOAD : Mutex<Option<DragDropPayload>> = {
        Mutex::new(None)
    };
}

pub fn render_payload(
    ui: &mut egui::Ui,
    payload: &DragDropPayload,
) {
    match payload {
        DragDropPayload::AssetReference(asset_id) => {
            ui.label(asset_id.to_string());
        }
    }
}

pub fn set_payload(payload: DragDropPayload) {
    *DRAG_DROP_PAYLOAD.lock().unwrap() = Some(payload);
}

pub fn peek_payload() -> Option<DragDropPayload> {
    DRAG_DROP_PAYLOAD.lock().unwrap().clone()
}

pub fn take_payload() -> Option<DragDropPayload> {
    DRAG_DROP_PAYLOAD.lock().unwrap().take()
}

pub fn drag_source(
    ui: &mut Ui,
    id: Id,
    payload: DragDropPayload,
    body: impl FnOnce(&mut Ui) -> Response,
) {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        //let response = ui.scope(body).response;
        let response = body(ui);

        // Check for drags:
        let mut allow_check_for_drag = false;
        ui.input(|input| {
            if let Some(press_origin) = input.pointer.press_origin() {
                if let Some(latest_pos) = input.pointer.latest_pos() {
                    if press_origin.distance(latest_pos) > 6.0 {
                        allow_check_for_drag = true;
                    }
                }
            }
        });
        if allow_check_for_drag && response.hovered() {
            ui.memory_mut(|mem| mem.set_dragged_id(id));
            set_payload(payload);
        }
    } else {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        let pointer_pos = ui.ctx().input(|input| input.pointer.latest_pos());

        // Paint the widget in-place
        body(ui);

        // Paint the widget floating under cursor and still allocate the space in the dragged widget's
        // container
        // let layer_id = LayerId::new(Order::Tooltip, id);
        // let response = ui.with_layer_id(layer_id, body).response;

        // original plan was to draw in another layer but this still allocates space causing visual problems
        // let layer_id = LayerId::new(Order::Tooltip, id);
        // let response = ui.with_layer_id(layer_id, |ui| {
        //     if let Some(payload) = peek_payload() {
        //         render_payload(ui, &payload);
        //     }
        // }).response;

        // Alternative way to draw a payload while also drawing the dragged widget in-place
        if let Some(mut pointer_pos) = pointer_pos {
            // A bit of a hack, but moving what we draw to be offset to the right from the cursor avoids the
            // cursor from hovering the drag source area rather than the intended drop target
            pointer_pos.x += 10.0;
            egui::Area::new("dragged_source")
                .movable(false)
                .fixed_pos(pointer_pos)
                .show(ui.ctx(), |ui| {
                    //ui.label("dragged")
                    render_payload(ui, &payload);
                    //body(ui);
                });
        }

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty [`Response`])
        // So this is fine!

        // if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
        //     let delta = pointer_pos - response.rect.center();
        //     ui.ctx().translate_layer(layer_id, delta);
        // }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());

    let margin = Vec2::splat(0.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    if is_being_dragged {
        let mut fill = Some(style.bg_fill);
        let mut stroke = style.bg_stroke;
        if !can_accept_what_is_being_dragged {
            //fill = ui.visuals().gray_out(fill);
            fill = None;
            stroke.color = ui.visuals().gray_out(stroke.color);
        }

        ui.painter().set(
            where_to_put_background,
            if let Some(fill) = fill {
                epaint::RectShape::new(rect, style.rounding, fill, stroke)
            } else {
                epaint::RectShape::stroke(rect, style.rounding, stroke)
            },
        );
    }

    InnerResponse::new(ret, response)
}

pub fn try_take_dropped_payload(
    ui: &egui::Ui,
    response: &egui::Response,
) -> Option<DragDropPayload> {
    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
    let any_released = ui.input(|input| input.pointer.any_released());
    if is_being_dragged && response.hovered() && any_released {
        take_payload()
    } else {
        None
    }
}
