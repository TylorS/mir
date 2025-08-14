//! WASI component model support

/// WASI component representation
pub struct WasiComponent {
    pub interfaces: Vec<ComponentInterface>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComponentInterface {
    pub name: String,
    pub functions: Vec<InterfaceFunction>,
}

#[derive(Debug, Clone)]
pub struct InterfaceFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

impl WasiComponent {
    pub fn new() -> Self {
        WasiComponent {
            interfaces: Vec::new(),
            exports: Vec::new(),
        }
    }
    
    pub fn add_interface(&mut self, interface: ComponentInterface) {
        self.interfaces.push(interface);
    }
}