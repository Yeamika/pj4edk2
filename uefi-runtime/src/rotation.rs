//! One clockwise rotation for portrait firmware scanout and its inverse input.
use pocketjs_embedded_runtime::output::Rect;

pub fn point(logical_height:usize,x:usize,y:usize)->[usize;2] { [logical_height-1-y,x] }
pub fn touch(logical_height:u32,physical:[f32;2])->[f32;2] { [physical[1],logical_height as f32-1.-physical[0]] }
pub fn region(logical_height:u32,r:Rect)->Rect { Rect { x:logical_height-r.y-r.height,y:r.x,width:r.height,height:r.width } }

/// Trade limited overdraw for fewer synchronous GOP presentation waits.
pub fn combine(regions:&[Rect])->Option<Rect> {
    let first=*regions.first()?;
    if regions.len()<2 {return None;}
    let (mut left,mut top)=(first.x,first.y);
    let (mut right,mut bottom)=(first.x.checked_add(first.width)?,first.y.checked_add(first.height)?);
    let mut area=u64::from(first.width)*u64::from(first.height);
    for r in &regions[1..] {
        left=left.min(r.x);top=top.min(r.y);
        right=right.max(r.x.checked_add(r.width)?);bottom=bottom.max(r.y.checked_add(r.height)?);
        area=area.saturating_add(u64::from(r.width)*u64::from(r.height));
    }
    let merged=Rect{x:left,y:top,width:right-left,height:bottom-top};
    let extra=u64::from(merged.width)*u64::from(merged.height);
    (extra<=area.saturating_add(600_000u64.saturating_mul((regions.len()-1) as u64))).then_some(merged)
}

/// Rotate only a logical damage rectangle into a retained physical framebuffer.
pub fn copy_region(source:&[u8],logical_width:usize,logical_height:usize,damage:Rect,target:&mut [u8]) {
    for y in damage.y as usize..(damage.y+damage.height) as usize {
        for x in damage.x as usize..(damage.x+damage.width) as usize {
            let [px,py]=point(logical_height,x,y);
            let from=(y*logical_width+x)*4;
            let to=(py*logical_height+px)*4;
            target[to..to+4].copy_from_slice(&source[from..from+4]);
        }
    }
}

/// Rotate into a compact GOP BLT rectangle, without a second full framebuffer.
pub fn copy_compact(source:&[u8],logical_width:usize,damage:Rect,target:&mut [u8]) {
    for y in 0..damage.height as usize { for x in 0..damage.width as usize {
        let from=((damage.y as usize+y)*logical_width+damage.x as usize+x)*4;
        let to=(x*damage.height as usize+damage.height as usize-1-y)*4;
        target[to..to+4].copy_from_slice(&source[from..from+4]);
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combine_limits_overdraw_and_includes_every_damage_rect() {
        let a=Rect{x:10,y:20,width:30,height:40};
        let b=Rect{x:80,y:90,width:20,height:30};
        let merged=combine(&[a,b]).unwrap();
        assert_eq!((merged.x,merged.y,merged.width,merged.height),(10,20,90,100));
        assert!(combine(&[a]).is_none());
        assert!(combine(&[a,Rect{x:4000,y:4000,width:20,height:20}]).is_none());
    }
    #[test]
    fn rotated_damage_matches_full_rotation_without_touching_other_pixels() {
        let (w,h)=(7,5);
        let source:alloc::vec::Vec<u8>=(0..w*h*4).map(|i|i as u8).collect();
        let mut full=alloc::vec![0;source.len()];
        for y in 0..h {for x in 0..w {
            let [px,py]=point(h,x,y);
            full[(py*h+px)*4..(py*h+px+1)*4].copy_from_slice(&source[(y*w+x)*4..(y*w+x+1)*4]);
        }}
        let damage=Rect{x:1,y:1,width:5,height:3};
        let mapped=region(h as u32,damage);
        let mut retained=alloc::vec![0xff;source.len()];
        copy_region(&source,w,h,damage,&mut retained);
        for y in 0..w {for x in 0..h {
            let i=(y*h+x)*4;
            let inside=(mapped.x as usize..(mapped.x+mapped.width) as usize).contains(&x) &&
                (mapped.y as usize..(mapped.y+mapped.height) as usize).contains(&y);
            assert_eq!(&retained[i..i+4],if inside {&full[i..i+4]} else {&[0xff;4]});
        }}
    }
    #[test]
    fn compact_strips_match_full_rotation_at_edges_and_partial_damage() {
        let (w,h)=(11,7);
        let source:alloc::vec::Vec<u8>=(0..w*h*4).map(|i|(i*17) as u8).collect();
        let mut full=alloc::vec![0;source.len()];
        copy_region(&source,w,h,Rect{x:0,y:0,width:w as u32,height:h as u32},&mut full);
        for damage in [Rect{x:0,y:0,width:11,height:7},Rect{x:2,y:1,width:8,height:5},Rect{x:10,y:6,width:1,height:1}] {
            for left in (damage.x..damage.x+damage.width).step_by(3) {
                let part=Rect{x:left,width:(damage.x+damage.width-left).min(3),..damage};
                let mapped=region(h as u32,part);
                let mut tile=alloc::vec![0xaa;3*h*4+4];
                copy_compact(&source,w,part,&mut tile);
                for y in 0..mapped.height as usize {for x in 0..mapped.width as usize {
                    let actual=(y*mapped.width as usize+x)*4;
                    let expected=((y+mapped.y as usize)*h+x+mapped.x as usize)*4;
                    assert_eq!(&tile[actual..actual+4],&full[expected..expected+4]);
                }}
                assert!(tile[(part.width*part.height*4) as usize..].iter().all(|&p|p==0xaa));
            }
        }
    }
    #[test]
    fn landscape_pixels_and_touch_land_on_the_same_panel_corners() {
        let source=[b'a',b'b',b'c',b'd',b'e',b'f']; // 3x2, row-major
        let mut panel=[0;6]; // 2x3
        for y in 0..2 { for x in 0..3 { let [px,py]=point(2,x,y);panel[py*2+px]=source[y*3+x]; } }
        assert_eq!(&panel,b"daebfc");
        assert_eq!(touch(2,[0.,0.]),[0.,1.]);
        assert_eq!(touch(2,[1.,2.]),[2.,0.]);
        let r=region(2,Rect{x:1,y:0,width:2,height:1});
        assert_eq!((r.x,r.y,r.width,r.height),(1,1,1,2));
    }
}
