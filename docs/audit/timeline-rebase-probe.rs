// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

use slint::ComponentHandle;
use std::{cell::Cell, rc::Rc};
slint::slint! {
    import { Lanes } from "../../src/crates/concat/ui/timeline/lanes.slint";
    import { TrackData, ClipData, TimelineTool } from "../../src/crates/concat/ui/timeline/model.slint";
    export { TrackData, ClipData, TimelineTool }
    export component Probe inherits Window {
        width: 1000px; height: 200px;
        in property<float> scroll;
        in property<int> origin;
        in property<float> zoom;
        in property<float> fps;
        in property<[TrackData]> tracks;
        in property<[ClipData]> clips;
        in property<TimelineTool> tool;
        callback observed(float);
        callback hit(string, float);
        callback released();
        callback dragged(float);
        callback cancelled();
        Lanes {
            width: parent.width; height: parent.height;
            tracks: root.tracks; clips: root.clips; tool: root.tool;
            clip-split(id, seconds) => { root.hit(id, seconds); }
            clip-released => { root.released(); }
            clip-dragged(s,p) => { root.dragged(s); }
            time-origin: root.origin; clip-cancelled => {root.cancelled();}
            scroll-left: root.scroll; seconds-per-pixel: root.zoom; frame-rate: root.fps;
            scrubbed(seconds) => { root.observed(seconds); }
        }
    }
}

fn main() {
    let app=Probe::new().unwrap();
    app.show().unwrap();
    let released=Rc::new(Cell::new(0));
    let dragged=Rc::new(Cell::new(0.0f32));
    let cancelled=Rc::new(Cell::new(0));
    app.on_cancelled({let r=cancelled.clone();move ||r.set(r.get()+1)});
    app.on_released({let r=released.clone();move ||r.set(r.get()+1)});
    app.on_dragged({let r=dragged.clone();move |s|r.set(s)});
    app.set_zoom(0.005); app.set_scroll(1023.75); app.set_fps(60.0);
    app.set_tracks(slint::ModelRc::new(slint::VecModel::from(vec![TrackData {height:120.0,visible:true,..Default::default()}])));
    app.set_clips(slint::ModelRc::new(slint::VecModel::from(vec![ClipData {id:"probe".into(),start:1023.8,duration:5.0,row:0,..Default::default()}])));
    let position=slint::LogicalPosition::new(50.0,80.0);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved {position});
    app.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {position,button:slint::platform::PointerEventButton::Left});
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved {position:slint::LogicalPosition::new(70.0,80.0)});
    assert!((dragged.get()-0.1).abs()<1e-5);
    app.set_origin(1024); app.set_scroll(-0.25);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved {position:slint::LogicalPosition::new(71.0,80.0)});
    assert!((dragged.get()-0.105).abs()<1e-4,"rebase delta {}",dragged.get());
    app.set_scroll(-0.20);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved {position:slint::LogicalPosition::new(72.0,80.0)});
    assert!((dragged.get()-0.16).abs()<1e-4,"scroll delta {}",dragged.get());
    app.window().dispatch_event(slint::platform::WindowEvent::PointerExited);
    assert_eq!(cancelled.get(),1);
    assert!(dragged.get()>0.0);
    assert_eq!(released.get(),0,"cancelled pointer must not request a committed release");
}
