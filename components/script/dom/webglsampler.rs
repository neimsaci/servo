/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{webgl_channel, WebGLCommand, WebGLSamplerId};
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLSampler {
    webgl_object: WebGLObject,
    #[no_trace]
    gl_id: WebGLSamplerId,
    marked_for_deletion: Cell<bool>,
}

#[derive(Clone, Copy)]
pub enum WebGLSamplerValue {
    Float(f32),
    GLenum(u32),
}

fn validate_params(pname: u32, value: WebGLSamplerValue) -> bool {
    match value {
        WebGLSamplerValue::GLenum(value) => {
            let allowed_values = match pname {
                constants::TEXTURE_MIN_FILTER => &[
                    constants::NEAREST,
                    constants::LINEAR,
                    constants::NEAREST_MIPMAP_NEAREST,
                    constants::LINEAR_MIPMAP_NEAREST,
                    constants::NEAREST_MIPMAP_LINEAR,
                    constants::LINEAR_MIPMAP_LINEAR,
                ][..],
                constants::TEXTURE_MAG_FILTER => &[constants::NEAREST, constants::LINEAR][..],
                constants::TEXTURE_WRAP_R |
                constants::TEXTURE_WRAP_S |
                constants::TEXTURE_WRAP_T => &[
                    constants::CLAMP_TO_EDGE,
                    constants::MIRRORED_REPEAT,
                    constants::REPEAT,
                ][..],
                constants::TEXTURE_COMPARE_MODE => {
                    &[constants::NONE, constants::COMPARE_REF_TO_TEXTURE][..]
                },
                constants::TEXTURE_COMPARE_FUNC => &[
                    constants::LEQUAL,
                    constants::GEQUAL,
                    constants::LESS,
                    constants::GREATER,
                    constants::EQUAL,
                    constants::NOTEQUAL,
                    constants::ALWAYS,
                    constants::NEVER,
                ][..],
                _ => &[][..],
            };
            allowed_values.contains(&value)
        },
        WebGLSamplerValue::Float(_) => match pname {
            constants::TEXTURE_MIN_LOD | constants::TEXTURE_MAX_LOD => true,
            _ => false,
        },
    }
}

impl WebGLSampler {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLSamplerId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            gl_id: id,
            marked_for_deletion: Cell::new(false),
        }
    }

    pub fn new(context: &WebGLRenderingContext) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GenerateSampler(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(Self::new_inherited(context, id)),
            &*context.global(),
        )
    }

    pub fn delete(&self, operation_fallibility: Operation) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);

            let command = WebGLCommand::DeleteSampler(self.gl_id);
            let context = self.upcast::<WebGLObject>().context();
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(command),
                Operation::Infallible => context.send_command(command),
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get()
    }

    pub fn bind(
        &self,
        context: &WebGLRenderingContext,
        unit: u32,
    ) -> Result<(), canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        context.send_command(WebGLCommand::BindSampler(unit, self.gl_id));
        Ok(())
    }

    pub fn set_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
        value: WebGLSamplerValue,
    ) -> Result<(), canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        if !validate_params(pname, value) {
            return Err(InvalidEnum);
        }
        let command = match value {
            WebGLSamplerValue::GLenum(value) => {
                WebGLCommand::SetSamplerParameterInt(self.gl_id, pname, value as i32)
            },
            WebGLSamplerValue::Float(value) => {
                WebGLCommand::SetSamplerParameterFloat(self.gl_id, pname, value)
            },
        };
        context.send_command(command);
        Ok(())
    }

    pub fn get_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
    ) -> Result<WebGLSamplerValue, canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        match pname {
            constants::TEXTURE_MIN_FILTER |
            constants::TEXTURE_MAG_FILTER |
            constants::TEXTURE_WRAP_R |
            constants::TEXTURE_WRAP_S |
            constants::TEXTURE_WRAP_T |
            constants::TEXTURE_COMPARE_FUNC |
            constants::TEXTURE_COMPARE_MODE => {
                let (sender, receiver) = webgl_channel().unwrap();
                context.send_command(WebGLCommand::GetSamplerParameterInt(
                    self.gl_id, pname, sender,
                ));
                Ok(WebGLSamplerValue::GLenum(receiver.recv().unwrap() as u32))
            },
            constants::TEXTURE_MIN_LOD | constants::TEXTURE_MAX_LOD => {
                let (sender, receiver) = webgl_channel().unwrap();
                context.send_command(WebGLCommand::GetSamplerParameterFloat(
                    self.gl_id, pname, sender,
                ));
                Ok(WebGLSamplerValue::Float(receiver.recv().unwrap()))
            },
            _ => Err(InvalidEnum),
        }
    }
}

impl Drop for WebGLSampler {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}
