use anyhow::Result;
use clap::ArgEnum;
use serde::Serialize;

use crate::{
    shared::config::Config,
    server::grpc::api::{MonitorResponse, TrackStatus},
};
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
    pub fn format_output(&self, response: MonitorResponse, status: TrackStatus, output_format: &str) -> Output {
        let text_template = &self.config.text_template;
        if let Some(track) = response.track {
            let mut class = vec![];
            let mut separator = "-";
            if response.is_liked.unwrap_or_default() {
                class.push("liked".to_string());
                separator = "+";
            }
            class.push(status.into());
            let text = match (track.artist, track.title) {
                (Some(artist), Some(title)) => {
                    let status_icon = match status {
                        TrackStatus::Playing => &text_template.playing,
                        TrackStatus::Paused => &text_template.paused,
                        _ => "" // unable to get player state
                    };
                    let liked_icon = match class.contains(&"liked".to_string()) {
                        true => &text_template.liked,
                        _ => "" // not liked, so display no icon
                    };
                    output_format
                        .replace("{artist}", &artist)
                        .replace("{title}", &title)
                        .replace("{separator}", separator)
                        .replace("{status}", status_icon)
                        .replace("{liked}", liked_icon)
                }
                (Some(artist), None) => artist,
                (None, Some(title)) => title,
                (None, None) => "".to_string(),
            };
            Output {
                text,
                tooltip: track.album,
                class: Some(class),
            }
        } else if status == TrackStatus::Added {
            Output {
                text: "Added to library!".to_string(),
                tooltip: None,
                class: Some(vec![status.into()]),
            }
        } else if status == TrackStatus::Removed {
            Output {
                text: "Removed from library!".to_string(),
                tooltip: None,
                class: Some(vec![status.into()]),
            }
        } else {
            Output {
                text: "".to_string(),
                tooltip: None,
                class: None,
            }
        }
    }

    pub fn print(&self, output: Output) -> Result<()> {
        match self.output_type {
            OutputType::Waybar => {
                let json = serde_json::to_string(&Output {
                    text: html_escape::encode_text(&output.text.to_string())
                        .into(),
                    ..output
                })?;
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
                println!("%{{F{color}}}{}%{{F-}}", output.text);
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
