#![allow(dead_code)]

#[macro_use]
extern crate neon;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate bincode;
extern crate glob;
extern crate num_cpus;
extern crate rayon;
extern crate regex;

// use neon::js::Key;
use neon::js::{JsArray, JsObject, JsString, Object};
use neon::mem::Handle;
use neon::vm::{Call, JsResult};
// use std::path::PathBuf;

mod haste_map;
mod js_parser;
mod types;
mod utils;

fn hello(call: Call) -> JsResult<JsString> {
    let scope = call.scope;
    Ok(JsString::new(scope, "hello node").unwrap())
}

fn build_haste_map(call: Call) -> JsResult<JsObject> {
    // let project_path = call.arguments
    //     .require(call.scope, 0)?
    //     .check::<JsString>()?
    //     .value() as String;

    // let haste = haste_map::derive_haste_map(PathBuf::from(&project_path));
    let haste = haste_map::read_haste_map_from_cache();
    let object: Handle<JsObject> = JsObject::new(call.scope);
    for source in haste {
        let k = source.path.display().to_string();
        let key = k.as_str();
        let a = JsArray::new(call.scope, source.dependencies.len() as u32);

        for (i, dep) in source.dependencies.iter().enumerate() {
            a.set(i as u32, JsString::new(call.scope, dep).unwrap())
                .unwrap();
        }
        object.set(key, a).unwrap();
    }

    Ok(object)
    // Ok(JsString::new(call.scope, &(&project_path)).unwrap())
}

fn get_module_names(call: Call) -> JsResult<JsArray> {
    // let modules: Vec<String> = HASTE
    //     .unwrap()
    //     .iter_mut()
    //     .flat_map(|source| source.dependencies.clone())
    //     .collect();
    let array: Handle<JsArray> = JsArray::new(call.scope, 4 as u32);
    Ok(array)
}

register_module!(m, {
    m.export("hello", hello)?;
    m.export("buildHasteMap", build_haste_map)?;
    Ok(())
});
