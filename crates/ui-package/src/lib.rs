//! PJ4EDK2 UI host profile inside the upstream PocketJS v2 container.
#![no_std]
pub use pocket_package;
pub const TARGET:&str="pj4edk2-ui";
pub const HOST_ABI:u32=1;
pub const MAX_BYTES:usize=16*1024*1024;
pub const PLAN:&[u8]=b"{\"host\":\"pj4edk2-ui\",\"version\":1,\"font_slots\":8}";
pub fn profile_hash()->[u8;32]{pocket_package::sha256::sha256(b"pj4edk2-ui-v1;ui-api=1;fonts=dcfa-v3;slots=8;services=boot-v1")}
#[derive(Debug)]
pub enum Error{Size,Admission,Assets,Script,Profile}
pub struct UiPackage<'a>{pub javascript:&'a str,pak:&'a [u8]}
impl<'a> UiPackage<'a>{
 pub fn parse(bytes:&'a [u8])->Result<Self,Error>{
  if bytes.len()>MAX_BYTES{return Err(Error::Size)}
  let guest=pocket_package::select_guest(bytes,TARGET,HOST_ABI,&profile_hash(),false).map_err(|_|Error::Admission)?;
  if guest.plan!=PLAN{return Err(Error::Profile)}
  pocket_package::pak::checked_entries(guest.pak).map_err(|_|Error::Assets)?;
  let js=guest.js.strip_suffix(&[0]).ok_or(Error::Script)?;
  if js.contains(&0){return Err(Error::Script)}
  let package=Self{javascript:core::str::from_utf8(js).map_err(|_|Error::Script)?,pak:guest.pak};
  for slot in 0..8{package.atlas(slot)?;}
  package.asset("font.ttf").ok_or(Error::Assets)?;
  Ok(package)
 }
 pub fn asset(&self,name:&str)->Option<&'a [u8]>{pocket_package::pak::find(self.pak,name)}
 pub fn atlas(&self,slot:usize)->Result<&'a [u8],Error>{
  let name=match slot{0=>"fonts/0.atlas",1=>"fonts/1.atlas",2=>"fonts/2.atlas",3=>"fonts/3.atlas",4=>"fonts/4.atlas",5=>"fonts/5.atlas",6=>"fonts/6.atlas",7=>"fonts/7.atlas",_=>return Err(Error::Assets)};
  let bytes=self.asset(name).ok_or(Error::Assets)?;
  if bytes.len()<24 || &bytes[..4]!=b"DCFA" || bytes[4..6]!=[3,0] || bytes[12]!=slot as u8{return Err(Error::Assets)}
  Ok(bytes)
 }
}
