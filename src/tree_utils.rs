use std::collections::BTreeMap;
use std::io::{self,Write};
use std::path::Path;

#[derive(Debug)]
pub struct Node{
    pub children:BTreeMap<String,Node>,
    pub is_dir:bool,
}

impl Node{
    pub fn new(is_dir:bool)->Self{
        Self {
            children: BTreeMap::new(),
            is_dir,
        }
    }
    pub fn insert_path(&mut self,path:&Path,is_dir:bool){
        let mut current = self;
        for component in path.components(){
            let name = component.as_os_str().to_string_lossy().to_string();
            current = current
                .children.entry(name.clone())
                .or_insert_with(|| Node::new( true));
        }
        current.is_dir = is_dir;
    }
    pub fn print<W: Write>(&self, indent: &str, writer: &mut W) -> io::Result<()>{
        let len = self.children.len();
        for (i,(name,child)) in self.children.iter().enumerate(){
            let is_last = i == len - 1;
            let connector = if is_last {"└── "} else {"├── "};
            writeln!(writer, "{}{}{}{}", indent, connector, name,if child.is_dir { "/" } else { "" })?;
            if !child.children.is_empty(){
                let new_indent = format!(
                    "{}{}",
                    indent,
                    if is_last{"    "} else{"│   "}
                );
                child.print(&new_indent, writer)?;
            }
        }
        Ok(())
    }
}