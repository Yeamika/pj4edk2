use std::{fs,path::PathBuf};
use pj4edk2_ui_package::{*,pocket_package::embedded};
fn align(n:usize)->usize{(n+15)&!15}
fn put(b:&mut [u8],at:usize,n:usize){b[at..at+4].copy_from_slice(&(n as u32).to_le_bytes());}
fn pak(files:&[(String,Vec<u8>)])->Vec<u8>{
 let names=32+files.len()*24;let mut out=vec![0;names];let mut offsets=Vec::new();
 for(name,_)in files{offsets.push(out.len()-names);out.extend_from_slice(name.as_bytes());}
 out.resize(align(out.len()),0);out[..4].copy_from_slice(b"DCPK");out[4..6].copy_from_slice(&1u16.to_le_bytes());put(&mut out,8,files.len());put(&mut out,12,32);put(&mut out,16,names);
 for(i,(name,bytes))in files.iter().enumerate(){let at=32+i*24;let offset=out.len();let hash=name.bytes().fold(0x811c9dc5u32,|h,b|(h^b as u32).wrapping_mul(0x01000193));put(&mut out,at,hash as usize);put(&mut out,at+4,offset);put(&mut out,at+8,bytes.len());put(&mut out,at+12,offsets[i]);out[at+16..at+18].copy_from_slice(&(name.len()as u16).to_le_bytes());out.extend_from_slice(bytes);out.resize(align(out.len()),0);}
 out
}
fn main(){
 let args=std::env::args().skip(1).collect::<Vec<_>>();assert_eq!(args.len(),2,"usage: pack-ui UI-DIRECTORY OUTPUT.pocket");
 let root=PathBuf::from(&args[0]).canonicalize().unwrap();let manifest:serde_json::Value=serde_json::from_slice(&fs::read(root.join("ui.json")).unwrap()).unwrap();
 let font=root.join(manifest["font"].as_str().unwrap()).canonicalize().unwrap();assert!(font.starts_with(&root));
 let mut files=vec![("font.ttf".into(),fs::read(font).unwrap())];
 if let Some(name)=manifest.get("font_license") {
  let license=root.join(name.as_str().expect("font_license must be a path")).canonicalize().unwrap();assert!(license.starts_with(&root));
  files.push(("font-license.txt".into(),fs::read(license).unwrap()));
 }
 for(i,bytes)in pocketjs_boot_assets::atlases(false).into_iter().enumerate(){files.push((format!("fonts/{i}.atlas"),bytes.to_vec()));}
 let mut js=String::new();for script in pocketjs_boot_ui::scripts(){js.push_str(script.source);js.push_str("\n;\n");}
 let pak=pak(&files);let identity=embedded::identity("ui","pj4edk2","PJ4EDK2 UI").unwrap();
 let bytes=embedded::encode(embedded::EmbeddedPackage{target:TARGET,host_abi:HOST_ABI,profile_hash:profile_hash(),identity:&identity,plan:PLAN,code:js.as_bytes(),pak:&pak}).unwrap();
 UiPackage::parse(&bytes).expect("validate using upstream reader");
 if let Some(parent)=PathBuf::from(&args[1]).parent(){fs::create_dir_all(parent).unwrap();}
 fs::write(&args[1],&bytes).unwrap();println!("{} bytes -> {}",bytes.len(),args[1]);
}
