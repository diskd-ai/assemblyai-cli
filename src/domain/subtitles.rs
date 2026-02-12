#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiarizedUtterance {
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: String,
    pub text: String,
}

impl DiarizedUtterance {
    pub fn new(start_ms: u64, end_ms: u64, speaker: String, text: String) -> Option<Self> {
        if end_ms <= start_ms {
            return None;
        }

        let speaker = speaker.trim().to_string();
        if speaker.is_empty() {
            return None;
        }

        let text = text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        Some(Self {
            start_ms,
            end_ms,
            speaker,
            text,
        })
    }
}

pub fn format_diarized_text(utterances: &[DiarizedUtterance]) -> String {
    let mut out = String::new();
    for utterance in utterances {
        out.push_str("Speaker ");
        out.push_str(&utterance.speaker);
        out.push_str(": ");
        out.push_str(&utterance.text);
        out.push('\n');
    }
    out
}

pub fn format_diarized_srt(utterances: &[DiarizedUtterance], chars_per_caption: u32) -> String {
    let max_chars = chars_per_caption as usize;
    let mut out = String::new();
    let mut idx: u32 = 1;

    for utterance in utterances {
        let speaker_prefix = format!("Speaker {}: ", utterance.speaker);
        let available = max_chars.saturating_sub(speaker_prefix.len()).max(1);
        let segments = split_text_by_max_chars(&utterance.text, available);
        for (seg_idx, segment) in segments.iter().enumerate() {
            let (start, end) = segment_time(utterance.start_ms, utterance.end_ms, seg_idx, segments.len());
            out.push_str(&idx.to_string());
            out.push('\n');
            out.push_str(&format_srt_time(start));
            out.push_str(" --> ");
            out.push_str(&format_srt_time(end));
            out.push('\n');
            out.push_str(&speaker_prefix);
            out.push_str(segment);
            out.push_str("\n\n");
            idx = idx.saturating_add(1);
        }
    }

    out
}

pub fn format_diarized_vtt(utterances: &[DiarizedUtterance], chars_per_caption: u32) -> String {
    let max_chars = chars_per_caption as usize;
    let mut out = String::new();
    out.push_str("WEBVTT\n\n");

    for utterance in utterances {
        let speaker_prefix = format!("Speaker {}: ", utterance.speaker);
        let available = max_chars.saturating_sub(speaker_prefix.len()).max(1);
        let segments = split_text_by_max_chars(&utterance.text, available);
        for (seg_idx, segment) in segments.iter().enumerate() {
            let (start, end) = segment_time(utterance.start_ms, utterance.end_ms, seg_idx, segments.len());
            out.push_str(&format_vtt_time(start));
            out.push_str(" --> ");
            out.push_str(&format_vtt_time(end));
            out.push('\n');
            out.push_str(&speaker_prefix);
            out.push_str(segment);
            out.push_str("\n\n");
        }
    }

    out
}

fn segment_time(start_ms: u64, end_ms: u64, seg_idx: usize, seg_count: usize) -> (u64, u64) {
    let duration = end_ms.saturating_sub(start_ms);
    let n = seg_count.max(1) as u64;
    let i = seg_idx as u64;

    let seg_start = start_ms.saturating_add(duration.saturating_mul(i) / n);
    let seg_end = if seg_idx + 1 >= seg_count {
        end_ms
    } else {
        start_ms.saturating_add(duration.saturating_mul(i + 1) / n)
    };

    (seg_start, seg_end.max(seg_start + 1))
}

fn split_text_by_max_chars(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![text.to_string()];
    }
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.len().saturating_add(1).saturating_add(word.len()) <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            segments.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    if segments.is_empty() {
        vec![text.to_string()]
    } else {
        segments
    }
}

fn format_srt_time(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let ms_part = ms % 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02},{ms_part:03}")
}

fn format_vtt_time(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let ms_part = ms % 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{ms_part:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_srt_with_speaker_prefix() {
        let utterances = vec![
            DiarizedUtterance::new(2000, 5000, "1".to_string(), "Hello world".to_string()).expect("utterance"),
            DiarizedUtterance::new(6000, 7000, "2".to_string(), "Hi".to_string()).expect("utterance"),
        ];

        let srt = format_diarized_srt(&utterances, 128);
        assert!(srt.contains("00:00:02,000 --> 00:00:05,000"));
        assert!(srt.contains("Speaker 1: Hello world"));
        assert!(srt.contains("Speaker 2: Hi"));
    }

    #[test]
    fn formats_vtt_with_header() {
        let utterances = vec![DiarizedUtterance::new(0, 1000, "1A".to_string(), "Test".to_string()).expect("utterance")];
        let vtt = format_diarized_vtt(&utterances, 128);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:00.000 --> 00:00:01.000"));
        assert!(vtt.contains("Speaker 1A: Test"));
    }

    #[test]
    fn splits_long_text() {
        let u = DiarizedUtterance::new(
            0,
            10_000,
            "1".to_string(),
            "one two three four five six seven eight nine ten".to_string(),
        )
        .expect("utterance");

        let srt = format_diarized_srt(&[u], 20);
        let count = srt.lines().filter(|l| l.contains("-->")).count();
        assert!(count >= 2, "expected multiple segments, got {count}");
    }
}
