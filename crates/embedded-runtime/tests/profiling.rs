//! Optional profiling must preserve the real guest/render path and turn off cleanly.
extern crate rquickjs_sys;
use pocketjs_embedded_runtime::{InputFrameBuilder,PocketJs,output::*};
mod support;
use support::{CaptureDisplay,config};
thread_local! { static CLOCK: core::cell::Cell<u64> = const { core::cell::Cell::new(0) }; }
fn clock_us()->u64 { CLOCK.with(|v| { let next=v.get()+100;v.set(next);next }) }
#[test]
fn profiling_splits_stages_without_changing_pixels_and_can_be_disabled() {
    let mut c=config();c.width=13;c.height=7;c.frame_buffer_budget=13*7*4;
    let readback=support::Readback::default();
    let mut vm=PocketJs::new(c).unwrap();
    vm.add_display(CaptureDisplay {framebuffer:FramebufferDisplay::new(vec![0;13*7*4],13,7,13*4,PixelFormat::Argb8888).unwrap(),readback:readback.clone()},RendererId::SOFTWARE).unwrap();
    let page=vm.replace_user(&["ui.setProp(1, 64, 0xff0000ff);"]).unwrap();
    vm.set_profile_clock(Some(clock_us));
    vm.advance(page,&InputFrameBuilder::new().frame()).unwrap();
    vm.present(page).unwrap();
    assert_eq!(vm.profile_us(),[100;6]);
    let pixels=readback.borrow().clone();
    assert!(pixels.chunks_exact(4).all(|p|p==[0,0,255,255]));
    vm.set_profile_clock(None);
    vm.advance(page,&InputFrameBuilder::new().frame()).unwrap();
    vm.present(page).unwrap();
    assert_eq!(vm.profile_us(),[0;6]);
    assert_eq!(*readback.borrow(),pixels);
}
