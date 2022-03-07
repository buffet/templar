use std::rc::Rc;

use anyhow::Result;
use rlua::prelude::*;
use std::fmt::Debug;

use super::{parser::ParserConfig, template::DynGenerator};

pub(super) trait Generator: Debug {
    /* Generates a String from a Directive. */
    fn generate(&self, _: &LuaContext) -> Result<String>;

    // NOTE: Might be sensible to put this method in ParserConfig and possibly add another trait?
    // idk lets keep it simple for now
    fn _display(&self, _: ParserConfig) -> Result<String> {
        unimplemented!()
    }
}

// Text
//impl<T> Generator for T
//where
//    T: AsRef<str> + Debug,
//{
//    fn generate(&self, _: &LuaContext) -> Result<String> {
//        Ok(self.as_ref().to_string())
//    }
//}

impl Generator for String {
    fn generate(&self, _: &LuaContext) -> Result<String> {
        Ok(self.clone())
    }
}

impl Generator for &str {
    fn generate(&self, _: &LuaContext) -> Result<String> {
        Ok(self.to_string())
    }
}

#[derive(Debug, Clone)]
pub(super) struct If {
    pub condition: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for If {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let condition_result = lua_context.load(&self.condition).eval::<bool>()?;
        if condition_result {
            self.blocks.generate(lua_context)
        } else {
            Ok("".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct IfElse {
    pub condition: String,
    pub if_blocks: Vec<Rc<dyn Generator>>,
    pub else_blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for IfElse {
    fn generate(&self, _: &LuaContext) -> Result<String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(super) struct Include {
    pub path: String,
}

impl Generator for Include {
    fn generate(&self, _: &LuaContext) -> Result<String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(super) struct Transform {
    pub input_name: String,
    pub transform: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for Transform {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let blocks = self.blocks.generate(lua_context)?;
        lua_context.globals().set(self.input_name.clone(), blocks)?;
        let r = lua_context.load(&self.transform).eval::<String>()?;
        lua_context.globals().set(self.input_name.clone(), LuaNil)?;
        Ok(r)
    }
}

impl Generator for Vec<DynGenerator> {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let mut result = String::new();
        for block in self {
            result.push_str(&block.generate(lua_context)?);
        }
        Ok(result.to_string())
    }
}
