use std::{path::{Path, PathBuf}, fs::File};

use serde::Serialize;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;


type Result<T> = anyhow::Result<T>;


const MODIFIER_SHIFT: usize = 1;
const MODIFIER_CONTROL: usize = 2;
const MODIFIER_COMMAND: usize = 4;
const MODIFIER_OPTION: usize = 8;



#[derive(Serialize,Deserialize)]
struct ConfigItem {
    key: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    when: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Value>
}

impl From<&KeyBinding> for ConfigItem {
    fn from(kb: &KeyBinding) -> Self {
        ConfigItem { 
            key:  format!("{}", kb.keys), 
            command: kb.command.clone(), 
            when: kb.when.clone(), 
            args: kb.args.clone() 
        }
    }
}


#[derive(Clone)]
struct KeyBinding {
    keys: KeyRule,
    command : String,
    when: Option<String>,
    args: Option<Value>
}


impl KeyBinding {
    fn has_control(&self) -> bool {
        self.keys.first.modifiers & MODIFIER_CONTROL != 0
    }

    fn copy_disabled(&self) -> Self {
        if self.command.starts_with("-") {
            self.clone()
        } else {
            KeyBinding {
                keys: self.keys.clone(),
                command: format!("-{}", self.command),
                when: self.when.clone(),
                args: self.args.clone()
            }
        }
    }

}


impl From<ConfigItem> for KeyBinding {
    fn from(ci: ConfigItem) -> Self {
        KeyBinding {
            keys: parse_key_sequence(&ci.key),
            command: ci.command,
            when: ci.when,
            args: ci.args
        }
    }
}

#[derive(Clone,PartialEq, Eq, Hash)]
struct KeyRule {
    first: Key,
    second: Option<Key>
}

impl std::fmt::Display for KeyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.first.fmt(f)?;
        if let Some(s) = &self.second {
            write!(f, " ")?;
            s.fmt(f)?;
        }
        Ok(())
    }
}





#[derive(Clone,PartialEq, Eq, Hash)]
struct Key {
    modifiers: usize,
    key: String
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers & MODIFIER_COMMAND != 0 {
            write!(f, "meta+")?
        }
        if self.modifiers & MODIFIER_OPTION != 0 {
            write!(f, "alt+")?
        }
        if self.modifiers & MODIFIER_CONTROL != 0 {
            write!(f, "ctrl+")?
        }
        if self.modifiers & MODIFIER_SHIFT != 0 {
            write!(f, "shift+")?
        }
        write!(f, "{}", self.key)
    }
}

fn anykey() -> Key {
    Key { modifiers: 0, key: String::new() }
}

fn main() -> color_eyre::eyre::Result<()> {

    color_eyre::install()?;

    let bindings = load_defaults().unwrap();

    let mut bneu: Vec<ConfigItem> = vec!();

    for k in bindings.iter() {
        // println!("{:x} {:>10} {}", k.keys.first.modifiers, k.keys.first.key, k.command)
        map_ctrl_binding(k).iter().for_each(|i| bneu.push(ConfigItem::from(i)));
    };

    println!("{}", serde_json::to_string_pretty(&bneu)?);

    Ok(())
}



fn load_defaults() -> Result<Vec<KeyBinding>> {

    let path = PathBuf::from("keys/default.json");
    let defaults_json: Vec<ConfigItem> = serde_json::from_reader(File::open(path)?)?;

    let bindings = defaults_json.into_iter().map(|item| KeyBinding::from(item)).collect();
    Ok(bindings)
}

fn parse_key_sequence(code: &str) -> KeyRule {
    let mut iter = code.split_ascii_whitespace().map(|k| parse_one_key(k));
    let k1 = iter.next();
    let k2 = iter.next();
    KeyRule {
        first: k1.unwrap_or_else(|| anykey()),
        second: k2
    }
}


fn parse_one_key(key: &str) -> Key {

    let mut modifiers: usize = 0;
    let mut thekey: Option<String> = None;

    for k in key.to_lowercase().split_inclusive("+") {
        match k {
            "ctrl+" => modifiers |= MODIFIER_CONTROL,
            "shift+" => modifiers |= MODIFIER_SHIFT,
            "super+" => modifiers |= MODIFIER_COMMAND,
            "cmd+" => modifiers |= MODIFIER_COMMAND,
            "meta+" => modifiers |= MODIFIER_COMMAND,
            "win+" => modifiers |= MODIFIER_COMMAND,
            "alt+" => modifiers |= MODIFIER_OPTION,
            _ => thekey = Some(String::from(k))
        }
    }

    Key {
        modifiers: modifiers,
        key: thekey.unwrap_or_else(|| String::new())
    }
}


fn map_ctrl_binding(kb: &KeyBinding) -> Vec<KeyBinding> {

    let mut r = vec!();

    if kb.keys.first.modifiers & MODIFIER_CONTROL != 0 {
        if let Some(k1) = map_ctrl_to_cmd(&kb.keys.first) {
        
            let k2 = match &kb.keys.second {
                Some(k) => map_ctrl_to_cmd(k),
                None => None
            };
    
            r.push(kb.copy_disabled());
            r.push(KeyBinding {
                keys: KeyRule { first: k1, second: k2 },
                command: kb.command.clone(),
                when: kb.when.clone(),
                args: kb.args.clone(),
            })
    
    
        }
    }

    r

}

fn map_ctrl_to_cmd(key: &Key) -> Option<Key> {

    if key.modifiers & MODIFIER_CONTROL != 0 && key.modifiers & MODIFIER_COMMAND == 0 {
        let xmod = (key.modifiers ^ MODIFIER_CONTROL) | MODIFIER_COMMAND;
        Some(Key { modifiers: xmod, key: key.key.clone() })
    } else {
        Some(key.clone())
    }


}
