use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start: String,
    pub end: String,
    pub text: String,
}

pub fn parse_srt(srt: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    // Normalize \r\n to \n for Windows compatibility
    let srt = srt.replace("\r\n", "\n");

    for block in srt.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            continue;
        }

        let index = match lines[0].trim().parse::<u32>() {
            Ok(i) => i,
            Err(_) => continue,
        };

        let time_parts: Vec<&str> = lines[1].split(" --> ").collect();
        if time_parts.len() != 2 {
            continue;
        }

        let text = lines[2..].join("\n");

        entries.push(SubtitleEntry {
            index,
            start: time_parts[0].trim().to_string(),
            end: time_parts[1].trim().to_string(),
            text,
        });
    }

    entries
}

/// Convert parsed entries to ASS format
pub fn entries_to_ass(entries: &[SubtitleEntry]) -> String {
    let mut ass = String::with_capacity(1024 + entries.len() * 80);
    ass.push_str("[Script Info]\r\n");
    ass.push_str("ScriptType: v4.00+\r\n");
    ass.push_str("PlayResX: 1920\r\n");
    ass.push_str("PlayResY: 1080\r\n");
    ass.push_str("WrapStyle: 0\r\n\r\n");
    ass.push_str("[V4+ Styles]\r\n");
    ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\r\n");
    ass.push_str("Style: Default,Noto Sans CJK TC,68,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,2.5,1,2,30,30,40,1\r\n\r\n");
    ass.push_str("[Events]\r\n");
    ass.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\r\n");

    for entry in entries {
        let start = srt_time_to_ass(&entry.start);
        let end = srt_time_to_ass(&entry.end);
        let text = entry.text.replace('\n', "\\N");
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\r\n",
            start, end, text
        ));
    }

    ass
}

/// Convert parsed entries to plain text
pub fn entries_to_txt(entries: &[SubtitleEntry]) -> String {
    entries
        .iter()
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn srt_time_to_ass(time: &str) -> String {
    // SRT: 00:01:23,456 -> ASS: 0:01:23.46
    let t = time.trim().replace(',', ".");
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() == 3 {
        let h: u32 = parts[0].parse().unwrap_or(0);
        let m = parts[1];
        // Split seconds from centiseconds
        let sec_parts: Vec<&str> = parts[2].split('.').collect();
        let s = sec_parts[0];
        let cs = if sec_parts.len() > 1 {
            let frac = sec_parts[1];
            if frac.len() >= 2 { &frac[..2] } else { frac }
        } else {
            "00"
        };
        format!("{}:{:0>2}:{:0>2}.{}", h, m, s, cs)
    } else {
        t
    }
}
