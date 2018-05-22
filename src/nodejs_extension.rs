#[macro_use]
extern crate neon;

use neon::js::JsString;
use neon::vm::{Call, JsResult};

fn hello(call: Call) -> JsResult<JsString> {
    let scope = call.scope;
    Ok(JsString::new(scope, "hello node").unwrap())
}

register_module!(m, { m.export("hello", hello) });
