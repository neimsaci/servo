/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::TextEncoderBinding::TextEncoderMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::{CreateWith, Uint8Array};
use std::ptr;
use std::ptr::NonNull;

#[dom_struct]
pub struct TextEncoder {
    reflector_: Reflector,
}

impl TextEncoder {
    fn new_inherited() -> TextEncoder {
        TextEncoder {
            reflector_: Reflector::new(),
        }
    }

    fn new(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<TextEncoder> {
        reflect_dom_object_with_proto(Box::new(TextEncoder::new_inherited()), global, proto)
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TextEncoder>> {
        Ok(TextEncoder::new(global, proto))
    }
}

impl TextEncoderMethods for TextEncoder {
    // https://encoding.spec.whatwg.org/#dom-textencoder-encoding
    fn Encoding(&self) -> DOMString {
        DOMString::from("utf-8")
    }

    #[allow(unsafe_code)]
    // https://encoding.spec.whatwg.org/#dom-textencoder-encode
    fn Encode(&self, cx: JSContext, input: USVString) -> NonNull<JSObject> {
        let encoded = input.0.as_bytes();

        unsafe {
            rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
            assert!(
                Uint8Array::create(*cx, CreateWith::Slice(&encoded), js_object.handle_mut())
                    .is_ok()
            );

            NonNull::new_unchecked(js_object.get())
        }
    }
}
