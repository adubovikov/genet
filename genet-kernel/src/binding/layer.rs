use binding::JsClass;
use genet_abi::{attr::AttrClass, layer::Layer, token::Token};
use genet_filter::{ast::Expr, unparser::unparse};
use genet_napi::napi::{
    CallbackInfo, Env, PropertyAttributes, PropertyDescriptor, Result, Status, TypedArrayType,
    Value, ValueRef, ValueType,
};
use std::rc::Rc;

pub fn wrapper(env: &Env) -> Rc<ValueRef> {
    fn ctor<'env>(env: &'env Env, _info: &CallbackInfo) -> Result<&'env Value> {
        env.get_null()
    }

    fn layer_id<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;
        env.create_string(&layer.id().to_string())
    }

    fn layer_attr<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;
        if let Some(id) = info.argv().get(0) {
            let id = match env.type_of(id)? {
                ValueType::Number => Token::from(env.get_value_uint32(id)?),
                _ => Token::from(env.get_value_string(env.coerce_to_string(id)?)?.as_str()),
            };
            if let Some(attr) = layer.attr(id) {
                let attr_class = env.get_constructor(JsClass::Attr as usize).unwrap();
                let instance = env.new_instance(&attr_class, &[])?;
                env.wrap(instance, attr)?;
                Ok(instance)
            } else {
                env.get_null()
            }
        } else {
            Err(Status::InvalidArg)
        }
    }

    fn layer_attrs<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;

        let headers = layer
            .headers()
            .map(|c| AttrClass::expand(c, &layer.data(), None))
            .flatten()
            .collect::<Vec<_>>();
        let hlen = headers.len();
        let attrs = layer.attrs().collect::<Vec<_>>();
        let attr_class = env.get_constructor(JsClass::Attr as usize).unwrap();
        let array = env.create_array(hlen + attrs.len())?;
        for (i, item) in headers.into_iter().enumerate() {
            let instance = env.new_instance(&attr_class, &[])?;
            env.wrap(instance, item)?;
            env.set_element(array, i as u32, instance)?;
        }
        for (i, item) in attrs.into_iter().enumerate() {
            let instance = env.new_instance(&attr_class, &[])?;
            env.wrap(instance, item)?;
            env.set_element(array, (hlen + i) as u32, instance)?;
        }
        Ok(array)
    }

    fn layer_payloads<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;
        let payloads = layer.payloads().collect::<Vec<_>>();
        let array = env.create_array(payloads.len())?;
        for (i, paylaod) in payloads.iter().enumerate() {
            let object = env.create_object()?;
            env.set_named_property(object, "id", env.create_string(&paylaod.id().to_string())?)?;
            env.set_named_property(
                object,
                "type",
                env.create_string(&paylaod.typ().to_string())?,
            )?;
            env.set_named_property(
                object,
                "data",
                env.create_typedarray(
                    TypedArrayType::Uint8Array,
                    paylaod.data().len(),
                    env.create_arraybuffer_from_slice(&paylaod.data())?,
                    0,
                )?,
            )?;
            env.set_element(array, i as u32, object)?;
        }
        Ok(array)
    }

    fn layer_data<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;
        env.create_typedarray(
            TypedArrayType::Uint8Array,
            layer.data().len(),
            env.create_arraybuffer_from_slice(&layer.data())?,
            0,
        )
    }

    fn layer_filter_expression<'env>(env: &'env Env, info: &CallbackInfo) -> Result<&'env Value> {
        let layer = env.unwrap::<Layer>(info.this())?;
        env.create_string(&unparse(&Expr::Token(layer.id())))
    }

    let class = env
        .define_class(
            "Layer",
            ctor,
            &[
                PropertyDescriptor::new_property(
                    env,
                    "id",
                    PropertyAttributes::DEFAULT,
                    layer_id,
                    false,
                ),
                PropertyDescriptor::new_method(
                    env,
                    "attr",
                    PropertyAttributes::DEFAULT,
                    layer_attr,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "attrs",
                    PropertyAttributes::DEFAULT,
                    layer_attrs,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "payloads",
                    PropertyAttributes::DEFAULT,
                    layer_payloads,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "data",
                    PropertyAttributes::DEFAULT,
                    layer_data,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "filterExpression",
                    PropertyAttributes::DEFAULT,
                    layer_filter_expression,
                    false,
                ),
            ],
        )
        .unwrap();
    env.create_ref(class)
}
