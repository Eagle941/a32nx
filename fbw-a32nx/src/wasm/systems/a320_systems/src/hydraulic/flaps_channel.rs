use crate::systems::shared::arinc429::{Arinc429Word, SignStatus};
use systems::hydraulic::command_sensor_unit::{CSUMonitor, CSU};
use systems::shared::PositionPickoffUnit;

use systems::simulation::{
    InitContext, SimulationElement, SimulatorWriter, UpdateContext, VariableIdentifier, Write,
};

use uom::si::{angle::degree, f64::*, velocity::knot};

pub struct FlapsChannel {
    flaps_fppu_angle_id: VariableIdentifier,
    flap_actual_position_word_id: VariableIdentifier,

    flaps_demanded_angle: Angle,
    flaps_feedback_angle: Angle,
}

impl FlapsChannel {
    const HANDLE_ONE_CONF_AIRSPEED_THRESHOLD_KNOTS: f64 = 100.;
    const CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS: f64 = 210.;

    pub fn new(context: &mut InitContext) -> Self {
        Self {
            flaps_fppu_angle_id: context.get_identifier("FLAPS_FPPU_ANGLE".to_owned()),
            flap_actual_position_word_id: context
                .get_identifier("SFCC_FLAP_ACTUAL_POSITION_WORD".to_owned()),

            flaps_demanded_angle: Angle::new::<degree>(0.),
            flaps_feedback_angle: Angle::new::<degree>(0.),
        }
    }

    fn generate_configuration(&self, csu_monitor: &CSUMonitor, context: &UpdateContext) -> Angle {
        // Ignored `CSU::OutOfDetent` and `CSU::Fault` positions due to simplified SFCC.
        match (
            csu_monitor.get_previous_detent(),
            csu_monitor.get_current_detent(),
        ) {
            (CSU::Conf0, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    <= Self::HANDLE_ONE_CONF_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(120.22)
            }
            (CSU::Conf0, CSU::Conf1) => Angle::default(),
            (CSU::Conf1, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    > Self::CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::default()
            }
            (CSU::Conf1, CSU::Conf1) => self.flaps_demanded_angle,
            (_, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    <= Self::CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(120.22)
            }
            (_, CSU::Conf1) => Angle::default(),
            (_, CSU::Conf0) => Angle::default(),
            (from, CSU::Conf2) if from != CSU::Conf2 => Angle::new::<degree>(145.51),
            (from, CSU::Conf3) if from != CSU::Conf3 => Angle::new::<degree>(168.35),
            (from, CSU::ConfFull) if from != CSU::ConfFull => Angle::new::<degree>(251.97),
            (_, _) => self.flaps_demanded_angle,
        }
    }

    pub fn update(
        &mut self,
        context: &UpdateContext,
        csu_monitor: &CSUMonitor,
        flaps_feedback: &impl PositionPickoffUnit,
    ) {
        self.flaps_demanded_angle = self.generate_configuration(csu_monitor, context);
        self.flaps_feedback_angle = flaps_feedback.angle();
    }

    pub fn get_demanded_angle(&self) -> Angle {
        self.flaps_demanded_angle
    }

    pub fn get_feedback_angle(&self) -> Angle {
        self.flaps_feedback_angle
    }

    fn flap_actual_position_word(&self) -> Arinc429Word<f64> {
        Arinc429Word::new(
            self.flaps_feedback_angle.get::<degree>(),
            SignStatus::NormalOperation,
        )
    }
}
impl SimulationElement for FlapsChannel {
    fn write(&self, writer: &mut SimulatorWriter) {
        writer.write(&self.flaps_fppu_angle_id, self.flaps_feedback_angle);

        writer.write(
            &self.flap_actual_position_word_id,
            self.flap_actual_position_word(),
        );
    }
}
