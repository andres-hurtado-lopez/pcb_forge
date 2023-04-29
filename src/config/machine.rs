use camino::Utf8PathBuf;
use std::collections::HashMap;
use uom::si::{
    angular_velocity::{revolution_per_minute, AngularVelocity},
    length::{millimeter, Length},
    power::{watt, Power},
    velocity::{millimeter_per_minute, Velocity},
};

use nalgebra::Vector2;
use serde::Deserialize;

use crate::parsing::parse_quantity;

#[derive(Debug, Deserialize)]
pub struct Machine {
    pub tools: HashMap<String, Tool>,

    /// Configurations for materials and tools that can be used for engraving.
    pub engraving_configs: HashMap<String, JobConfig>,

    /// Configurations for materials and tools that can be used for cutting.
    pub cutting_configs: HashMap<String, JobConfig>,

    /// The safe working area of the machine.
    pub workspace_area: WorkspaceSize,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct WorkspaceSize {
    #[serde(deserialize_with = "parse_quantity")]
    pub width: Length<uom::si::SI<f32>, f32>,
    #[serde(deserialize_with = "parse_quantity")]
    pub height: Length<uom::si::SI<f32>, f32>,
}

impl From<WorkspaceSize> for Vector2<Length<uom::si::SI<f32>, f32>> {
    fn from(value: WorkspaceSize) -> Self {
        Self::new(value.width, value.height)
    }
}

#[derive(Debug, Deserialize)]
pub struct JobConfig {
    /// The tool installed in the machine. For a milling machine, this would be the bit you installed.
    /// For a laser cutter, this should represent the laser.
    pub tool: Utf8PathBuf,

    #[serde(default = "distance_per_step_default")]
    pub distance_per_step: Length<uom::si::SI<f32>, f32>,

    /// The power of the tool. The unit depends on the tool.
    #[serde(flatten)]
    pub tool_power: ToolConfig,
}

fn distance_per_step_default() -> Length<uom::si::SI<f32>, f32> {
    Length::new::<millimeter>(0.1)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ToolConfig {
    Laser {
        #[serde(deserialize_with = "parse_quantity")]
        laser_power: Power<uom::si::SI<f32>, f32>,

        #[serde(deserialize_with = "parse_quantity")]
        work_speed: Velocity<uom::si::SI<f32>, f32>,
    },
    Drill {
        #[serde(deserialize_with = "parse_quantity")]
        spindle_rpm: AngularVelocity<uom::si::SI<f32>, f32>,

        #[serde(deserialize_with = "parse_quantity")]
        plunge_speed: Velocity<uom::si::SI<f32>, f32>,
    },
    EndMill {
        #[serde(deserialize_with = "parse_quantity")]
        spindle_rpm: AngularVelocity<uom::si::SI<f32>, f32>,

        /// The max depth that the end mill should plunge into the board.
        #[serde(deserialize_with = "parse_quantity")]
        max_cut_depth: Length<uom::si::SI<f32>, f32>,

        #[serde(deserialize_with = "parse_quantity")]
        plunge_speed: Velocity<uom::si::SI<f32>, f32>,

        #[serde(deserialize_with = "parse_quantity")]
        work_speed: Velocity<uom::si::SI<f32>, f32>,
    },
}

impl std::fmt::Display for ToolConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolConfig::Laser {
                laser_power,
                work_speed,
            } => write!(
                f,
                "Power: {} W, Work Speed: {} mm/m",
                laser_power.get::<watt>(),
                work_speed.get::<millimeter_per_minute>()
            ),
            ToolConfig::Drill {
                spindle_rpm,
                plunge_speed,
            } => write!(
                f,
                "RPM: {}, Plunge Speed: {} mm/m",
                spindle_rpm.get::<revolution_per_minute>(),
                plunge_speed.get::<millimeter_per_minute>()
            ),
            ToolConfig::EndMill {
                spindle_rpm,
                max_cut_depth,
                plunge_speed,
                work_speed,
            } => write!(
                f,
                "RPM: {}, Max Cut Depth: {} mm, Plunge Speed: {} mm/m, Work Speed: {} mm/m",
                spindle_rpm.get::<revolution_per_minute>(),
                max_cut_depth.get::<millimeter>(),
                plunge_speed.get::<millimeter_per_minute>(),
                work_speed.get::<millimeter_per_minute>()
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum Tool {
    #[serde(rename = "laser")]
    Laser(LaserConfig),

    #[serde(rename = "spindle")]
    Spindle(SpindleConfig),
}

#[derive(Debug, Deserialize)]
pub struct LaserConfig {
    #[serde(deserialize_with = "parse_quantity")]
    pub point_diameter: Length<uom::si::SI<f32>, f32>,

    #[serde(deserialize_with = "parse_quantity")]
    pub max_power: Power<uom::si::SI<f32>, f32>,
}

#[derive(Debug, Deserialize)]
pub struct SpindleConfig {
    #[serde(deserialize_with = "parse_quantity")]
    pub max_speed: AngularVelocity<uom::si::SI<f32>, f32>,

    pub bits: HashMap<String, SpindleBit>,
}

#[derive(Debug, Deserialize)]
pub enum SpindleBit {
    #[serde(rename = "drill")]
    Drill {
        #[serde(deserialize_with = "parse_quantity")]
        diameter: Length<uom::si::SI<f32>, f32>,
    },

    #[serde(rename = "end_mill")]
    EndMill {
        #[serde(deserialize_with = "parse_quantity")]
        diameter: Length<uom::si::SI<f32>, f32>,
    },
}
