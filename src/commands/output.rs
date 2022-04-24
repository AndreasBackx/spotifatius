use anyhow::Result;
use clap::ArgEnum;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Output {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
}

#[derive(ArgEnum, Clone, Copy)]
pub enum OutputType {
    Waybar,
    Polybar,
}

impl Output {
    pub fn print(&self, output_type: OutputType) -> Result<()> {
        match output_type {
            OutputType::Waybar => {
                let json = serde_json::to_string(&self)?;
                println!("{}", json);
            }
            OutputType::Polybar => {
                println!("{}", self.text);
            }
        }
        Ok(())
    }
}

impl Default for Output {
    fn default() -> Self {
        Output {
            text: "".to_string(),
            tooltip: None,
            class: None,
        }
    }
}
