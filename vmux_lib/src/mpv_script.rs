pub fn mpv_script(video: &str, audio: &str, media_title: &str) -> String {
    mpv_script_ex(video, audio, media_title, None)
}
pub fn mpv_script_ex(
    video: &str,
    audio: &str,
    media_title: &str,
    chapters: Option<String>,
) -> String {
    let skrpt_prefix = r#"#!/usr/bin/env bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

"#;

    let chaps = if let Some(e) = chapters {
        format!("--chapters-file={}", e)
    } else {
        "".to_owned()
    };
    format!(
        "{}{}",
        skrpt_prefix,
        format!(
            "mpv {} --audio-file={} {} --force-media-title=\"{}\" --cache=no",
            video, audio, chaps, media_title
        )
    )
}
