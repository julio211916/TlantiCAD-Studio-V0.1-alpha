//! S343-S347: G-code Post-Processor & Machine Output
//!
//! Generate machine-specific code from toolpaths.

use serde::{Deserialize, Serialize};
use crate::toolpath::{Toolpath, MotionType};

/// G-code dialect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GCodeDialect {
    Fanuc,
    Heidenhain,
    Siemens840D,
    RolandDWX,
    DeguDent,
    ImesMcoreI,
    VhfCam,
    Generic,
}

/// G-code line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeLine {
    pub line_number: usize,
    pub code: String,
    pub comment: Option<String>,
}

impl GCodeLine {
    pub fn new(number: usize, code: impl Into<String>) -> Self {
        Self { line_number: number, code: code.into(), comment: None }
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn to_string_formatted(&self) -> String {
        match &self.comment {
            Some(c) => format!("N{} {} ({})", self.line_number, self.code, c),
            None => format!("N{} {}", self.line_number, self.code),
        }
    }
}

/// Post-processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessorConfig {
    pub dialect: GCodeDialect,
    pub header_lines: Vec<String>,
    pub footer_lines: Vec<String>,
    pub decimal_places: u8,
    pub use_line_numbers: bool,
    pub coolant_code: String,
    pub spindle_cw_code: String,
    pub rapid_code: String,
    pub linear_code: String,
    pub arc_cw_code: String,
    pub arc_ccw_code: String,
}

impl PostProcessorConfig {
    pub fn fanuc() -> Self {
        Self {
            dialect: GCodeDialect::Fanuc,
            header_lines: vec!["%".into(), "O0001".into(), "G90 G21".into()],
            footer_lines: vec!["M30".into(), "%".into()],
            decimal_places: 3,
            use_line_numbers: true,
            coolant_code: "M08".into(),
            spindle_cw_code: "M03".into(),
            rapid_code: "G00".into(),
            linear_code: "G01".into(),
            arc_cw_code: "G02".into(),
            arc_ccw_code: "G03".into(),
        }
    }

    pub fn roland_dwx() -> Self {
        Self {
            dialect: GCodeDialect::RolandDWX,
            header_lines: vec!["%".into(), "G90 G21 G17".into()],
            footer_lines: vec!["M05".into(), "G28 G91 Z0".into(), "M30".into()],
            decimal_places: 4,
            use_line_numbers: false,
            coolant_code: "M07".into(),
            spindle_cw_code: "M03".into(),
            rapid_code: "G00".into(),
            linear_code: "G01".into(),
            arc_cw_code: "G02".into(),
            arc_ccw_code: "G03".into(),
        }
    }
}

/// Generated G-code program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeProgram {
    pub lines: Vec<GCodeLine>,
    pub dialect: GCodeDialect,
    pub total_distance_mm: f64,
    pub estimated_time_min: f64,
}

impl GCodeProgram {
    pub fn to_string_full(&self) -> String {
        self.lines.iter()
            .map(|l| l.to_string_formatted())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn line_count(&self) -> usize { self.lines.len() }
}

fn format_coord(val: f64, decimals: u8) -> String {
    format!("{:.1$}", val, decimals as usize)
}

fn motion_to_gcode(motion: MotionType, config: &PostProcessorConfig) -> &str {
    match motion {
        MotionType::Rapid | MotionType::Retract => &config.rapid_code,
        MotionType::Linear | MotionType::Plunge => &config.linear_code,
        MotionType::ArcCW => &config.arc_cw_code,
        MotionType::ArcCCW => &config.arc_ccw_code,
    }
}

/// Generate G-code for a toolpath
pub fn generate_gcode(toolpath: &Toolpath, config: &PostProcessorConfig) -> GCodeProgram {
    let mut lines = Vec::new();
    let mut line_num = 10;
    let dp = config.decimal_places;

    // Header
    for h in &config.header_lines {
        lines.push(GCodeLine::new(line_num, h.clone()));
        line_num += 10;
    }

    // Find first segment's spindle RPM for start command
    let rpm = toolpath.segments.first().map_or(15000.0, |s| s.spindle_rpm);
    lines.push(GCodeLine::new(line_num, format!("{} S{}", config.spindle_cw_code, rpm as u32)));
    line_num += 10;

    // Coolant on
    lines.push(GCodeLine::new(line_num, config.coolant_code.clone()));
    line_num += 10;

    let mut total_dist = 0.0_f64;

    for seg in &toolpath.segments {
        let gcode_cmd = motion_to_gcode(seg.motion, config);

        let code = if matches!(seg.motion, MotionType::Rapid | MotionType::Retract) {
            format!("{} X{} Y{} Z{}", gcode_cmd,
                format_coord(seg.end.x, dp),
                format_coord(seg.end.y, dp),
                format_coord(seg.end.z, dp))
        } else {
            format!("{} X{} Y{} Z{} F{}", gcode_cmd,
                format_coord(seg.end.x, dp),
                format_coord(seg.end.y, dp),
                format_coord(seg.end.z, dp),
                seg.feed_rate_mm_min as u32)
        };

        total_dist += seg.length();
        lines.push(GCodeLine::new(line_num, code));
        line_num += 10;
    }

    // Footer
    for f in &config.footer_lines {
        lines.push(GCodeLine::new(line_num, f.clone()));
        line_num += 10;
    }

    let avg_feed = toolpath.segments.first()
        .map_or(1000.0, |s| s.feed_rate_mm_min.max(100.0));
    let time = total_dist / avg_feed;

    GCodeProgram {
        lines,
        dialect: config.dialect,
        total_distance_mm: total_dist,
        estimated_time_min: time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolpath::*;
    use nalgebra::Point3;

    fn make_test_toolpath() -> Toolpath {
        let mut tp = Toolpath::new(ToolpathStrategy::Roughing, 2.0);
        tp.segments = vec![
            ToolpathSegment {
                start: Point3::new(0.0, 0.0, 5.0),
                end: Point3::new(0.0, 0.0, 0.0),
                motion: MotionType::Plunge,
                feed_rate_mm_min: 300.0,
                spindle_rpm: 15000.0,
            },
            ToolpathSegment {
                start: Point3::new(0.0, 0.0, 0.0),
                end: Point3::new(10.0, 0.0, 0.0),
                motion: MotionType::Linear,
                feed_rate_mm_min: 1000.0,
                spindle_rpm: 15000.0,
            },
            ToolpathSegment {
                start: Point3::new(10.0, 0.0, 0.0),
                end: Point3::new(10.0, 0.0, 5.0),
                motion: MotionType::Retract,
                feed_rate_mm_min: 5000.0,
                spindle_rpm: 15000.0,
            },
        ];
        tp
    }

    #[test]
    fn test_generate_fanuc() {
        let tp = make_test_toolpath();
        let cfg = PostProcessorConfig::fanuc();
        let prog = generate_gcode(&tp, &cfg);
        assert!(prog.line_count() > 5);
        let text = prog.to_string_full();
        assert!(text.contains("G01"));
        assert!(text.contains("G00"));
    }

    #[test]
    fn test_generate_roland() {
        let tp = make_test_toolpath();
        let cfg = PostProcessorConfig::roland_dwx();
        let prog = generate_gcode(&tp, &cfg);
        assert!(prog.total_distance_mm > 0.0);
    }

    #[test]
    fn test_gcode_line_format() {
        let l = GCodeLine::new(10, "G01 X1.000 Y2.000").with_comment("move");
        let s = l.to_string_formatted();
        assert!(s.contains("(move)"));
    }

    #[test]
    fn test_format_coord() {
        assert_eq!(format_coord(1.2345, 3), "1.234");
        assert_eq!(format_coord(0.0, 4), "0.0000");
    }
}
