#[allow(dead_code)]
#[path = "../../src/crates/concat/src/timeline_time.rs"]
mod timeline_time;
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
        Lanes {
            width: parent.width; height: parent.height;
            tracks: root.tracks; clips: root.clips; tool: root.tool;
            clip-split(id, seconds) => { root.hit(id, seconds); }
            time-origin: root.origin; scroll-left: root.scroll; seconds-per-pixel: root.zoom; frame-rate: root.fps;
            scrubbed(seconds) => { root.observed(seconds); }
        }
    }
}

fn click(app: &Probe, x: f32, y: f32) {
    let position=slint::LogicalPosition::new(x, y);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved {position});
    app.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {position,button:slint::platform::PointerEventButton::Left});
    app.window().dispatch_event(slint::platform::WindowEvent::PointerReleased {position,button:slint::platform::PointerEventButton::Left});
}
fn main() {
    let app=Probe::new().unwrap();
    app.show().unwrap();
    let seen=Rc::new(Cell::new(f32::NAN));
    let hit=Rc::new(Cell::new(false));
    app.on_observed({let seen=seen.clone(); move |s| seen.set(s)});
    app.on_hit({let hit=hit.clone(); move |id,_| {assert_eq!(id.as_str(), "probe"); hit.set(true)}});
    app.set_tracks(slint::ModelRc::new(slint::VecModel::from(vec![TrackData { id:"row".into(), height:120.0, visible:true, ..Default::default() }])));
    println!("num,den,zoom,scroll,frame,ideal_seconds,input_seconds,pixel,observed_seconds,expected_frame,actual_frame");
    let mut total=0;
    let mut changed=0;
    let mut hits=0;
    let mut mismatches=0;
    for (num,den) in [(24,1),(25,1),(30,1),(60,1),(24000,1001),(30000,1001)] {
        let fps=num as f64/den as f64;
        app.set_fps(fps as f32);
        for zoom in [0.0005_f32,0.005,0.05,0.5,1.5] {
            app.set_zoom(zoom);
            for scroll in [0.0_f32,60.0,3600.0,21600.0,86400.0,262144.0] {
                let base=timeline_time::origin(f64::from(scroll));
                app.set_origin(base as i32);
                app.set_scroll(timeline_time::local(f64::from(scroll),base));
                for pixel in [100.0_f64,300.0,500.0,700.0,900.0] {
                    let frame=((f64::from(scroll)+pixel*f64::from(zoom))*fps).round() as i64;
                    let ideal=frame as f64/fps;
                    let input=timeline_time::local(ideal,base);
                    let x=(input-timeline_time::local(f64::from(scroll),base))*(1.0/zoom);
                    assert!((0.0..1000.0).contains(&x));
                    seen.set(f32::NAN);
                    click(&app,x,12.0);
                    let actual=timeline_time::absolute(seen.get(),base);
                    assert!(actual.is_finite(), "real ruler callback must run");
                    let actual_frame=(f64::from(actual)*fps).round() as i64;
                    let input_frame=(timeline_time::absolute(input,base)*fps).round() as i64;
                    let error=(f64::from(actual)-ideal).abs();
                    let ulp_bound=f64::from(input.abs().max(1.0))*f64::from(f32::EPSILON)*3.0;
                    assert!(error <= ulp_bound, "unexpected conversion error");
                    assert_eq!(actual_frame,frame, "precise local frame identity");
                    if actual_frame != frame { mismatches+=1; }
                    if actual_frame != input_frame { changed+=1; }
                    println!("{num},{den},{zoom},{scroll},{frame},{ideal:.12},{input:.12},{x:.6},{actual:.12},{frame},{actual_frame}");
                    total+=1;
                    app.set_clips(slint::ModelRc::new(slint::VecModel::from(vec![ClipData {
                        id:"probe".into(), start:input, duration:zoom*60.0, row:0, name:"probe".into(), ..Default::default()
                    }])));
                    app.set_tool(TimelineTool::Split);
                    hit.set(false);
                    click(&app,x+20.0,80.0);
                    assert!(hit.get(), "clip positioned by real Lanes::at must be hit: {num}/{den} {zoom} {scroll} {x}");
                    hits+=1;
                    app.set_tool(TimelineTool::Select);
                    app.set_clips(Default::default());
                }
            }
        }
    }
    assert_eq!(mismatches,0);
    // Audit's old absolute f32 loss remains a before-fix comparison.
    let frame=15_728_641_i64;
    let stored=(frame as f64/60.0) as f32;
    assert_eq!((f64::from(stored)*60.0).round() as i64,frame+1);
    eprintln!("matrix: {total} ruler callbacks; {hits} real clip hit-tests; {mismatches} frame mismatches; {changed} introduced after the input narrowing; legacy absolute-f32 loss checked separately");
}
