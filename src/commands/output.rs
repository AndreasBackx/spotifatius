use anyhow::Result;
use clap::ArgEnum;
use serde::Serialize;

use crate::shared::config::Config;

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

pub struct OutputFormatter {
    pub output_type: OutputType,
    pub config: Config,
}

impl OutputFormatter {
    pub fn print(&self, output: Output) -> Result<()> {
        match self.output_type {
            OutputType::Waybar => {
                let json = serde_json::to_string(&output)?;
                println!("{}", json);
            }
            OutputType::Polybar => {
                let color = output
                    .class
                    .unwrap_or_default()
                    .iter()
                    .find_map(|color| {
                        self.config
                            .polybar
                            .colors
                            .get(color)
                            .map(|color| color.to_owned())
                    })
                    .unwrap_or_default();
                println!("%{{F{color}}}{}{{%F-}}", output.text);
            }
        }
        Ok(())
    }
}

// impl Output {
//     pub fn print(&self, output_type: OutputType, config: Config) -> Result<()> {
//         match output_type {
//             OutputType::Waybar => {
//                 let json = serde_json::to_string(&self)?;
//                 println!("{}", json);
//             }
//             OutputType::Polybar => {
//                 println!("{}", self.text);
//             }
//         }
//         Ok(())
//     }
// }

impl Default for Output {
    fn default() -> Self {
        Output {
            text: "".to_string(),
            tooltip: None,
            class: None,
        }
    }
}
