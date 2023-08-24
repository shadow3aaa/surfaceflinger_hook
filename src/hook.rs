/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
use std::{convert::AsRef, fs, ptr};

use dobby_api::{hook, resolve_func_addr, undo_hook, Address};
use goblin::Object;

use crate::error::{Error, Result};

const LIB_PATH: &str = "/system/lib64/libsurfaceflinger.so";

pub struct SymbolHooker {
    symbols: Vec<String>,
}

impl SymbolHooker {
    pub fn new() -> Result<Self> {
        let buffer = fs::read(LIB_PATH)?;

        let Object::Elf(lib) = Object::parse(&buffer)? else {
            return Err(Error::Other("Not an elf lib"));
        };

        if !lib.is_lib {
            return Err(Error::Other("Not an elf lib"))?;
        }

        let symbols = lib
            .dynstrtab
            .to_vec()?
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();
        Ok(Self { symbols })
    }

    pub unsafe fn find_and_hook<I, S>(&self, s: I, replace_func: Address) -> Result<Address>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let s: Vec<_> = s.into_iter().collect();
        let symbol = self.find_symbol(&s)?;
        let mut save_temp = ptr::null_mut();

        let _ = undo_hook(symbol);
        hook(symbol, replace_func, Some(&mut save_temp))?;

        Ok(save_temp)
    }

    fn find_symbol<S: AsRef<str>>(&self, s: &[S]) -> Result<Address> {
        let symbol = self
            .symbols
            .iter()
            .filter(|symbol| {
                !symbol.starts_with("_ZThn") && s.iter().all(|s| symbol.contains(s.as_ref()))
            }) // 关键字匹配，排除虚函数
            .min_by_key(|s| s.len()) // 更短的更匹配目标
            .ok_or(Error::Symbol)?;

        Ok(resolve_func_addr(None, symbol)?)
    }
}
